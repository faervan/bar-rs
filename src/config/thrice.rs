#[derive(Debug)]
pub struct Thrice<T> {
    pub left: T,
    pub center: T,
    pub right: T,
}

impl From<f32> for Thrice<f32> {
    fn from(value: f32) -> Self {
        Self {
            left: value,
            center: value,
            right: value,
        }
    }
}

impl From<(f32, f32, f32)> for Thrice<f32> {
    fn from((left, center, right): (f32, f32, f32)) -> Self {
        Self {
            left,
            center,
            right,
        }
    }
}
