use mlua::prelude::*;
use anyhow::Result;
use crate::LuaRuntime;

pub fn register(rt: &LuaRuntime) -> Result<()> {
    let lua = rt.lua();
    let ef = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.setup({ theme = "dark" })
    let setup_fn = lua.create_function(|_, _opts: LuaValue| {
        Ok(())
    }).map_err(|e| anyhow::anyhow!("{e}"))?;
    ef.set("setup", setup_fn).map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.use("url")
    let use_fn = lua.create_function(|_, url: String| {
        log::info!("plugin queued: {}", url);
        Ok(())
    }).map_err(|e| anyhow::anyhow!("{e}"))?;
    ef.set("use", use_fn).map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.workspace
    let workspace = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;

    let add_panel_fn = lua.create_function(|_, opts: LuaTable| {
        let id: String = opts.get("id")?;
        let pos: String = opts.get::<String>("position").unwrap_or_else(|_| "right".to_string());
        log::info!("panel registered: {} @ {}", id, pos);
        Ok(())
    }).map_err(|e| anyhow::anyhow!("{e}"))?;
    workspace.set("add_panel", add_panel_fn).map_err(|e| anyhow::anyhow!("{e}"))?;

    let toggle_fn = lua.create_function(|_, id: String| {
        log::info!("toggle panel: {}", id);
        Ok(())
    }).map_err(|e| anyhow::anyhow!("{e}"))?;
    workspace.set("toggle_panel", toggle_fn).map_err(|e| anyhow::anyhow!("{e}"))?;

    ef.set("workspace", workspace).map_err(|e| anyhow::anyhow!("{e}"))?;

    // ef.keymap
    let keymap = lua.create_table().map_err(|e| anyhow::anyhow!("{e}"))?;

    let keymap_set_fn = lua.create_function(|_, (_mode, _key, _cb): (String, String, LuaFunction)| {
        Ok(())
    }).map_err(|e| anyhow::anyhow!("{e}"))?;
    keymap.set("set", keymap_set_fn).map_err(|e| anyhow::anyhow!("{e}"))?;

    ef.set("keymap", keymap).map_err(|e| anyhow::anyhow!("{e}"))?;

    lua.globals().set("ef", ef).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}
