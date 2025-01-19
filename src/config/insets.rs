use iced::{runtime::platform_specific::wayland::layer_surface::IcedMargin, Padding, Radius};

pub struct Insets {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
}

impl Insets {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self { a, b, c, d }
    }
}

impl From<Insets> for IcedMargin {
    fn from(insets: Insets) -> Self {
        Self {
            top: insets.a as i32,
            right: insets.b as i32,
            bottom: insets.c as i32,
            left: insets.d as i32,
        }
    }
}

impl From<Insets> for Padding {
    fn from(insets: Insets) -> Self {
        Self {
            top: insets.a,
            right: insets.b,
            bottom: insets.c,
            left: insets.d,
        }
    }
}

impl From<Insets> for Radius {
    fn from(insets: Insets) -> Self {
        Self {
            top_left: insets.a,
            top_right: insets.b,
            bottom_right: insets.c,
            bottom_left: insets.d,
        }
    }
}
