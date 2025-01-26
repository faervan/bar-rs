use iced::platform_specific::shell::commands::layer_surface::Anchor;

#[derive(Debug, Default, Clone, Copy)]
pub enum BarAnchor {
    Left,
    Right,
    #[default]
    Top,
    Bottom,
}

impl BarAnchor {
    pub fn vertical(&self) -> bool {
        match self {
            BarAnchor::Top | BarAnchor::Bottom => false,
            BarAnchor::Left | BarAnchor::Right => true,
        }
    }
}

impl From<BarAnchor> for String {
    fn from(anchor: BarAnchor) -> String {
        match anchor {
            BarAnchor::Top => "top",
            BarAnchor::Bottom => "bottom",
            BarAnchor::Left => "left",
            BarAnchor::Right => "right",
        }
        .to_string()
    }
}

impl From<&BarAnchor> for Anchor {
    fn from(anchor: &BarAnchor) -> Self {
        match anchor {
            BarAnchor::Top => Anchor::TOP,
            BarAnchor::Bottom => Anchor::BOTTOM,
            BarAnchor::Left => Anchor::LEFT,
            BarAnchor::Right => Anchor::RIGHT,
        }
    }
}
