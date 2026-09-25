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
  url "https://github.com/Londopy/nexium/archive/refs/tags/v1.3.2.tar.gz"
  sha256 "dc8033379381b600b3028e0562a0fff30f20c564d95a6f8284e04f8c5f93f851"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.3.2/nx-v1.3.2-aarch64-apple-darwin.tar.gz"
      sha256 "c63ce9c9a7910ed49d487d401e97dec7ac7dfcc3aa5482c05ab855a5b3c04e98"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Londopy/nexium/releases/download/v1.3.2/nx-v1.3.2-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "5559914fe822f10ca65be6a0a86d672f143bed3b17a7fe6b99bc9f460c7a655a"
    end
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.3.2/nx-v1.3.2-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "785a2d781d64455c69bfc492c74856d9447fac1613995feeb8dbd63d900f8b5c"
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
