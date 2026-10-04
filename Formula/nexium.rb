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
  url "https://github.com/Londopy/nexium/archive/refs/tags/v1.5.0.tar.gz"
  sha256 "c0f1e63cdc83c33ce112fca23c926eb6058daa62688bcf74f9f7d0d9a5c3faeb"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.5.0/nx-v1.5.0-aarch64-apple-darwin.tar.gz"
      sha256 "a4f7ae2cbc2043a94599f376d8e20394112cd2aa613f60ac948d75ab351a6501"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Londopy/nexium/releases/download/v1.5.0/nx-v1.5.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "1aee1466924583575912246151f32b2750fd16eb6c2ba8e7807b4d808170e99a"
    end
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.5.0/nx-v1.5.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "3ccfdebe2c177555a546712007e20da2476e03bfdc4dc19fe7a36a2c43ed8d4d"
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
