{
  # nix run github:Londopy/nexium -- version
  # nix profile install github:Londopy/nexium
  #
  # nx built from the compiler's one C file, bootstrap/nx.c, by the platform's
  # C compiler, with that compiler on nx's PATH so programs build too (nx
  # falls back to cc, gcc or clang when there is no zig). The standard
  # library is embedded in the binary; the examples and the docs land in
  # share/nexium, the manual page in share/man, the completions where each
  # shell looks. The checks build the example the tests run first.
  description = "The Nexium language: a compiler that emits C and ships libraries, packages and tools";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
      version = builtins.head (builtins.match ".*const VERSION = \"([^\"]+)\".*" (builtins.readFile ./self/nx.nx));
    in
    {
      packages = forAll (pkgs: rec {
        nexium = pkgs.stdenv.mkDerivation {
          pname = "nexium";
          inherit version;
          src = self;
          nativeBuildInputs = [ pkgs.makeWrapper pkgs.installShellFiles ];
          dontConfigure = true;
          buildPhase = ''
            runHook preBuild
            libs=""
            if [ "$(uname)" != "Darwin" ]; then libs="-lm -lc -lpthread"; fi
            $CC -std=gnu11 -O2 -w -fno-strict-aliasing -o nx bootstrap/nx.c $libs
            ./nx man > nx.1
            for sh in bash zsh fish; do ./nx completions $sh > nx.$sh; done
            runHook postBuild
          '';
          installPhase = ''
            runHook preInstall
            install -Dm755 nx $out/bin/nx
            mkdir -p $out/share/nexium
            cp -r examples std docs $out/share/nexium/
            cp README.md LICENSE CHANGELOG.md $out/share/nexium/
            installManPage nx.1
            installShellCompletion --bash --name nx nx.bash --zsh --name _nx nx.zsh --fish nx.fish
            wrapProgram $out/bin/nx --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.stdenv.cc ]}
            runHook postInstall
          '';
          meta = with pkgs.lib; {
            description = "The Nexium language: a compiler that emits C and ships libraries, packages and tools";
            homepage = "https://londopy.github.io/nexium/";
            license = licenses.mit;
            mainProgram = "nx";
            platforms = systems;
          };
        };
        default = nexium;
      });

      apps = forAll (pkgs: {
        default = { type = "app"; program = "${self.packages.${pkgs.system}.nexium}/bin/nx"; };
      });

      checks = forAll (pkgs: {
        hello = pkgs.runCommand "nexium-hello" { } ''
          cd ${self}
          ${self.packages.${pkgs.system}.nexium}/bin/nx run examples/hello.nx --out-dir $TMPDIR/out > $out
          grep -q "hello" $out
        '';
      });

      devShells = forAll (pkgs: {
        default = pkgs.mkShell { packages = [ self.packages.${pkgs.system}.nexium pkgs.zig ]; };
      });
    };
}
