use anyhow::Result;
use ef_core::{DebugState, Panel};
use gpui::*;
use notify::{EventKind, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

actions!(ef, [Quit]);

struct WelcomeView {
    debug_state: Arc<Mutex<DebugState>>,
    runtime: &'static ef_lua::LuaRuntime,
    should_reload: Arc<AtomicBool>,
    polling_started: bool,
}

/// Continuously polls the reload flag each frame.
fn schedule_poll(window: &mut Window, should_reload: Arc<AtomicBool>, entity_id: EntityId) {
    window.on_next_frame(move |w, cx| {
        if should_reload.load(Ordering::Relaxed) {
            cx.notify(entity_id);
        }
        schedule_poll(w, should_reload, entity_id);
    });
}

/// Client-side titlebar with drag + window controls
fn render_titlebar(window: &mut Window) -> impl IntoElement {
    let is_maximized = window.is_maximized();

    div()
        .id("titlebar")
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w_full()
        .h(px(32.0))
        .bg(rgb(0x1a1a2e))
        .border_b_1()
        .border_color(rgb(0x333355))
        .window_control_area(WindowControlArea::Drag)
        .child(
            div()
                .pl_3()
                .text_size(px(13.0))
                .text_color(rgb(0x888899))
                .child("editor-framework")
        )
        .child(
            div()
                .id("window-controls")
                .flex()
                .flex_row()
                .items_center()
                .gap_0()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                // Minimize
                .child(
                    div()
                        .id("minimize")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(46.0))
                        .h(px(32.0))
                        .text_size(px(16.0))
                        .text_color(rgb(0x888899))
                        .hover(|s| s.bg(rgb(0x333355)))
                        .child("\u{2014}")
                        .on_click(|_, window, _cx| {
                            window.minimize_window();
                        })
                )
                // Maximize/Restore
                .child(
                    div()
                        .id("maximize")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(46.0))
                        .h(px(32.0))
                        .text_size(px(12.0))
                        .text_color(rgb(0x888899))
                        .hover(|s| s.bg(rgb(0x333355)))
                        .child(if is_maximized { "\u{2750}" } else { "\u{25A1}" })
                        .on_click(|_, window, _cx| {
                            window.zoom_window();
                        })
                )
                // Close
                .child(
                    div()
                        .id("close")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(46.0))
                        .h(px(32.0))
                        .text_size(px(14.0))
                        .text_color(rgb(0x888899))
                        .hover(|s| s.bg(rgb(0xcc3333)).text_color(rgb(0xffffff)))
                        .child("\u{2715}")
                        .on_click(|_, _, cx| {
                            cx.dispatch_action(&Quit);
                        })
                )
        )
}

impl Render for WelcomeView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check reload flag
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
        let panels = state.panels.clone();
        drop(state);

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0x0f0f1a))
            // Client-side titlebar
            .child(render_titlebar(window))
            // Content area
            .child(
                div()
                    .flex()
                    .flex_1()
                    .items_center()
                    .justify_center()
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
        cx.on_action(|_action: &Quit, cx| cx.quit());

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: size(px(1024.0), px(768.0)),
                })),
                titlebar: None,
                window_decorations: Some(WindowDecorations::Client),
                ..Default::default()
            },
            move |_window, cx| {
                cx.new(|_cx| WelcomeView {
                    debug_state: debug_state_gpui.clone(),
                    runtime,
                    should_reload: should_reload.clone(),
                    polling_started: false,
                })
            },
        )
        .unwrap();
    });

    Ok(())
}
