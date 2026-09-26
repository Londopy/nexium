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
  url "https://github.com/Londopy/nexium/archive/refs/tags/v1.4.1.tar.gz"
  sha256 "95d5af2023f42da8b9269914003f9643eafebe24a3ce3423b55c290a4cbbd1a6"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.4.1/nx-v1.4.1-aarch64-apple-darwin.tar.gz"
      sha256 "e30bdb440a1ea1c1b7013dd09519fbdf251c0a7dd359748937bbb6bcb172e5b2"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Londopy/nexium/releases/download/v1.4.1/nx-v1.4.1-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "30cede4f75a67d8cc948aed0b7ac4b6fe6ecbccae669465aa2d66df653e4f4e9"
    end
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.4.1/nx-v1.4.1-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "41d1a15d8e8565969f8763e72f5d6d6990dd8d88faabe9ec0d2a0abfa8d8fdaf"
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
