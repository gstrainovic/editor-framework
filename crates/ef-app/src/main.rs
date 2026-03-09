use anyhow::Result;
use gpui::*;

struct WelcomeView;

impl Render for WelcomeView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .child("editor-framework")
    }
}

fn main() -> Result<()> {
    // Lua Runtime starten
    let runtime = ef_lua::LuaRuntime::new()?;
    ef_lua::api::register(&runtime)?;

    // ~/.ef/init.lua laden falls vorhanden
    let config = dirs::home_dir()
        .unwrap_or_default()
        .join(".ef")
        .join("init.lua");

    if config.exists() {
        let code = std::fs::read_to_string(&config)?;
        runtime.exec(&code)?;
    }

    // GPUI App starten
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_window, cx| {
            cx.new(|_cx| WelcomeView)
        })
        .unwrap();
    });

    Ok(())
}
