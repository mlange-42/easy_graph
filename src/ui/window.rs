//!
//! Provides a simple window for drawing primitive shapes.
//!
//! Also used by [`Chart`](../chart/struct.Chart.html).
//!
//! # Example
//! ```
//! use easy_graph::ui::window::WindowBuilder;
//! use easy_graph::color::style::{WHITE, RED, Color, ShapeStyle};
//! use easy_graph::ui::drawing::IntoDrawingArea;
//! use easy_graph::ui::element::Circle;
//!
//! fn main() {
//!     let mut win = WindowBuilder::new()
//!            .with_title("Test window")
//!            .with_dimensions(600, 400)
//!            .with_fps_limit(30.0)
//!            .build();
//!
//!     for _ in 0..10 { // change upper limit for longer run!
//!         win.draw(|b| {
//!             let root = b.into_drawing_area();
//!             root.fill(&WHITE).unwrap();
//!             root.draw(&Circle::new((50, 50), 15, ShapeStyle::from(&RED).filled())).unwrap();
//!         });
//!     }
//! }
//! ```
//!

use minifb::{Scale, ScaleMode};
use plotters::drawing::bitmap_pixel::RGBPixel;
use plotters::prelude::*;
use std::time::{Duration, SystemTime};

///
/// Builder for [`BufferWindow`](struct.BufferWindow.html). See [`window`](index.html) module docs for an example.
///
pub struct WindowBuilder {
    dim: (usize, usize),
    title: String,
    scale: Scale,
    max_fps: Option<f64>,
    max_fps_skip: Option<f64>,
    position: Option<(isize, isize)>,
}

impl WindowBuilder {
    /// Creates a default `WindowBuilder`.
    pub fn new() -> Self {
        WindowBuilder {
            dim: (600, 400),
            title: "".to_string(),
            scale: Scale::X1,
            max_fps: None,
            max_fps_skip: None,
            position: None,
        }
    }
    /// Sets the dimensions of the window in screen pixels.
    /// Final size is also subject to window's scale factor.
    pub fn with_dimensions(mut self, width: usize, height: usize) -> Self {
        self.dim = (width, height);
        self
    }
    /// Sets the window's title.
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
    /// Sets the chart's FPS limit. Slows down the process updating the chart.
    ///
    /// The chart's draw() method will block to achieve the FPS limit.
    pub fn with_fps_limit(mut self, max_fps: f64) -> Self {
        self.max_fps = Some(max_fps);
        self
    }
    /// Sets the chart's FPS skip. Skips updates, but does not slow down the process updating the chart.
    ///
    /// The chart's draw() method will skip frames to achieve the FPS limit.
    pub fn with_fps_skip(mut self, max_fps: f64) -> Self {
        self.max_fps_skip = Some(max_fps);
        self
    }
    /// Sets the window's scale factor. Particularly useful to display raster-like data efficiently.
    ///
    /// # Example
    /// ```
    /// use easy_graph::ui::window::WindowBuilder;
    /// use easy_graph::ui::Scale;
    /// use easy_graph::color::style::{WHITE, BLACK};
    /// use easy_graph::ui::drawing::IntoDrawingArea;
    ///
    /// let size = 100;
    ///
    /// let mut win = WindowBuilder::new()
    ///     .with_title("Scaled")
    ///     .with_dimensions(size, size)
    ///     .with_scale(Scale::X4)
    ///     .build();
    ///
    /// for _ in 0..10 { // change upper limit for longer run!
    ///     win.draw(|b| {
    ///         let root = b.into_drawing_area();
    ///         root.fill(&WHITE).unwrap();
    ///         for x in 0..size {
    ///             for y in 0..size {
    ///                 root.draw_pixel((x as i32, y as i32), if x % 2 == 0 {&WHITE} else {&BLACK} ).unwrap();
    ///             }           
    ///         }       
    ///     });
    /// }
    /// ```
    pub fn with_scale(mut self, scale: Scale) -> Self {
        self.scale = scale;
        self
    }
    /// Sets the position of the window's upper left corner in screen pixels.
    pub fn with_position(mut self, pos: (isize, isize)) -> Self {
        self.position = Some(pos);
        self
    }

    /// Builds the window.
    pub fn build(self) -> BufferWindow {
        let mut win = BufferWindow::new(
            &self.title,
            self.dim,
            self.max_fps,
            self.max_fps_skip,
            self.scale,
            true,
        );
        if let Some(pos) = self.position {
            win.window.set_position(pos.0, pos.1);
        }
        win
    }
}

///
/// A window for simple drawing. Construct using [`WindowBuilder`](struct.WindowBuilder.html).
///
/// See [`window`](index.html) module docs for an example.
///
#[allow(dead_code)]
pub struct BufferWindow {
    window: minifb::Window,
    pub buffer_u8: Vec<u8>,
    buffer_u32: Vec<u32>,
    dim: (usize, usize),
    fps_skip: UpdateSkip,
}

impl BufferWindow {
    pub(crate) fn new(
        title: &str,
        dim: (usize, usize),
        max_fps: Option<f64>,
        fps_skip: Option<f64>,
        scale: Scale,
        resize: bool,
    ) -> Self {
        let buffer_u8 = vec![0 as u8; (3 * dim.0 * dim.1) as usize];
        let buffer_u32 = vec![0 as u32; (dim.0 * dim.1) as usize];
        let mut opt = minifb::WindowOptions::default();
        opt.scale = scale;
        opt.resize = resize;
        opt.scale_mode = ScaleMode::AspectRatioStretch;

        let mut window = minifb::Window::new(title, dim.0, dim.1, opt).unwrap_or_else(|e| {
            panic!("{}", e);
        });
        window.limit_update_rate(match max_fps {
            Some(fps) => Some(Duration::from_millis((1000.0 / fps) as u64)),
            _ => None,
        });
        BufferWindow {
            window,
            buffer_u8,
            buffer_u32,
            dim,
            fps_skip: UpdateSkip::from(
                fps_skip.and_then(|fps| Some(Duration::from_millis((1000.0 / fps) as u64))),
            ),
        }
    }

    /// Returns the underlying `minifb::Window`.
    pub fn window(&mut self) -> &mut minifb::Window {
        &mut self.window
    }

    /// Returns the unscaled size of the window in pixels.
    pub fn size(&self) -> (usize, usize) {
        self.dim
    }

    /// Sets the position of the window's upper left corner in screen pixels.
    /// Preferably use method `with_position()` in [WindowBuilder](struct.WindowBuilder.html).
    pub fn set_position(&mut self, pos: (isize, isize)) {
        self.window.set_position(pos.0, pos.1);
    }

    /// Draws the window's content given a drawing closure.
    /// ```
    ///# use easy_graph::ui::window::WindowBuilder;
    ///# use easy_graph::ui::drawing::IntoDrawingArea;
    ///# use easy_graph::ui::element::Circle;
    ///# use easy_graph::color::style::{BLACK, WHITE};
    /// let mut win = WindowBuilder::new().build();
    /// win.draw(|b| {
    ///     let root = b.into_drawing_area();
    ///     root.fill(&WHITE);
    ///     root.draw(&Circle::new((50, 50), 10, &BLACK)).unwrap();
    /// });
    /// ```
    pub fn draw<F>(&mut self, draw: F)
    where
        F: FnOnce(BitMapBackend<RGBPixel>) -> (),
    {
        if self.window.is_open() && self.fps_skip.update() {
            {
                let b = BitMapBackend::with_buffer(
                    &mut self.buffer_u8,
                    (self.dim.0 as u32, self.dim.1 as u32),
                );
                draw(b);
            }
            self.transfer_buffer();
            self.window
                .update_with_buffer(&self.buffer_u32[..], self.dim.0, self.dim.1)
                .unwrap();
        }
    }
    /// Returns if the window is open.
    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    /// Saves the current buffer to a file at the path specified.
    /// The image format is derived from the file extension. Currently, only jpeg, png, ico, pnm, bmp and tiff files are supported.
    pub fn save_buffer(&self, path: &str) -> Result<(), image::ImageError> {
        image::save_buffer(
            path,
            &self.buffer_u8,
            self.dim.0 as u32,
            self.dim.1 as u32,
            image::ColorType::Rgb8,
        )
    }

    fn transfer_buffer(&mut self) {
        for (inp, outp) in self.buffer_u8.chunks(3).zip(&mut self.buffer_u32) {
            *outp = Self::from_u8arr_rgb(inp);
        }
    }
    fn from_u8arr_rgb(rgb: &[u8]) -> u32 {
        let (r, g, b) = (rgb[0] as u32, rgb[1] as u32, rgb[2] as u32);
        (r << 16) | (g << 8) | b
    }
    #[allow(dead_code)]
    fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        (r << 16) | (g << 8) | b
    }
}

struct UpdateSkip {
    target_rate: Option<Duration>,
    prev_time: f64,
}

#[allow(dead_code)]
impl UpdateSkip {
    fn new() -> UpdateSkip {
        UpdateSkip {
            // Default limit to 4 ms
            target_rate: Some(Duration::from_millis(4)),
            prev_time: 0.0,
        }
    }
    fn from(dur: Option<Duration>) -> UpdateSkip {
        UpdateSkip {
            target_rate: dur,
            prev_time: 0.0,
        }
    }

    #[inline]
    fn set_rate(&mut self, rate: Option<Duration>) {
        self.target_rate = rate
    }

    fn update(&mut self) -> bool {
        match self.target_rate {
            Some(rate) => {
                let target_rate = rate.as_secs_f64();
                let current_time = Self::time_now();
                let delta = current_time - self.prev_time;
                if delta >= target_rate {
                    self.prev_time = current_time;
                    true
                } else {
                    false
                }
            }
            None => true,
        }
    }

    fn time_now() -> f64 {
        (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH))
            .expect("System clock was before 1970.")
            .as_secs_f64()
    }
}

//#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use crate::ui::window::BufferWindow;
    use plotters::prelude::*;

    #[test]
    fn buffer_test() {
        let mut win = BufferWindow::new("Test", (100, 100), None, None, minifb::Scale::X1, true);
        for _i in 0..100 {
            win.draw(|b| {
                let root = b.into_drawing_area();
                root.fill(&WHITE).unwrap();
                root.draw(&Circle::new((50, 50), 15, &RED)).unwrap();
            });
        }
    }
}
