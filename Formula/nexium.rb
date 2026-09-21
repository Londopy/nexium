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
  url "https://github.com/Londopy/nexium/archive/refs/tags/v1.1.0.tar.gz"
  sha256 "008631bb78967a3c6ec160f37dc1332cdba7e2fb9c5b8a385b0c04a8cf0e0ecd"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.1.0/nx-v1.1.0-aarch64-apple-darwin.tar.gz"
      sha256 "95b4227414ba2f5147b3dc690f0df0cc9e0f3d0353c5d6925b8d9cab7fb292f1"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Londopy/nexium/releases/download/v1.1.0/nx-v1.1.0-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "a6df04704c4628362ddaae8631adc01b462eba238f868b50fc28a8991670f41c"
    end
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.1.0/nx-v1.1.0-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "7cfa37760d1f84f4c92d8ad9bad59d6896e1e82000c967be4392bcac7a71787b"
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
