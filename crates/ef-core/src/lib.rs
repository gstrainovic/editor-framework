pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;
}

pub struct App {
    plugins: Vec<Box<dyn Plugin>>,
}

impl App {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    pub fn add_plugin(mut self, p: impl Plugin + 'static) -> Self {
        self.plugins.push(Box::new(p));
        self
    }
}
