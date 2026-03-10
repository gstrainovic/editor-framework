use anyhow::Result;
use ef_core::{DebugState, Panel};
use gpui::*;
use notify::{EventKind, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

struct WelcomeView {
    debug_state: Arc<Mutex<DebugState>>,
    runtime: &'static ef_lua::LuaRuntime,
    should_reload: Arc<AtomicBool>,
    polling_started: bool,
}

/// Continuously polls the reload flag each frame.
/// When the flag is set, notifies the entity to trigger a re-render.
/// Perpetuates itself via on_next_frame chaining.
fn schedule_poll(window: &mut Window, should_reload: Arc<AtomicBool>, entity_id: EntityId) {
    window.on_next_frame(move |w, cx| {
        if should_reload.load(Ordering::Relaxed) {
            cx.notify(entity_id);
        }
        schedule_poll(w, should_reload, entity_id);
    });
}

impl Render for WelcomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check reload flag - panels.clear() + re-exec Lua + updated UI in this same render pass
        if self.should_reload.swap(false, Ordering::Relaxed) {
            self.debug_state.lock().unwrap().panels.clear();
            let config = dirs::home_dir()
                .unwrap_or_default()
                .join(".ef")
                .join("init.lua");
            if config.exists() {
                if let Ok(code) = std::fs::read_to_string(&config) {
                    if let Err(e) = self.runtime.exec(&code) {
                        log::error!("Reload error: {e}");
                        self.debug_state.lock().unwrap().log.push(format!("Reload error: {e}"));
                    }
                }
            }
        }

        // Start the polling chain once on first render
        if !self.polling_started {
            self.polling_started = true;
            schedule_poll(window, self.should_reload.clone(), cx.entity_id());
        }

        let state = self.debug_state.lock().unwrap();
        let show_debug = state.panel_open;
        let panels = state.panels.clone();
        drop(state);

        div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child("editor-framework")
                    .children(panels.into_iter().map(|panel: Panel| {
                        let content = panel.content.join("\n");
                        let panel_div = div()
                            .absolute()
                            .bg(rgb(0x111111))
                            .p_4()
                            .font_family("monospace")
                            .text_color(rgb(0xffffff))
                            .child(content);

                        match panel.position.as_str() {
                            "center" => panel_div.top_1_2().left_1_2(),
                            "left"   => panel_div.left_0().top_0(),
                            "right"  => panel_div.right_0().top_0(),
                            "top"    => panel_div.top_0().left_0(),
                            "bottom" => panel_div.bottom_0().left_0(),
                            _        => panel_div.top_0().left_0(),
                        }
                    }))
            )
    }
}

fn main() -> Result<()> {
    let runtime = Box::leak(Box::new(ef_lua::LuaRuntime::new()?));
    let debug_state = DebugState::new();

    ef_lua::api::register_with_state(runtime, debug_state.clone())?;

    // Initial load
    debug_state.lock().unwrap().panels.clear();
    let config = dirs::home_dir()
        .unwrap_or_default()
        .join(".ef")
        .join("init.lua");
    if config.exists() {
        if let Ok(code) = std::fs::read_to_string(&config) {
            let _ = runtime.exec(&code);
        }
    }

    let should_reload = Arc::new(AtomicBool::new(false));

    // File watcher thread
    let should_reload_watch = should_reload.clone();
    std::thread::spawn(move || {
        let (wtx, wrx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                    let _ = wtx.send(());
                }
            }
        })
        .expect("watcher");

        let ef_dir = dirs::home_dir().unwrap_or_default().join(".ef");
        if ef_dir.exists() {
            let _ = watcher.watch(&ef_dir, RecursiveMode::Recursive);
        }

        let plugins_dir = std::env::current_exe()
            .unwrap_or_default()
            .parent().unwrap_or(std::path::Path::new("."))
            .parent().unwrap_or(std::path::Path::new("."))
            .parent().unwrap_or(std::path::Path::new("."))
            .join("plugins");
        if plugins_dir.exists() {
            let _ = watcher.watch(&plugins_dir, RecursiveMode::Recursive);
        }

        loop {
            if wrx.recv().is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(150));
                while wrx.try_recv().is_ok() {}
                should_reload_watch.store(true, Ordering::Relaxed);
            }
        }
    });

    let debug_state_gpui = debug_state.clone();

    Application::new().run(move |cx: &mut App| {
        cx.open_window(WindowOptions::default(), move |_window, cx| {
            cx.new(|_cx| WelcomeView {
                debug_state: debug_state_gpui.clone(),
                runtime,
                should_reload: should_reload.clone(),
                polling_started: false,
            })
        })
        .unwrap();
    });

    Ok(())
}
