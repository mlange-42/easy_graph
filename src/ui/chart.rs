//!
//! Provides a window for easily plotting line and scatter charts.
//!
//! Internally uses [`BufferWindow`](../window/struct.BufferWindow.html).
//!
//! # Example
//! ```
//! use easy_graph::ui::chart::{ChartBuilder, Series};
//! use easy_graph::color::style::{RED, BLUE};
//!
//! fn main() {
//!     let mut chart = ChartBuilder::new()
//!         .with_title("Test")
//!         .with_labels("some x", "some y")
//!         .with_dimensions(600, 400)
//!         .with_data_limit(100)
//!         .with_fps_limit(30.0)
//!         .add_series(Series::point("A", &RED))
//!         .add_series(Series::line("B", &BLUE))
//!         .build();
//!     
//!     for i in 1..10 { // Increase upper limit for longer run!
//!         let v = i as f64;
//!         chart.push_time_series(v, &[v.log10(), v.sqrt()]);
//!         chart.update();
//!     }
//! }
//! ```
//!

use crate::ui::window::BufferWindow;
use minifb::Scale;
use plotters::prelude::*;
use std::collections::VecDeque;

///
/// Builder for [`Chart`](struct.Chart.html). See [`chart`](index.html) module docs for an example.
///
pub struct ChartBuilder {
    title: String,
    dim: (usize, usize),
    position: Option<(isize, isize)>,
    data: Vec<Series>,
    data_limit: Option<usize>,
    x_label: String,
    y_label: String,
    x_scale: f64,
    y_scale: f64,
    y_log: bool,
    limits: AxisLimits,
    max_fps: Option<f64>,
    fps_skip: Option<f64>,
}

impl ChartBuilder {
    /// Creates a default chart builder.
    pub fn new() -> Self {
        ChartBuilder {
            title: "Plot".to_string(),
            dim: (600, 400),
            position: None,
            data: Vec::new(),
            data_limit: None,
            x_label: "X".to_string(),
            y_label: "Y".to_string(),
            x_scale: 1.0,
            y_scale: 1.0,
            y_log: false,
            limits: AxisLimits::empty(),
            max_fps: None,
            fps_skip: None,
        }
    }
    /// Adds a [Series](struct.Series.html) to the chart.
    pub fn add_series(mut self, series: Series) -> Self {
        self.data.push(series);
        self
    }
    /// Sets the chart's title.
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }
    /// Sets the chart's x axis label.
    pub fn with_x_label(mut self, x_label: &str) -> Self {
        self.x_label = x_label.to_string();
        self
    }
    /// Sets the chart's y axis label.
    pub fn with_y_label(mut self, y_label: &str) -> Self {
        self.y_label = y_label.to_string();
        self
    }
    /// Sets the chart's x and y axis label.
    pub fn with_labels(mut self, x_label: &str, y_label: &str) -> Self {
        self.x_label = x_label.to_string();
        self.y_label = y_label.to_string();
        self
    }
    /// Sets the chart's x axis limits. Use `None` for automatic limit(s).
    ///
    /// E.g., for a fixed lower, but automatic upper limit, use:
    /// ```
    ///# use easy_graph::ui::chart::ChartBuilder;
    /// let mut chart = ChartBuilder::new().with_xlim(Some(0.0), None).build();
    /// ```
    /// Limits apply to the unscaled data.
    pub fn with_xlim(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.limits.x_min = min;
        self.limits.x_max = max;
        self
    }
    /// Sets the chart's y axis limits. Use `None` for automatic limit(s).
    ///
    /// E.g., for a fixed lower, but automatic upper limit, use:
    /// ```
    ///# use easy_graph::ui::chart::ChartBuilder;
    /// let mut chart = ChartBuilder::new().with_ylim(Some(0.0), None).build();
    /// ```
    /// Limits apply to the unscaled data.
    pub fn with_ylim(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        self.limits.y_min = min;
        self.limits.y_max = max;
        self
    }
    /// Sets the chart's x axis scale.
    ///
    /// Data is multiplied by this factor before plotting.
    /// However, axis limits apply to the unscaled data.
    pub fn with_x_scale(mut self, x_scale: f64) -> Self {
        self.x_scale = x_scale;
        self
    }
    /// Sets the chart's y axis to logarithmic.
    pub fn with_y_log(mut self) -> Self {
        self.y_log = true;
        self
    }
    /// Sets the chart's y axis scale.
    ///
    /// Data is multiplied by this factor before plotting.
    /// However, axis limits apply to the unscaled data.
    pub fn with_y_scale(mut self, y_scale: f64) -> Self {
        self.y_scale = y_scale;
        self
    }
    /// Sets the chart's FPS limit. Slows down the process updating the chart.
    ///
    /// The chart's update() method will block to achieve the FPS limit.
    pub fn with_fps_limit(mut self, max_fps: f64) -> Self {
        self.max_fps = Some(max_fps);
        self
    }
    /// Sets the chart's FPS skip. Skips updates, but does not slow down the process updating the chart.
    ///
    /// The chart's update() method will skip frames to achieve the FPS limit.
    pub fn with_fps_skip(mut self, max_fps: f64) -> Self {
        self.fps_skip = Some(max_fps);
        self
    }
    /// Sets the chart's data limit.
    /// For each series, when the given number of enties is exceeded, entries are dropped from the front of the series.
    ///
    /// Use to achieve time series plots with a limited time window.
    pub fn with_data_limit(mut self, max_values: usize) -> Self {
        self.data_limit = Some(max_values);
        self
    }
    /// Sets the dimensions of the chart in screen pixels.
    pub fn with_dimensions(mut self, width: usize, height: usize) -> Self {
        self.dim = (width, height);
        self
    }
    /// Sets the position of the chart's upper left corner in screen pixels.
    pub fn with_position(mut self, x: isize, y: isize) -> Self {
        self.position = Some((x, y));
        self
    }
    /// Builds the chart.
    pub fn build(self) -> Chart {
        let mut win = Chart::new(
            &self.title,
            self.dim,
            self.data,
            self.max_fps,
            self.fps_skip,
        );
        win.x_scale = self.x_scale;
        win.y_scale = self.y_scale;
        win.y_log = self.y_log;
        win.x_label = self.x_label;
        win.y_label = self.y_label;
        win.data_limit = self.data_limit;
        win.limits = self.limits;

        if let Some(pos) = self.position {
            win.window.set_position(pos);
        }
        win
    }
}

/// The type of [`Series`](struct.Series.html) for [`Chart`](struct.Chart.html)s, like Point or Line. Different types can be mixed in the same chart.
pub enum SeriesType {
    Point,
    Line,
}

///
/// A data series for [`Chart`](struct.Chart.html).
/// Also contains information about the series' name and stype.
///
#[allow(dead_code)]
pub struct Series {
    name: String,
    color: RGBColor,
    series_type: SeriesType,
    data: VecDeque<(f64, f64)>,
}
impl Series {
    fn new<T: Color>(name: &str, color: &T, series_type: SeriesType) -> Self {
        let (r, g, b) = color.rgb();
        Series {
            name: name.to_string(),
            color: RGBColor(r, g, b),
            series_type,
            data: VecDeque::new(),
        }
    }
    /// Creates an empty point series.
    pub fn point(name: &str, color: &RGBColor) -> Self {
        Self::new(name, color, SeriesType::Point)
    }

    /// Creates an empty line series.
    pub fn line(name: &str, color: &RGBColor) -> Self {
        Self::new(name, color, SeriesType::Line)
    }

    /// Pushes an xy entry to the back (end) of the series.
    /// Preferably use [`Chart`'s](struct.Chart.html) methods to add or change data.
    pub fn push(&mut self, xy: (f64, f64)) {
        self.data.push_back(xy);
    }
    /// Drops entries from the front of the series until the series has `targ_len` entries.
    pub fn drop_front(&mut self, targ_len: usize) {
        let mut drop = self.data.len() as i32 - targ_len as i32;
        while drop > 0 {
            let _ = self.data.pop_front();
            drop -= 1;
        }
    }
    /// Drops entries from the back (end) of the series until the series has `targ_len` entries.
    pub fn drop_back(&mut self, targ_len: usize) {
        let mut drop = self.data.len() - targ_len;
        while drop > 0 {
            let _ = self.data.pop_back();
            drop -= 1;
        }
    }
    /// Clears the data of the series. Name and style are not affected.
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

struct AxisLimits {
    x_min: Option<f64>,
    x_max: Option<f64>,
    y_min: Option<f64>,
    y_max: Option<f64>,
}
impl AxisLimits {
    pub fn empty() -> Self {
        AxisLimits {
            x_min: None,
            x_max: None,
            y_min: None,
            y_max: None,
        }
    }
}

///
/// A window for easy plotting. Construct using [`ChartBuilder`](struct.ChartBuilder.html).
///
/// See [`chart`](index.html) module docs for an example.
///
#[allow(dead_code)]
pub struct Chart {
    window: BufferWindow,
    data: Vec<Series>,
    data_limit: Option<usize>,
    x_label: String,
    y_label: String,
    x_scale: f64,
    y_scale: f64,
    y_log: bool,
    limits: AxisLimits,
}

impl Chart {
    fn new(
        title: &str,
        dim: (usize, usize),
        series: Vec<Series>,
        max_fps: Option<f64>,
        fps_skip: Option<f64>,
    ) -> Self {
        let window = BufferWindow::new(title, dim, max_fps, fps_skip, Scale::X1, true);

        Chart {
            window,
            data: series,
            data_limit: None,
            x_label: "X".to_string(),
            y_label: "Y".to_string(),
            x_scale: 1.0,
            y_scale: 1.0,
            y_log: false,
            limits: AxisLimits::empty(),
        }
    }

    /// Returns if the chart's window is open.
    pub fn is_open(&self) -> bool {
        self.window.is_open()
    }

    /// Returns the underlying [`BufferWindow`](../window/struct.BufferWindow.html).
    pub fn window(&mut self) -> &mut BufferWindow {
        &mut self.window
    }

    /// Returns the number of series in the chart.
    pub fn num_series(&self) -> usize {
        self.data.len()
    }
    /// Pushes a data row to the chart as a time series entry.
    ///
    /// # Arguments
    /// * `t` - Time or x value for all y values.
    /// * `y` - Slice of y values, one per series.
    ///
    /// # Panics
    /// Panics if the length of `y` does not equal the number of series in the chart.
    pub fn push_time_series(&mut self, t: f64, y: &[f64]) {
        if !self.window.is_open() {
            return;
        }
        if self.data.len() != y.len() {
            panic!("Length of y must be equaltu number of series!");
        }
        for (ser, value) in self.data.iter_mut().zip(y) {
            ser.push((t, *value));
            if let Some(lim) = self.data_limit {
                ser.drop_front(lim);
            }
        }
    }

    /// Pushes an xy entry to a certain series.
    ///
    /// # Arguments
    /// * `index` - Index of the series to push to.
    /// * `xy` - Data point as a tuple of (x, y).
    ///
    /// # Panics
    /// Panics if the index is not in the range of series indices.
    pub fn push_xy(&mut self, index: usize, xy: (f64, f64)) {
        let ser = &mut self.data[index];
        ser.push(xy);
        if let Some(lim) = self.data_limit {
            ser.drop_front(lim);
        }
    }

    /// Replaces the data of a certain series.
    ///
    /// # Arguments
    /// * `index` - Index of the series to replace.
    /// * `data` - Slice of tuples of (x, y).
    ///
    /// # Panics
    /// Panics if the index is not in the range of series indices.
    pub fn replace_series(&mut self, index: usize, data: &[(f64, f64)]) {
        let ser = &mut self.data[index];
        ser.clear();
        for xy in data {
            ser.push(*xy);
        }
    }

    /// Render the graph
    pub fn update(&mut self) {
        let data = &self.data;
        let x_label = &self.x_label;
        let y_label = &self.y_label;
        let x_scale = self.x_scale;
        let y_scale = self.y_scale;
        let y_log = self.y_log;
        let (xlim, ylim) = self.calc_axis_ranges();
        self.window.draw(|b| {
            let root = b.into_drawing_area();
            root.fill(&WHITE).unwrap();
            if y_log {
                let mut cc: ChartContext<_, _> = plotters::chart::ChartBuilder::on(&root)
                    .margin(10)
                    .x_label_area_size(40)
                    .y_label_area_size(60)
                    .build_ranged(
                        (xlim.0 * x_scale)..(xlim.1 * x_scale),
                        LogRange((ylim.0 * y_scale)..(ylim.1 * y_scale)),
                    )
                    .unwrap();

                cc.configure_mesh()
                    .x_label_formatter(&|x| format!("{}", *x))
                    .y_label_formatter(&|y| format!("{}", *y))
                    .x_labels(15)
                    .y_labels(8)
                    .x_desc(x_label)
                    .y_desc(y_label)
                    .axis_desc_style(("sans-serif", 15).into_font())
                    .draw()
                    .unwrap();

                for (_, series) in (0..).zip(data.iter()) {
                    let draw = match &series.series_type {
                        SeriesType::Line => cc.draw_series(LineSeries::new(
                            series.data.iter().map(|(a, b)| {
                                (
                                    *a * x_scale,
                                    if y_log && *b <= 0.0 {
                                        std::f64::NAN
                                    } else {
                                        *b * y_scale
                                    },
                                )
                            }),
                            ShapeStyle::from(&series.color),
                        )),
                        SeriesType::Point => cc.draw_series(series.data.iter().map(|(a, b)| {
                            Circle::new(
                                (*a * x_scale, *b * y_scale),
                                2,
                                ShapeStyle::from(&series.color).filled(),
                            )
                        })),
                    };
                    draw.unwrap().label(&series.name).legend(move |(x, y)| {
                        Rectangle::new(
                            [(x - 5, y - 5), (x + 5, y + 5)],
                            ShapeStyle::from(&series.color).filled(),
                        )
                    });
                }

                cc.configure_series_labels()
                    .background_style(&WHITE.mix(0.8))
                    .border_style(&BLACK)
                    .draw()
                    .unwrap();
            } else {
                let mut cc: ChartContext<_, _> = plotters::chart::ChartBuilder::on(&root)
                    .margin(10)
                    .x_label_area_size(40)
                    .y_label_area_size(60)
                    .build_ranged(
                        (xlim.0 * x_scale)..(xlim.1 * x_scale),
                        (ylim.0 * y_scale)..(ylim.1 * y_scale),
                    )
                    .unwrap();

                cc.configure_mesh()
                    .x_label_formatter(&|x| format!("{}", *x))
                    .y_label_formatter(&|y| format!("{}", *y))
                    .x_labels(15)
                    .y_labels(8)
                    .x_desc(x_label)
                    .y_desc(y_label)
                    .axis_desc_style(("sans-serif", 15).into_font())
                    .draw()
                    .unwrap();

                for (_, series) in (0..).zip(data.iter()) {
                    let draw = match &series.series_type {
                        SeriesType::Line => cc.draw_series(LineSeries::new(
                            series.data.iter().map(|(a, b)| {
                                (
                                    *a * x_scale,
                                    if y_log && *b <= 0.0 {
                                        std::f64::NAN
                                    } else {
                                        *b * y_scale
                                    },
                                )
                            }),
                            ShapeStyle::from(&series.color),
                        )),
                        SeriesType::Point => cc.draw_series(series.data.iter().map(|(a, b)| {
                            Circle::new(
                                (*a * x_scale, *b * y_scale),
                                2,
                                ShapeStyle::from(&series.color).filled(),
                            )
                        })),
                    };
                    draw.unwrap().label(&series.name).legend(move |(x, y)| {
                        Rectangle::new(
                            [(x - 5, y - 5), (x + 5, y + 5)],
                            ShapeStyle::from(&series.color).filled(),
                        )
                    });
                }

                cc.configure_series_labels()
                    .background_style(&WHITE.mix(0.8))
                    .border_style(&BLACK)
                    .draw()
                    .unwrap();
            }
        });
    }

    fn calc_axis_ranges(&self) -> ((f64, f64), (f64, f64)) {
        (self.calc_axis_range(true), self.calc_axis_range(false))
    }
    fn calc_axis_range(&self, is_x: bool) -> (f64, f64) {
        let (min, max) = if is_x {
            (self.limits.x_min, self.limits.x_max)
        } else {
            (self.limits.y_min, self.limits.y_max)
        };
        if min.is_some() && max.is_some() {
            (min.unwrap(), max.unwrap())
        } else {
            let find_min = min.is_none();
            let find_max = max.is_none();
            let mut v_min = std::f64::MAX;
            let mut v_max = std::f64::MIN;
            for ser in &self.data {
                for xy in &ser.data {
                    let v = if is_x { xy.0 } else { xy.1 };
                    if find_min && v < v_min {
                        v_min = v;
                    }
                    if find_max && v > v_max {
                        v_max = v;
                    }
                }
            }
            (min.unwrap_or(v_min), max.unwrap_or(v_max))
        }
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use crate::ui::chart::{Chart, ChartBuilder, Series};
    use plotters::style::{BLUE, GREEN, RED};
    use rand::Rng;

    #[test]
    fn time_series_plot() {
        let mut chart = ChartBuilder::new()
            .with_title("Test")
            .with_labels("some x", "some y")
            .with_dimensions(800, 400)
            .with_data_limit(100)
            .with_fps_limit(30.0)
            .add_series(Series::point("A", &RED))
            .add_series(Series::line("A", &BLUE))
            .build();

        for i in 1..5 {
            chart.push_time_series(i as f64, &[(i as f64).log10(), (i as f64).sqrt()]);
            chart.update();
        }
    }

    #[test]
    fn scatter_plot() {
        let mut rng = rand::thread_rng();
        let mut chart = ChartBuilder::new()
            .with_title("Scatter")
            .with_labels("some x", "some y")
            .with_dimensions(400, 400)
            .with_xlim(Some(0.0), Some(1.0))
            .with_ylim(Some(0.0), Some(1.0))
            .with_fps_limit(30.0)
            .add_series(Series::point("A", &RED))
            .build();

        for _i in 1..5 {
            let points: Vec<_> = (0..100)
                .map(|_| (rng.gen_range(0.0, 1.0), rng.gen_range(0.0, 1.0)))
                .collect();
            chart.replace_series(0, &points);
            chart.update();
        }
    }
}
