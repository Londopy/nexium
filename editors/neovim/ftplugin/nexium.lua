-- Neovim filetype plugin for Nexium: buffer settings, tree-sitter
-- highlighting when the parser is installed, and :NxRun / :NxTest /
-- :NxCheck / :NxFmt on the current file.
if vim.b.did_ftplugin then
  return
end
vim.b.did_ftplugin = 1

local o = vim.opt_local
o.commentstring = "// %s"
o.comments = ":///,://!,://"
o.formatoptions:remove("t")
o.formatoptions:append("croql")
o.expandtab = true
o.shiftwidth = 4
o.softtabstop = 4
o.tabstop = 4
o.suffixesadd = ".nx"

-- `nx check` as the compiler for :make; the quickfix list gets the diagnostics
o.makeprg = "nx check %"
o.errorformat = [[error: %m,%*[ ]--> %f:%l:%c]]

-- tree-sitter highlighting and indentation when `:TSInstall nexium` has run;
-- otherwise the regex syntax from ../vim (see lua/nexium/init.lua) applies
if vim.treesitter.language.add and pcall(vim.treesitter.language.add, "nexium") then
  pcall(vim.treesitter.start)
  if pcall(require, "nvim-treesitter") then
    o.indentexpr = "v:lua.require'nvim-treesitter'.indentexpr()"
    o.indentkeys = "0{,0},0),0],!^F,o,O,e"
    vim.b.did_indent = 1 -- keeps ../vim/indent/nexium.vim from replacing it
  end
end

local function nx(sub, reload)
  return function()
    local file = vim.fn.shellescape(vim.fn.expand("%"))
    if reload then
      vim.cmd("silent !nx " .. sub .. " " .. file)
      vim.cmd("edit")
    else
      vim.cmd("!nx " .. sub .. " " .. file)
    end
  end
end
vim.api.nvim_buf_create_user_command(0, "NxRun", nx("run"), {})
vim.api.nvim_buf_create_user_command(0, "NxTest", nx("test"), {})
vim.api.nvim_buf_create_user_command(0, "NxCheck", nx("check"), {})
vim.api.nvim_buf_create_user_command(0, "NxFmt", nx("fmt", true), {})

vim.b.undo_ftplugin = "setl cms< com< fo< et< sw< sts< ts< sua< mp< efm< inde<"
