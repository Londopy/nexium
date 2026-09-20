// The Zed extension for Nexium. Zed extensions are WebAssembly, so this is
// the one Rust file in the repository: it tells Zed to start `nx lsp` from
// the PATH for every Nexium buffer. The grammar and queries do the rest.
use zed_extension_api as zed;

struct Nexium;

impl zed::Extension for Nexium {
    fn new() -> Self {
        Nexium
    }

    fn language_server_command(
        &mut self,
        _id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        let nx = worktree
            .which("nx")
            .ok_or("nx is not on the PATH; install it: https://londopy.github.io/nexium/docs/install.html")?;
        Ok(zed::Command {
            command: nx,
            args: vec!["lsp".to_string()],
            env: Vec::new(),
        })
    }
}

zed::register_extension!(Nexium);
