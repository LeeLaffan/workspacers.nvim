local M = {}

local rpc = require('workspacers.rpc_handler')
local cmds = require('workspacers.commands')

local function setup_commands()
    vim.api.nvim_create_user_command('WorkspacersAdd',
        function(o) M.WorkspacersAdd(o.args) end, { nargs = 1 })
    vim.api.nvim_create_user_command('WorkspacersJson',
        function(o) M.WorkspacersJson(o.args) end, { nargs = 1 })
    vim.api.nvim_create_user_command('WorkspacersList',
        function(o) cmds.WorkspacersList({ ws_name = o.args }) end, { nargs = 1 })
end

M.WorkspacersList = function(ws_name)
    M.opts.ws_name = ws_name
    cmds.WorkspacersList(M.opts)
end

M.WorkspacersJson = function(ws_name)
    M.opts.ws_name = ws_name
    cmds.WorkspacersJson(M.opts)
end

---@param opts.json_dir string: Json dir to be used(If using custom location)
---@param opts.binary string: Direct path to workspacers-nvim binary(If not in path)
---@param opts.theme string: Telescope theme: `ivy` |` dropdown` | `cursor`
M.setup = function(opts)
    M.opts = opts or {}
    M.opts.theme = opts.theme or 'ivy'
    rpc.setup(M.opts)
    cmds.setup(M.opts)
    setup_commands()
end

return M
