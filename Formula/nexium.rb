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
  url "https://github.com/Londopy/nexium/archive/refs/tags/v1.4.0.tar.gz"
  sha256 "1d5deab0971d67dee0631ba40802d5c8823849d59002eb7d2dc0ad509075955a"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.4.0/nx-v1.4.0-aarch64-apple-darwin.tar.gz"
      sha256 "4d174fa94d9725327b3ff9c24c3f8a61e75bcd9ff54aa0dbc3defdee6ef5cda7"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Londopy/nexium/releases/download/v1.4.0/nx-v1.4.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "d2bee5ed476ae9aa84f2061185d81c2e76c6f7c0cda6303bfff110ff1db30dab"
    end
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.4.0/nx-v1.4.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "65354d5aede09b8542e48c445c420cc1fc546574f7568aaaaebab827f26a2647"
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
