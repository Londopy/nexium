-- Nexium for Neovim: registers the tree-sitter parser with nvim-treesitter,
-- the language server (`nx lsp`) with the built-in LSP client, and the
-- regex syntax from ../vim as the fallback when the parser is not installed.
--
--   require("nexium").setup()
--
-- Options (all optional):
--   lsp = false          do not configure the language server
--   treesitter = false   do not register the parser
--   cmd = { "nx", "lsp" }  the server command

local M = {}

-- editors/neovim/lua/nexium/init.lua -> editors
local here = debug.getinfo(1, "S").source:sub(2)
local editors = vim.fn.fnamemodify(here, ":h:h:h:h")

local function register_parser()
  local ok, parsers = pcall(require, "nvim-treesitter.parsers")
  if not ok then
    return
  end
  local spec = {
    install_info = {
      url = "https://github.com/Londopy/nexium",
      location = "editors/tree-sitter-nexium",
      files = { "src/parser.c" },
      branch = "main",
      generate = false,
      queries = "editors/neovim/queries/nexium",
    },
    filetype = "nexium",
  }
  if parsers.get_parser_configs then
    -- nvim-treesitter master
    parsers.get_parser_configs().nexium = spec
  else
    -- nvim-treesitter main (2025 and later): the parsers module is the table
    parsers.nexium = spec
  end
  pcall(vim.treesitter.language.register, "nexium", "nexium")
end

local function register_lsp(cmd)
  if vim.lsp.config and vim.lsp.enable then
    -- Neovim 0.11 and later
    vim.lsp.config("nexium", {
      cmd = cmd,
      filetypes = { "nexium" },
      root_markers = { "nexium.toml", ".git" },
    })
    vim.lsp.enable("nexium")
    return
  end
  local ok, lspconfig = pcall(require, "lspconfig")
  if not ok then
    -- no nvim-lspconfig: start the server ourselves on every Nexium buffer
    vim.api.nvim_create_autocmd("FileType", {
      pattern = "nexium",
      callback = function(ev)
        vim.lsp.start({
          name = "nexium",
          cmd = cmd,
          root_dir = vim.fs.dirname(vim.fs.find({ "nexium.toml", ".git" }, { upward = true, path = ev.match })[1]),
        })
      end,
    })
    return
  end
  local configs = require("lspconfig.configs")
  if not configs.nexium then
    configs.nexium = {
      default_config = {
        cmd = cmd,
        filetypes = { "nexium" },
        root_dir = lspconfig.util.root_pattern("nexium.toml", ".git"),
        single_file_support = true,
      },
    }
  end
  lspconfig.nexium.setup({})
end

function M.setup(opts)
  opts = opts or {}
  -- the regex syntax, indent script and :Nx commands from ../vim are the
  -- fallback until the tree-sitter parser is installed
  vim.opt.rtp:append(editors .. "/vim")
  if opts.treesitter ~= false then
    register_parser()
  end
  if opts.lsp ~= false then
    register_lsp(opts.cmd or { "nx", "lsp" })
  end
end

return M
