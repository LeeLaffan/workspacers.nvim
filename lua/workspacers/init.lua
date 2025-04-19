local M = {}

local rpc = require('workspacers.rpc_handler')
local cmds = require('workspacers.commands')

M.setup = function(opts)
    opts = opts or {}
    opts.keys = opts.keys or {
        ["<C-x>"] = cmds.DeleteWorkspace,
        ["<C-e>"] = cmds.EditWorkspace,
        ["<C-u>"] = function(o) cmds.PromoteWorkspace(o, true) end,
        ["<C-d>"] = function(o) cmds.PromoteWorkspace(o, false) end,
        ["<C-a>"] = function()
            cmds.WorkspacersAdd()
            cmds.WorkspacersList()
        end,
    }
    rpc.setup(opts)
    cmds.setup(opts)
    vim.api.nvim_create_user_command('WorkspacersAdd', cmds.WorkspacersAdd, {})
    vim.api.nvim_create_user_command('WorkspacersJson', cmds.WorkspacersJson, {})
    vim.api.nvim_create_user_command('WorkspacersList', cmds.WorkspacersList, {})

    vim.keymap.set('n', '<leader>lw', cmds.WorkspacersList)
    vim.keymap.set('n', '<C-S-w>', cmds.WorkspacersList)
end

return M
