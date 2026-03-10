use mlua::prelude::*;
use anyhow::Result;
use std::sync::Arc;

pub struct LuaRuntime {
    lua: Arc<mlua::Lua>,
}

impl LuaRuntime {
    pub fn new() -> Result<Self> {
        Ok(Self {
            lua: Arc::new(Lua::new()),
        })
    }

    pub fn eval<T: FromLua>(&self, code: &str) -> Result<T> {
        self.lua.load(code).eval::<T>().map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub fn exec(&self, code: &str) -> Result<()> {
        self.lua.load(code).exec().map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub fn lua(&self) -> Arc<mlua::Lua> {
        self.lua.clone()
    }
}
