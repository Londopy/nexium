# The Nexium language, for Homebrew. Written by scripts/packaging.py at each
# release; this repository is the tap:
#
#     brew tap londopy/tap https://github.com/Londopy/nexium
#     brew install londopy/tap/nexium
#
# The release build where there is one (Apple Silicon, Linux x86-64 and
# aarch64); on any other machine, the compiler's own C, bootstrap/nx.c, built
# from the source tarball by the system compiler.
class Nexium < Formula
  desc "Nexium language: a compiler that emits C and ships libraries, packages and tools"
  homepage "https://londopy.github.io/nexium/"
  url "https://github.com/Londopy/nexium/archive/refs/tags/v1.3.0.tar.gz"
  sha256 "82d9482838955e7211cf4350a9a14c8c3be59956c5416801bcf7cae6c22c5722"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.3.0/nx-v1.3.0-aarch64-apple-darwin.tar.gz"
      sha256 "f84511bd1698fb5b8a30fe255a79ba731da18b81600053f059c72aa427b5677e"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Londopy/nexium/releases/download/v1.3.0/nx-v1.3.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "85ba77260797b586bd5ad71f6f173c8925f53a5243404c80faebfeaf3a258576"
    end
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.3.0/nx-v1.3.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "6b2e3f30aa7028532993cf3cb4154092d574133eb519682fc3b62cfd22795f8b"
    end
  end

  def install
    unless File.exist?("nx")
      # no release for this machine: the one C file
      libs = OS.mac? ? [] : ["-lm", "-lc", "-lpthread"]
      system ENV.cc, "-std=gnu11", "-O2", "-w", "-fno-strict-aliasing", "-o", "nx", "bootstrap/nx.c", *libs
    end
    bin.install "nx"
    pkgshare.install "examples", "std", "docs"
    doc.install "README.md", "CHANGELOG.md"
    man1.install "nx.1" if File.exist?("nx.1")
    generate_completions_from_executable(bin/"nx", "completions", shells: [:bash, :zsh, :fish])
  end

  def caveats
    <<~EOS
      nx builds programs with a C compiler: the Xcode command line tools on
      macOS (xcode-select --install), gcc, clang or zig on Linux; `nx doctor`
      says which one it found. The examples are in #{pkgshare}/examples.
    EOS
  end

  test do
    (testpath/"hello.nx").write "fn main() { println(\"hi\", .{}) }\n"
    assert_match "hi", shell_output("#{bin}/nx run hello.nx")
  end
end
