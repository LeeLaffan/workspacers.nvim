local M = {}
local pickers = require "telescope.pickers"
local finders = require "telescope.finders"
local conf = require("telescope.config").values
local actions = require("telescope.actions")
local action_state = require "telescope.actions.state"

local close = function(opts)
    actions.close(opts.bufnr)
end

local callback = function(opts)
    opts.selected = action_state.get_selected_entry()
    opts.callback(opts)
end

--@params opts table Options
--@params opts.vals string[] Values to be displayed
--@params opts.close bool Should close on select (default: true)
--@params opts.callback function Callback on close
M.pick = function(opts)
    opts = opts or {}
    opts.start_idx = opts.start_idx or 0
    opts.finder = finders.new_table {
        results = opts.vals
    }
    opts.sorter = conf.generic_sorter({})
    opts.attach_mappings = function(bufnr, map)
        opts.bufnr = bufnr
        opts.map = map
        opts.close = function() close(opts) end
        actions.select_default:replace(function(tele_opts)
            vim.print(tele_opts)
            -- close(opts)
            callback(opts)
        end)
        for k, v in pairs(opts.keys) do
            map({ 'i', 'n' }, k, function()
                opts.selected = action_state.get_selected_entry()
                opts.text = action_state.get_current_line()
                v(opts)
            end)
        end
        return true
    end
    local picker = pickers.new(opts, require("telescope.themes").get_dropdown({
        layout_config = {
            width = string.len(opts.vals[1] or "") + 5,
            height = #opts.vals + 4,
        }
    }))
    picker:register_completion_callback(function(picker_instance)
        picker_instance:set_selection(opts.start_idx)
    end)
    picker:find()
end

return M

-- for id, rec in ipairs(records) do
--     -- if rec.Record.Name == selection[1] then
--     --     vim.notify("Changing to Dir: ", rec.Record.Path)
--     --     vim.cmd("Oil --float " .. string.gsub(rec.Record.Path, '\n', ''))
--     -- end
-- end

-- vim.api.nvim_put({ color_map[selection[1]] }, "", false, true)
-- end)
