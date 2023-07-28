use crate::config::Float;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorRgb {
    pub r: Float,
    pub g: Float,
    pub b: Float
}

impl ColorRgb {
    pub fn to_rgb(self) -> Self {
        self
    }
}

impl From<(Float, Float, Float)> for ColorRgb {
    fn from(t: (Float, Float, Float)) -> Self {
        ColorRgb{r: t.0, g: t.1, b: t.2}
    }
}