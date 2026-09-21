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
  url "https://github.com/Londopy/nexium/archive/refs/tags/v1.0.2.tar.gz"
  sha256 "a2fafd58f888ce582aa859834cab92b0c01a402daa29a197a8756308c366473d"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.0.2/nx-v1.0.2-aarch64-apple-darwin.tar.gz"
      sha256 "3e754baea58c23db8329489f340e49ddb03eeeaaf4c3351d2afbd85b2bcd0a89"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Londopy/nexium/releases/download/v1.0.2/nx-v1.0.2-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "4ef02ce94556b902941eb85aa2442cab38831728ca87d87b73fe4595d61b37b5"
    end
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.0.2/nx-v1.0.2-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "6e842bb93e3a86845be24885ccd7bb3b1313542035ca1970f24d23f4b6351bc5"
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
