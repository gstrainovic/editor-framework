# editor-framework — Design Document

**Datum:** 2026-03-08
**Status:** Approved

## Ziel

Ein GPU-beschleunigtes Editor-Framework das vollständig über Lua-Plugins steuerbar ist. Kein eingebauter Editor — alles ist Plugin.

## Architektur

```
editor-framework/
  crates/
    ef-core/        ← App-Bootstrap, Plugin-Loading, Event-Loop
    ef-lua/         ← mlua Bindings, Lua-API Definitionen
    ef-widgets/     ← gpui-component Wrapper für Lua
    ef-workspace/   ← Fenster, Panels, Tabs, Layout
    ef-pm/          ← Package-Manager (CLI + Lua API)

  plugins/          ← Beispiel-Plugins
    welcome/        ← Welcome-Screen Plugin
```

## Plugin-System

```lua
-- ~/.ef/init.lua
local ef = require("ef")

ef.setup({ theme = "dark" })

ef.use("https://github.com/user/ef-neovim")
ef.use("https://github.com/user/ef-git-graph")
```

```lua
-- Plugin init.lua
local ef = require("ef")

ef.plugin({
    id = "git-graph",
    name = "Git Graph",
    setup = function(opts)
        ef.workspace.add_panel({
            id = "git-graph",
            position = "right",
            render = function(cx)
                cx:text("Git Graph")
                cx:button("Refresh", function() end)
            end
        })
        ef.keymap.set("n", "<leader>gg", function()
            ef.workspace.toggle_panel("git-graph")
        end)
    end
})
```

## Package-Manager

```bash
ef install <git-url>   # Plugin installieren
ef update              # alle Plugins updaten
ef remove <id>         # Plugin entfernen
ef list                # installierte Plugins
```

```
~/.ef/
  plugins/
    ef-neovim/         ← git clone
    ef-git-graph/      ← git clone
  init.lua             ← Nutzer-Config
```

Versionen pinnen:
```lua
ef.use("https://github.com/user/ef-neovim", { tag = "v1.2.0" })
ef.use("https://github.com/user/ef-neovim", { branch = "main" })
```

## Lua ↔ GPUI Bridge

```
Lua Plugin
    ↓  mlua
Rust API (ef-lua)
    ↓  AppContext
GPUI Renderer
    ↓  GPU
Bildschirm
```

- Lua läuft auf dem Main-Thread
- Rust definiert alle Widgets als Lua-Objekte
- GPUI Events → Rust → Lua Callbacks
- Lua Aufrufe → Rust → GPUI Views

## Widgets (via gpui-component)

Alle 60+ gpui-component Widgets via Lua nutzbar:
- Panel, Tabs, Split-Layout
- Button, Input, List, Tree
- Statusbar, Toolbar
- u.v.m.

## Plattformen

- Linux
- macOS
- Windows

## Technologie-Stack

- **Rendering:** GPUI (Zed's GPU-Framework)
- **Widgets:** gpui-component
- **Scripting:** mlua (Lua 5.4)
- **Package-Manager:** git2-rs
- **Sprache:** Rust
