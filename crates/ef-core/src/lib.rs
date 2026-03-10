use std::sync::{Arc, Mutex};

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

// Shared debug state for UI and Lua
#[derive(Clone)]
pub struct Panel {
    pub id: String,
    pub position: String,
    pub content: Vec<String>,
}

pub struct DebugState {
    pub panel_open: bool,
    pub log: Vec<String>,
    pub panels: Vec<Panel>,
}

impl DebugState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            panel_open: false,
            log: vec!["Debug panel initialized".to_string()],
            panels: vec![],
        }))
    }
}
