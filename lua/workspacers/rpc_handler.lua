local M = {}

local start_job = function(opts)
    local args = { opts.binary }
    if opts.json_dir then
        table.insert(args, '--json-dir=' .. M.opts.json_dir)
    end
    return vim.fn.jobstart(
        args,
        {
            rpc = true,
            on_stderr = function(_, data)
                if data and data[1] ~= "" then
                    vim.schedule(function()
                        vim.notify("Plugin error: " .. vim.inspect(data), vim.log.levels.ERROR)
                    end)
                end
            end
        }
    )
end

M.req = function(method, ...)
    -- vim.print('sending request: ', method)
    return vim.rpcrequest(M.job_id, method, ...)
end

M.req_res = function(name, callback, ...)
    local result = M.req(name, ...)
    if result then
        callback(result)
    else
        print("No result available")
    end
    return result
end

M.setup = function(opts)
    M.opts = opts or {}
    if not opts.binary then
        M.opts.binary = "workspacers-nvim"
    end
    M.job_id = start_job(opts)
end

return M
