# editor-framework

A minimal Rust framework for building GUI applications with Lua plugins and hot-reload support.

## Features

- **GPUI-based rendering** - Fast, efficient UI framework
- **Lua plugin system** - Extend functionality with Lua scripts
- **Hot-reload** - Edit plugins and see changes without rebuilding
- **File watcher** - Automatically detects changes to `~/.ef/` and `plugins/` directories
- **Async architecture** - Non-blocking plugin loading and rendering

## Quick Start

### Build

```bash
cargo build --release
```

### Run

```bash
./target/release/ef
```

The app loads plugins from `~/.ef/init.lua` on startup. Create a plugin by calling `ef.plugin()` with a setup function.

## Plugin Example

Create `~/.ef/init.lua`:

```lua
ef.plugin({
    id = "my-plugin",
    name = "My Plugin",
    version = "0.1.0",
    setup = function(opts)
        ef.workspace.add_panel({
            id = "my-panel",
            position = "center",
            render = function(cx)
                cx:text("Hello from Lua!")
            end
        })
    end
})
```

## Architecture

### Crates

- **ef-core** - Core data structures and types
- **ef-lua** - Lua runtime and API bindings
- **ef-app** - GPUI window and hot-reload loop
- **ef-pm** - Plugin manager (planning)
- **ef-widgets** - UI widget library (planning)
- **ef-workspace** - Workspace management (planning)

### Hot-Reload Mechanism

1. **File watcher thread** monitors `~/.ef/` and `plugins/` directories
2. When a file changes, watcher sets `AtomicBool` dirty flag
3. **Render loop** polls the flag each frame via `schedule_poll()`
4. When flag is set, calls `cx.notify(entity_id)` to trigger re-render
5. Re-render reloads `~/.ef/init.lua` via `runtime.exec()`
6. Lua `dofile()` loads updated plugins

This avoids expensive full rebuilds—only Lua code re-executes.

## Development

See `docs/` for architecture and design notes.

### Testing

Build and run:

```bash
cargo build --release
./target/release/ef
```

Modify a plugin file to verify hot-reload works.

## License

TBD
