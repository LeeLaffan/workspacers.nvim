local M = {}

local rpc = require('workspacers.rpc_handler')
local tele = require('workspacers.telescope')
local icons = require('utils.icons')

M.setup = function(opts)
    M.opts = opts or {}
end

local rpc_names = {
    list = 'lee.ws.list',
    add = 'lee.ws.add',
    delete = 'lee.ws.delete',
    json = 'lee.ws.json',
    promote = 'lee.ws.promote',
    demote = 'lee.ws.demote',
    record = 'lee.ws.record',
    replace = 'lee.ws.replace',
}

local function read_file(file)
    local f = io.open(file, "r")
    if not f then return {} end

    local lines = {}
    for line in f:lines() do
        table.insert(lines, line)
    end
    f:close()

    return lines
end

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
    local path = vim.fn.getbufinfo(0)[1].name
    if path:match("^oil://") then
        return path:gsub("^oil://", "")
    else
        return path
    end
end

M.WorkspacersAdd = function()
    local name, success = try_get_input({
        prompt = "Enter Workspace Name: "
    })
    if not success then return end

    vim.print(vim.fn.getbufinfo(0)[1].name)
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

    rpc.req(rpc_names.add, new_ws)
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
            rpc.req(rpc_names.replace, rpc_args)
            opts.close()
            M.WorkspacersList()
        else
            vim.notify("No selected Workspace", vim.log.levels.ERROR)
        end
    end, opts.selected[1])
end

M.DeleteWorkspace = function(opts)
    if vim.fn.confirm("Delete Selected Workspace: ", "&Yes\n&No", 2) == 1 then
        if opts.selected and opts.selected[1] then
            rpc.req(rpc_names.delete, opts.selected[1])
            opts.close()
            M.WorkspacersList()
        else
            vim.notify("No selected Workspace", vim.log.levels.ERROR)
        end
    end
end

local function select_workspace(ws)
    vim.cmd("edit " .. ws.Path)
    vim.cmd("cd " .. ws.Path)
end

M.WorkspacersList = function(idx)
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
        local tele_opts = {
            vals = fmt_vals,
            ws_by_fmt = ws_by_fmt,
            callback = function(call_opts)
                call_opts.close()
                if call_opts.selected and call_opts.selected[1] then
                    select_workspace(ws_by_fmt[call_opts.selected[1]])
                else
                    vim.notify("No selected Workspace", vim.log.levels.ERROR)
                end
            end,
            previewer = require('telescope.previewers').new_buffer_previewer({
                title = "Preview",
                define_preview = function(self, entry, _)
                    local path = ws_by_fmt[entry.value].Path
                    require('telescope.previewers').buffer_previewer_maker(path, self.state.bufnr, {
                        use_ft_detect = true
                    })
                end
            }),
            prompt_title = icons.hammer .. " Workspacers " .. icons.planet,
            keys = M.opts.keys,
            start_idx = idx or 0,
            get_preview_content = function(entry)
                -- vim.print(entry.value)
                return ws_by_fmt[entry.value].Path
            end,
        }
        tele.pick(tele_opts)
    end)
end

M.WorkspacersJson = function()
    rpc.req_res(rpc_names.json, function(json_path)
        vim.cmd("edit " .. json_path)
    end)
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
                M.WorkspacersList(new_idx)
            end,
            opts.selected[1])
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
                M.WorkspacersList(new_idx)
            end,
            opts.selected[1])
    else
        vim.notify("No selected Workspace", vim.log.levels.ERROR)
    end
end

return M
