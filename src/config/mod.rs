#[derive(Debug)]
pub struct Config {
    pub reload_interval: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reload_interval: 3.,
        }
    }
}
