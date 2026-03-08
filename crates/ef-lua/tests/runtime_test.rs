#[test]
fn test_lua_eval() {
    let rt = ef_lua::LuaRuntime::new().unwrap();
    let result: i32 = rt.eval("return 1 + 1").unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_lua_exec() {
    let rt = ef_lua::LuaRuntime::new().unwrap();
    rt.exec("x = 42").unwrap();
    let result: i32 = rt.eval("return x").unwrap();
    assert_eq!(result, 42);
}
