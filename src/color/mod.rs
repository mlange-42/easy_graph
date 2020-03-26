use crate::color::style::{RGBColor, SimpleColor};
#[doc(no_inline)]
pub use plotters::style;

pub trait ColorMap {
    fn get_color_norm(&self, value: f64) -> RGBColor;
    fn get_color(&self, min: f64, max: f64, value: f64) -> RGBColor {
        let range = max - min;
        self.get_color_norm((value - min) / range)
    }

    fn lerp(lower: u8, upper: u8, frac: f64) -> u8 {
        (lower as f64 + frac * (upper as i16 - lower as i16) as f64).round() as u8
    }
    fn lerp_rgb(lower: (u8, u8, u8), upper: (u8, u8, u8), frac: f64) -> (u8, u8, u8) {
        (
            Self::lerp(lower.0, upper.0, frac),
            Self::lerp(lower.1, upper.1, frac),
            Self::lerp(lower.2, upper.2, frac),
        )
    }
    fn lerp_colors(lower: (u8, u8, u8), upper: (u8, u8, u8), frac: f64) -> RGBColor {
        RGBColor(
            Self::lerp(lower.0, upper.0, frac),
            Self::lerp(lower.1, upper.1, frac),
            Self::lerp(lower.2, upper.2, frac),
        )
    }
}

pub struct LinearColorMap {
    colors: Vec<(u8, u8, u8)>,
}
impl LinearColorMap {
    pub fn new(colors: &[&RGBColor]) -> Self {
        LinearColorMap {
            colors: colors.iter().map(|c| c.rgb()).collect(),
        }
    }
}
impl ColorMap for LinearColorMap {
    fn get_color_norm(&self, value: f64) -> RGBColor {
        let num_cols = self.colors.len();
        let rel = value * (num_cols - 1) as f64;
        let lower = rel.floor() as usize;
        let frac = rel - lower as f64;
        if frac < 0.001 {
            let (r, g, b) = self.colors[lower];
            return RGBColor(r, g, b);
        }

        let col1 = self.colors[lower];
        let col2 = self.colors[lower + 1];
        Self::lerp_colors(col1, col2, frac)
    }
}

//#[cfg(test)]
#[allow(unused_imports)]
mod test {
    use crate::color::style::{Color, RGBColor, GREEN, RED, YELLOW};
    use crate::color::{ColorMap, LinearColorMap};

    #[test]
    fn color_map_test() {
        let map = LinearColorMap::new(&[&GREEN, &YELLOW, &RED]);

        assert_eq!(map.get_color_norm(0.0).rgb(), (0, 255, 0));
        assert_eq!(map.get_color_norm(0.5).rgb(), (255, 255, 0));
        assert_eq!(map.get_color_norm(1.0).rgb(), (255, 0, 0));

        assert_eq!(map.get_color_norm(0.25).rgb(), (128, 255, 0));
    }
}
