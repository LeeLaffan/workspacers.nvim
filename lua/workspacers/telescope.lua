local M = {}
local pickers = require "telescope.pickers"
local finders = require "telescope.finders"
local previewers = require "telescope.previewers"
local conf = require("telescope.config").values
local actions = require("telescope.actions")
local action_state = require "telescope.actions.state"
local themes = require("telescope.themes")

local close = function(opts)
    actions.close(opts.bufnr)
end

local select_callback = function(opts)
    opts.selected = action_state.get_selected_entry()
    opts.callback(opts)
end

local function get_theme(opts)
    if type(opts.theme) == "function" then
        return opts.theme()
    end
    if type(opts.theme) == "table" then
        return opts.theme
    end

    if opts.theme == 'ivy' then
        return themes.get_ivy({
            layout_config = {
                width = string.len(opts.records[1] or "") + 5,
                height = #opts.records + 4,
            }
        })
    elseif opts.theme == 'dropdown' then
        return themes.get_dropdown({
            layout_config = {
                -- width = string.len(opts.records[1] or "") + 50,
                --     height = #opts.records + 4,
                mirror = true,
                --     anchor = "W"
            }
        })
    elseif opts.theme == 'cursor' then
        return themes.get_cursor()
    else
        vim.print("ERROR: Workspacers has unknown theme value in setup")
        return themes.get_cursor()
    end
end

--@params opts table Options
--@params opts.records string[] Records to be displayed
--@params opts.close bool Should close on select (default: true)
--@params opts.callback function Callback on close
M.pick = function(opts)
    opts = opts or {}
    opts.finder = finders.new_table {
        results = opts.records
    }
    opts.sorter = conf.generic_sorter({})
    -- opts.previewer = opts.previewer or previewers.new_buffer_previewer({
    --     define_preview = function(self, entry, status)
    --         local content = opts.get_preview_content and opts.get_preview_content(entry) or "No preview available"
    --         vim.api.nvim_buf_set_lines(self.state.bufnr, 0, -1, false, vim.split(content, "\n"))
    --     end
    -- })
    opts.preview = true
    -- In your pick function
    opts.attach_mappings = function(bufnr, map)
        -- Add Tele vars to opts
        opts.bufnr = bufnr
        opts.map = map
        opts.close = function() close(opts) end
        actions.select_default:replace(function()
            select_callback(opts)
        end)
        -- Add keys
        for k, v in pairs(opts.keys) do
            map({ 'i', 'n' }, k, function()
                opts.selected = action_state.get_selected_entry()
                opts.text = action_state.get_current_line()
                v(opts)
            end)
        end
        return true
    end
    local picker = pickers.new(opts, get_theme(opts))
    picker:register_completion_callback(function(picker_instance)
        picker_instance:set_selection(opts.selected_idx)
    end)
    picker:find()
end

return M
