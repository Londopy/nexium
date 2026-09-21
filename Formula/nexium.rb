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
  url "https://github.com/Londopy/nexium/archive/refs/tags/v1.2.0.tar.gz"
  sha256 "edd0ff3b005ba9d736350810bf16dc8ec6bae0f0b8792d81276d45738ba98162"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.2.0/nx-v1.2.0-aarch64-apple-darwin.tar.gz"
      sha256 "a962a3be3b5b16ad614811806eb192b521a01a4883f0bfb9e802ac5d40e57e8e"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Londopy/nexium/releases/download/v1.2.0/nx-v1.2.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "aaee16a68e98c257cccea33ba5f986e526598f4f3e6da05a48a84a4c861808e6"
    end
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.2.0/nx-v1.2.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "a559177c8145bce65fab0dc1b25b73807d4112373624acc2a9aaebd13e684ffc"
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
