local M = {}

local rpc = require('workspacers.rpc_handler')
local tele = require('workspacers.telescope')
local icons = require('utils.icons')

M.setup = function(opts)
    M.opts = opts or {}
    M.opts.keys = opts.keys or {
        ["<C-x>"] = M.DeleteWorkspace,
        ["<C-e>"] = M.EditWorkspace,
        ["<C-u>"] = function(o) M.PromoteWorkspace(o, true) end,
        ["<C-d>"] = function(o) M.PromoteWorkspace(o, false) end,
        ["<C-a>"] = function(o)
            M.WorkspacersAdd(o)
        end,
    }
end

local rpc_names = {
    list = 'WORKSPACERS.LIST',
    list_all = 'WORKSPACERS.LIST_ALL',
    add = 'WORKSPACERS.ADD',
    delete = 'WORKSPACERS.DELETE',
    json = 'WORKSPACERS.JSON',
    promote = 'WORKSPACERS.PROMOTE',
    demote = 'WORKSPACERS.DEMOTE',
    record = 'WORKSPACERS.RECORD',
    replace = 'WORKSPACERS.REPLACE',
}

local function try_get_input(input_opts, allow_blank)
    local success, input = pcall(function() return vim.fn.input(input_opts) end)
    print("input:   " .. input)
    if not success then
        vim.notify("Operation Cancelled", vim.log.levels.INFO)
        return nil, false
    end

    if not allow_blank then
        if input == "" or input == nil then
            vim.notify("Input cannot be blank. <C-c> to exit.", vim.log.levels.WARN)
            return try_get_input(input_opts, allow_blank)
        end
    end

    return input, true
end

local function get_buf_path()
    vim.print(vim.fn.expand('%:p'))
    local path = vim.fn.expand('%:p')
    if path:match("^oil://") then
        return path:gsub("^oil://", "")
    else
        return path
    end
end

M.WorkspacersAdd = function(opts)
    opts.close()
    local name, success = try_get_input({
        prompt = "Enter Workspace Name: "
    })
    if not success then return end

    local path, success = try_get_input({
        prompt = "Enter Workspace Path: ",
        default = get_buf_path(),
        completion = "dir"
    })
    if not success then return end

    local new_ws = {
        name = name,
        path = path
    }

    rpc.req_res(rpc_names.add, function(ws)
        M.WorkspacersList(opts)
    end, opts.ws_name, new_ws)
    M.WorkspacersList(opts)
end

M.EditWorkspace = function(opts)
    rpc.req_res(rpc_names.record, function(ws)
        local name, success = try_get_input({
            prompt = "Enter New Name: ",
            default = ws.Name,
        })
        if not success then return end
        local path, success = try_get_input({
            prompt = "Enter New Path: ",
            default = ws.Path,
            completion = "file"
        })
        if not success then return end
        local rpc_args = {
            Key = opts.selected[1],
            New = {
                Name = name,
                Path = path,
            }
        }

        if opts.selected and opts.selected[1] then
            rpc.req(rpc_names.replace, opts.ws_name, rpc_args)
            opts.close()
            M.WorkspacersList(opts)
        else
            vim.notify("No selected Workspace", vim.log.levels.ERROR)
        end
    end, opts.ws_name, opts.selected[1])
end

M.DeleteWorkspace = function(opts)
    if vim.fn.confirm("Delete Selected Workspace: ", "&Yes\n&No", 2) == 1 then
        if opts.selected and opts.selected[1] then
            rpc.req_res(rpc_names.delete, function()
                opts.close()
                M.WorkspacersList(opts)
            end, opts.ws_name, opts.selected[1])
        else
            vim.notify("No selected Workspace", vim.log.levels.ERROR)
        end
    end
end

local function select_workspace(ws)
    vim.cmd("edit " .. ws.Path)
    vim.cmd("cd " .. ws.Path)
end

M.WorkspacersList = function(opts)
    rpc.req_res(rpc_names.list, function(rpc_obj)
        -- Arrange into lua friendly format
        local fmt_vals = {}
        local ws_by_fmt = {} -- Have a keyed table to lookup in preview
        for _, entry in pairs(rpc_obj) do
            for fmt, ws in pairs(entry) do
                table.insert(fmt_vals, fmt)
                ws_by_fmt[fmt] = ws
            end
        end
        opts.records = fmt_vals
        opts.ws_by_fmt = ws_by_fmt
        opts.callback = function(call_opts)
            call_opts.close()
            if call_opts.selected and call_opts.selected[1] then
                select_workspace(ws_by_fmt[call_opts.selected[1]])
            else
                vim.notify("No selected Workspace", vim.log.levels.ERROR)
            end
        end
        opts.previewer = require('telescope.previewers').new_buffer_previewer({
            title = "Preview",
            define_preview = function(self, entry, _)
                local path = ws_by_fmt[entry.value].Path
                require('telescope.previewers').buffer_previewer_maker(path, self.state.bufnr, {
                    use_ft_detect = true
                })
            end
        })
        opts.keys = M.opts.keys
        opts.get_preview_content = function(entry)
            return ws_by_fmt[entry.value].Path
        end

        tele.pick(opts)
    end, opts.ws_name)
end

M.WorkspacersJson = function(ws_name)
    rpc.req_res(rpc_names.json, function(json_path)
        vim.cmd("edit " .. json_path)
    end, ws_name)
end

M.PromoteWorkspace = function(opts, promote)
    if opts.text and opts.text ~= "" then
        vim.notify("Cannot reorder with search text", vim.log.levels.ERROR)
        return
    end

    local rpc_action = promote and rpc_names.promote or rpc_names.demote
    if opts.selected and opts.selected[1] then
        rpc.req_res(rpc_action, function(new_idx)
                opts.close()
                opts.selected_idx = new_idx
                M.WorkspacersList(opts)
            end,
            opts.ws_name, opts.selected[1])
    else
        vim.notify("No selected Workspace", vim.log.levels.ERROR)
    end
end

M.DemoteWorkspace = function(opts)
    if opts.text and opts.text ~= "" then
        vim.notify("Cannot Demote with search text", vim.log.levels.ERROR)
        return
    end
    if opts.selected and opts.selected[1] then
        rpc.req_res(rpc_names.demote,
            function(new_idx)
                opts.close()
                opts.selected_idx = new_idx
                M.WorkspacersList(opts)
            end,
            opts.ws_name, opts.selected[1])
    else
        vim.notify("No selected Workspace", vim.log.levels.ERROR)
    end
end

return M
