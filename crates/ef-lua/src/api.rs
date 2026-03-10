use crate::LuaRuntime;
use anyhow::Result;
use ef_core::{DebugState, Panel};
use mlua::prelude::*;
use std::sync::{Arc, Mutex};

pub fn register(rt: &LuaRuntime) -> Result<()> {
    // Dummy registration without state
    let lua = rt.lua();
    let ef = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;
    lua.globals()
        .set("ef", ef)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}

pub fn register_with_state(rt: &LuaRuntime, debug_state: Arc<Mutex<DebugState>>) -> Result<()> {
    let lua = rt.lua();
    let ef = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.setup({ theme = "dark" })
    let setup_fn = lua
        .create_function(|_, _opts: LuaValue| Ok(()))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    ef.set("setup", setup_fn)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.use("url")
    let use_fn = lua
        .create_function(|_, url: String| {
            log::info!("plugin queued: {}", url);
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    ef.set("use", use_fn).map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.plugin({ id, name, version, setup = function(opts) ... end })
    let plugin_fn = lua
        .create_function(|lua_ctx, spec: LuaTable| {
            let id: String = spec.get::<String>("id").unwrap_or_default();
            let name: String = spec.get::<String>("name").unwrap_or_else(|_| id.clone());
            log::info!("plugin loaded: {} ({})", name, id);
            // Call setup() with empty opts if present
            if let Ok(setup) = spec.get::<LuaFunction>("setup") {
                let opts = lua_ctx.create_table()?;
                setup.call::<()>(opts)?;
            }
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    ef.set("plugin", plugin_fn)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.workspace
    let workspace = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;

    let state_clone = debug_state.clone();
    let add_panel_fn = lua.create_function(move |lua_ctx, opts: LuaTable| {
        let id: String = opts.get("id")?;
        let pos: String = opts
            .get::<String>("position")
            .unwrap_or_else(|_| "right".to_string());

        // Execute the render callback with a context object that collects cx:text() calls
        let content: Vec<String> = if let Ok(render) = opts.get::<LuaFunction>("render") {
            let cx = lua_ctx.create_table()?;
            let lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
            let lines_clone = lines.clone();
            let text_fn = lua_ctx.create_function(move |_, args: LuaMultiValue| {
                // cx:text("foo") passes cx as first arg, string as second
                let s = args.iter()
                    .find_map(|v| if let LuaValue::String(s) = v { s.to_str().ok().map(|x| x.to_string()) } else { None })
                    .unwrap_or_default();
                lines_clone.lock().unwrap().push(s);
                Ok(())
            })?;
            cx.set("text", text_fn)?;
            render.call::<()>(cx)?;
            let collected = lines.lock().unwrap().clone();
            collected
        } else {
            vec!["Panel: ".to_string() + &id]
        };

        let mut state = state_clone.lock().unwrap();
        state.panels.push(Panel {
            id: id.clone(),
            position: pos.clone(),
            content,
        });
        log::info!("panel registered: {} @ {}", id, pos);
        Ok(())
    });
    let add_panel_fn = add_panel_fn.map_err(|e| anyhow::anyhow!("{e}"))?;

    workspace
        .set("add_panel", add_panel_fn)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let toggle_fn = lua
        .create_function(|_, id: String| {
            log::info!("toggle panel: {}", id);
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    workspace
        .set("toggle_panel", toggle_fn)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    ef.set("workspace", workspace)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.keymap
    let keymap = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;

    let keymap_set_fn = lua
        .create_function(|_, (_mode, _key, _cb): (String, String, LuaFunction)| Ok(()))
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    keymap
        .set("set", keymap_set_fn)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    ef.set("keymap", keymap)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.debug
    let debug = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;

    // Screenshot Funktion (placeholder)
    let screenshot_fn = lua
        .create_function(|_, path: String| {
            log::info!("Screenshot requested: {}", path);
            // Hier später Window::screenshot() Aufruf einbauen
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    debug
        .set("screenshot", screenshot_fn)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Panel öffnen
    let state_clone = debug_state.clone();
    let open_panel_fn = lua
        .create_function(move |_, _opts: LuaValue| {
            let mut state = state_clone.lock().unwrap();
            state.panel_open = true;
            state.log.push("Panel opened via Lua".to_string());
            log::info!("Debug panel requested");
            Ok(())
        })
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    debug
        .set("open_panel", open_panel_fn)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    ef.set("debug", debug).map_err(|e| anyhow::anyhow!("{e}"))?;

    lua.globals()
        .set("ef", ef)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}
