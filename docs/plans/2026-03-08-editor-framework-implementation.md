# editor-framework Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Ein GPU-beschleunigtes Editor-Framework mit GPUI und mlua, vollständig über Lua-Plugins steuerbar.

**Architecture:** GPUI übernimmt Rendering und Event-Loop. mlua bettet Lua auf dem Main-Thread ein. Alle Editor-Features (Panels, Tabs, Keybindings) sind Lua-Plugins die über eine definierte Rust-API GPUI steuern.

**Tech Stack:** Rust, GPUI, gpui-component, mlua (Lua 5.4), git2

---

### Task 1: Cargo Workspace aufsetzen

**Files:**
- Create: `Cargo.toml`
- Create: `crates/ef-core/Cargo.toml` + `src/lib.rs`
- Create: `crates/ef-lua/Cargo.toml` + `src/lib.rs`
- Create: `crates/ef-widgets/Cargo.toml` + `src/lib.rs`
- Create: `crates/ef-workspace/Cargo.toml` + `src/lib.rs`
- Create: `crates/ef-pm/Cargo.toml` + `src/lib.rs`
- Create: `crates/ef-app/Cargo.toml` + `src/main.rs`

**Step 1: Workspace Cargo.toml**

```toml
[workspace]
members = [
    "crates/ef-core",
    "crates/ef-lua",
    "crates/ef-widgets",
    "crates/ef-workspace",
    "crates/ef-pm",
    "crates/ef-app",
]
resolver = "2"

[workspace.dependencies]
gpui = { git = "https://github.com/zed-industries/zed", branch = "main" }
mlua = { version = "0.10", features = ["lua54", "vendored"] }
anyhow = "1"
log = "0.4"
```

**Step 2: ef-core — Plugin Trait**

```toml
# crates/ef-core/Cargo.toml
[package]
name = "ef-core"
version = "0.1.0"
edition = "2021"
[dependencies]
gpui = { workspace = true }
anyhow = { workspace = true }
```

```rust
// crates/ef-core/src/lib.rs
pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn init(&self, cx: &mut gpui::AppContext);
}

pub struct App {
    plugins: Vec<Box<dyn Plugin>>,
}

impl App {
    pub fn new() -> Self { Self { plugins: vec![] } }

    pub fn add_plugin(mut self, p: impl Plugin + 'static) -> Self {
        self.plugins.push(Box::new(p));
        self
    }

    pub fn run(self) {
        gpui::App::new().run(move |cx| {
            for p in &self.plugins { p.init(cx); }
        });
    }
}
```

**Step 3: Build verifizieren**

```bash
cargo build
```
Erwartet: kompiliert ohne Fehler

**Step 4: Commit**

```bash
git init && git add . && git commit -m "feat: initial workspace structure"
```

---

### Task 2: ef-lua — Lua Runtime

**Files:**
- Modify: `crates/ef-lua/Cargo.toml`
- Create: `crates/ef-lua/src/runtime.rs`
- Create: `crates/ef-lua/tests/runtime_test.rs`

**Step 1: Failing Test**

```rust
// crates/ef-lua/tests/runtime_test.rs
#[test]
fn test_lua_eval() {
    let rt = ef_lua::LuaRuntime::new().unwrap();
    let result: i32 = rt.eval("return 1 + 1").unwrap();
    assert_eq!(result, 2);
}
```

**Step 2: Fehlschlagen lassen**

```bash
cargo test -p ef-lua
```
Erwartet: FAIL

**Step 3: Implementieren**

```toml
[dependencies]
mlua = { workspace = true }
anyhow = { workspace = true }
```

```rust
// crates/ef-lua/src/runtime.rs
use mlua::prelude::*;
use anyhow::Result;

pub struct LuaRuntime { lua: Lua }

impl LuaRuntime {
    pub fn new() -> Result<Self> {
        Ok(Self { lua: Lua::new() })
    }
    pub fn eval<T: FromLua>(&self, code: &str) -> Result<T> {
        Ok(self.lua.load(code).eval::<T>()?)
    }
    pub fn exec(&self, code: &str) -> Result<()> {
        Ok(self.lua.load(code).exec()?)
    }
    pub fn lua(&self) -> &Lua { &self.lua }
}
```

**Step 4: Tests bestehen**

```bash
cargo test -p ef-lua
```

**Step 5: Commit**

```bash
git add crates/ef-lua/ && git commit -m "feat: lua runtime via mlua"
```

---

### Task 3: ef-lua — `ef` Global API

**Files:**
- Create: `crates/ef-lua/src/api.rs`
- Create: `crates/ef-lua/tests/api_test.rs`

**Step 1: Failing Test**

```rust
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
```

**Step 2: Fehlschlagen**

```bash
cargo test -p ef-lua -- api
```

**Step 3: Implementieren**

```rust
// crates/ef-lua/src/api.rs
use mlua::prelude::*;
use anyhow::Result;
use crate::LuaRuntime;

pub fn register(rt: &LuaRuntime) -> Result<()> {
    let lua = rt.lua();
    let ef = lua.create_table()?;

    ef.set("setup", lua.create_function(|_, _opts: LuaTable| {
        Ok(())
    })?)?;

    ef.set("use", lua.create_function(|_, url: String| {
        log::info!("plugin queued: {}", url);
        Ok(())
    })?)?;

    // ef.workspace
    let workspace = lua.create_table()?;
    workspace.set("add_panel", lua.create_function(|_, opts: LuaTable| {
        let id: String = opts.get("id")?;
        let pos: String = opts.get("position").unwrap_or("right".to_string());
        log::info!("panel: {} @ {}", id, pos);
        Ok(())
    })?)?;
    workspace.set("toggle_panel", lua.create_function(|_, id: String| {
        log::info!("toggle: {}", id);
        Ok(())
    })?)?;
    ef.set("workspace", workspace)?;

    // ef.keymap
    let keymap = lua.create_table()?;
    keymap.set("set", lua.create_function(|_, (_mode, _key, _cb): (String, String, LuaFunction)| {
        Ok(())
    })?)?;
    ef.set("keymap", keymap)?;

    lua.globals().set("ef", ef)?;
    Ok(())
}
```

**Step 4: Tests bestehen**

```bash
cargo test -p ef-lua
```

**Step 5: Commit**

```bash
git add crates/ef-lua/ && git commit -m "feat: ef global lua api"
```

---

### Task 4: ef-pm — Package Manager

**Files:**
- Modify: `crates/ef-pm/Cargo.toml`
- Create: `crates/ef-pm/src/manager.rs`
- Create: `crates/ef-pm/tests/manager_test.rs`

**Step 1: Failing Test**

```rust
#[test]
fn test_plugins_dir_created() {
    let tmp = tempfile::tempdir().unwrap();
    let pm = ef_pm::PluginManager::new(tmp.path().to_path_buf());
    pm.init().unwrap();
    assert!(tmp.path().join("plugins").exists());
}
```

**Step 2: Fehlschlagen**

```bash
cargo test -p ef-pm
```

**Step 3: Implementieren**

```toml
[dependencies]
git2 = "0.19"
anyhow = { workspace = true }
[dev-dependencies]
tempfile = "3"
```

```rust
// crates/ef-pm/src/manager.rs
use std::path::PathBuf;
use anyhow::Result;

pub struct PluginManager { base_dir: PathBuf }

impl PluginManager {
    pub fn new(base_dir: PathBuf) -> Self { Self { base_dir } }

    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(self.base_dir.join("plugins"))?;
        Ok(())
    }

    pub fn install(&self, url: &str) -> Result<PathBuf> {
        let name = url.split('/').last().unwrap_or("plugin");
        let dest = self.base_dir.join("plugins").join(name);
        if !dest.exists() {
            git2::Repository::clone(url, &dest)?;
        }
        Ok(dest)
    }

    pub fn list(&self) -> Result<Vec<String>> {
        Ok(std::fs::read_dir(self.base_dir.join("plugins"))?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect())
    }
}
```

**Step 4: Tests bestehen**

```bash
cargo test -p ef-pm
```

**Step 5: Commit**

```bash
git add crates/ef-pm/ && git commit -m "feat: plugin manager with git install"
```

---

### Task 5: ef-app — GPUI Fenster + init.lua

**Files:**
- Modify: `crates/ef-app/Cargo.toml`
- Modify: `crates/ef-app/src/main.rs`

**Step 1: Implementieren**

```toml
[dependencies]
ef-core = { path = "../ef-core" }
ef-lua = { path = "../ef-lua" }
gpui = { workspace = true }
anyhow = { workspace = true }
dirs = "5"
```

```rust
// crates/ef-app/src/main.rs
use anyhow::Result;

fn main() -> Result<()> {
    let runtime = ef_lua::LuaRuntime::new()?;
    ef_lua::api::register(&runtime)?;

    let config = dirs::home_dir().unwrap().join(".ef").join("init.lua");
    if config.exists() {
        runtime.exec(&std::fs::read_to_string(config)?)?;
    }

    gpui::App::new().run(|cx| {
        cx.open_window(gpui::WindowOptions::default(), |cx| {
            cx.new_view(|_| WelcomeView)
        }).unwrap();
    });

    Ok(())
}

struct WelcomeView;
impl gpui::Render for WelcomeView {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        gpui::div()
            .flex().items_center().justify_center().size_full()
            .child("editor-framework — alles ist ein Plugin")
    }
}
```

**Step 2: Starten**

```bash
cargo run -p ef-app
```
Erwartet: Fenster öffnet sich

**Step 3: Commit**

```bash
git add crates/ef-app/ && git commit -m "feat: gpui window with lua bootstrap"
```

---

### Task 6: Beispiel-Plugin (Welcome)

**Files:**
- Create: `plugins/welcome/init.lua`
- Create: `plugins/welcome/plugin.toml`

**Step 1: Plugin**

```toml
# plugins/welcome/plugin.toml
id = "welcome"
name = "Welcome Screen"
version = "0.1.0"
```

```lua
-- plugins/welcome/init.lua
ef.plugin({
    id = "welcome",
    name = "Welcome Screen",
    setup = function()
        ef.workspace.add_panel({
            id = "welcome",
            position = "center",
            render = function(cx)
                cx:text("Willkommen in editor-framework")
            end
        })
    end
})
```

**Step 2: Commit**

```bash
git add plugins/ && git commit -m "feat: welcome example plugin"
```

---

## Reihenfolge

1. Task 1 — Workspace
2. Task 2 — Lua Runtime  
3. Task 3 — Lua API
4. Task 4 — Package Manager
5. Task 5 — GPUI App
6. Task 6 — Beispiel Plugin
