#[test]
fn test_ef_global_exists() {
    let rt = ef_lua::LuaRuntime::new().unwrap();
    ef_lua::api::register(&rt).unwrap();
    let ok: bool = rt.eval("return ef ~= nil").unwrap();
    assert!(ok);
}

#[test]
fn test_ef_setup_callable() {
    let rt = ef_lua::LuaRuntime::new().unwrap();
    ef_lua::api::register(&rt).unwrap();
    rt.exec(r#"ef.setup({ theme = "dark" })"#).unwrap();
}

#[test]
fn test_ef_use_callable() {
    let rt = ef_lua::LuaRuntime::new().unwrap();
    ef_lua::api::register(&rt).unwrap();
    rt.exec(r#"ef["use"]("https://github.com/test/plugin")"#).unwrap();
}

#[test]
fn test_ef_workspace_add_panel() {
    let rt = ef_lua::LuaRuntime::new().unwrap();
    ef_lua::api::register(&rt).unwrap();
    rt.exec(r#"
        ef.workspace.add_panel({
            id = "test-panel",
            position = "right",
            render = function(cx) end
        })
    "#).unwrap();
}

#[test]
fn test_ef_keymap_set() {
    let rt = ef_lua::LuaRuntime::new().unwrap();
    ef_lua::api::register(&rt).unwrap();
    rt.exec(r#"
        ef.keymap.set("n", "<leader>x", function() end)
    "#).unwrap();
}
