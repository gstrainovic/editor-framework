-- Welcome Plugin für editor-framework
-- Registriert ein Welcome-Panel beim Start

ef.plugin({
    id = "welcome",
    name = "Welcome Screen",
    version = "0.1.0",
    setup = function(opts)
        opts = opts or {}

        ef.workspace.add_panel({
            id = "welcome",
            position = opts.position or "center",
            render = function(cx)
                cx:text("Willkommen in editor-framework")
                cx:text("")
                cx:text("Plugins installieren:")
                cx:text("  ef install https://github.com/user/ef-neovim")
                cx:text("")
                cx:text("Konfiguration: ~/.ef/init.lua")
            end
        })
    end
})
