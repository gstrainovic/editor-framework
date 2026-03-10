# Claude Development Guide

This file documents the architecture and key design decisions for editor-framework development with Copilot CLI.

## Project Structure

```
crates/
  ef-core/       # Core types: DebugState, Panel, etc.
  ef-lua/        # Lua runtime and API registration
  ef-app/        # GPUI window, main.rs with hot-reload loop
  ef-pm/         # Plugin manager (stub)
  ef-widgets/    # Widget library (stub)
  ef-workspace/  # Workspace (stub)
plugins/
  welcome/       # Example plugin: register welcome panel
docs/
  plans/         # Architecture and planning docs
```

## Hot-Reload Implementation

**File:** `crates/ef-app/src/main.rs`

### Key Components

1. **File watcher thread** (lines 118-151)
   - Uses `notify` crate with recursive mode
   - Watches `~/.ef/` and `plugins/` directories
   - Sends signal on Modify/Create events
   - 150ms debounce to avoid cascade reloads

2. **Polling callback** (lines 16-22)
   - `schedule_poll()` function creates self-perpetuating frame callback chain
   - Each frame checks `AtomicBool` flag via `load(Ordering::Relaxed)`
   - When set, calls `cx.notify(entity_id)` to wake GPUI renderer
   - Recursively calls itself via `window.on_next_frame()`

3. **Render loop** (lines 28-36 in WelcomeView::render)
   - Checks dirty flag at start of render
   - If set, clears panels and re-executes `~/.ef/init.lua` via `runtime.exec()`
   - Lua code dynamically loads plugins via `dofile()`
   - Updated panel content renders in same frame

### Why This Works

- **GPUI Constraints**: GPUI doesn't expose direct message channels or event loop. `on_next_frame` is the only way to schedule work.
- **Thread Safety**: `AtomicBool` is Send/Sync, avoids mlua's non-Send Lua runtime
- **Efficiency**: File watcher runs in background thread. Polling is cheap (single atomic load per frame).
- **Responsiveness**: Changes detected within 150ms + 1 frame = ~20ms total latency on 60fps display

## Lua API

**File:** `crates/ef-lua/src/api.rs`

### Registered Functions

- `ef.plugin(spec)` - Register a plugin with `{id, name, version, setup}` table
  - Immediately calls `setup(opts)` function
  - Setup receives empty `opts` table

- `ef.workspace.add_panel(spec)` - Register a UI panel with `{id, position, render}` table
  - `position` can be: "center", "left", "right", "top", "bottom" (default "center")
  - `render` is a closure: `render(cx) -> nil`
  - Inside render, call `cx:text(string)` to add content

- `ef.debug.open_panel()` - Opens debug output (currently no-op after removal)

### Method Calling Syntax

Lua methods use colon syntax:
```lua
cx:text("hello")  -- passes cx as implicit first argument
```

In Rust, this becomes:
```rust
closure(LuaValue::Table(cx), LuaValue::String("hello"))
```

## Testing Hot-Reload

1. Start the app: `./target/release/ef`
2. Edit `~/.ef/init.lua` or change plugin files
3. Within 2 seconds, UI updates without rebuild
4. Check that new text appears in plugin panel

## Common Issues

### Plugin text doesn't appear
- Check `~/.ef/init.lua` exists
- Verify `dofile(plugin_path)` can find the file
- Check `ef.plugin()` is called during init
- Verify `ef.workspace.add_panel()` is called in setup

### Hot-reload doesn't trigger
- Confirm file watcher is running (check for "notify watcher" in code)
- Edit `~/.ef/init.lua` directly (watcher monitors this)
- Wait 150ms for debounce + 1 frame for polling
- Check `AtomicBool` flag is being set (add log statements if needed)

### Build fails with mlua errors
- Ensure `mlua` version matches across all crates
- mlua is NOT Send—don't share Lua runtime across threads
- Use AtomicBool for thread-safe signaling instead

## Future Work

- [ ] Plugin manager (load/unload plugins dynamically)
- [ ] Settings persistence
- [ ] More UI widgets
- [ ] Error handling improvements
- [ ] Plugin versioning and dependencies
- [ ] Test framework for plugins
