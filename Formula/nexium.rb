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
  url "https://github.com/Londopy/nexium/archive/refs/tags/v1.0.3.tar.gz"
  sha256 "007d3d290be08ae2ee7412b28dd23f8a89f4d577434069f1539d110b4d0a195f"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.0.3/nx-v1.0.3-aarch64-apple-darwin.tar.gz"
      sha256 "5592d6fdc00c765db34e110dcbee758bda890432ba4cebd6095f020af970c2ab"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/Londopy/nexium/releases/download/v1.0.3/nx-v1.0.3-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0864de94d9de06f137a25a3bccfea74217126f4cfce555e98bf12b689b3f79ca"
    end
    on_arm do
      url "https://github.com/Londopy/nexium/releases/download/v1.0.3/nx-v1.0.3-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "1cf6b625f288526cf9d847b158d6df99fdf136919c24509b000eee45d0cbb77f"
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
