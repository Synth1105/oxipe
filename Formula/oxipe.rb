class Oxipe < Formula
  desc "A fast, customizable terminal-based typing speed test and practice application"
  homepage "https://github.com/Synth1105/oxipe"
  url "https://github.com/Synth1105/oxipe/archive/refs/tags/v0.3.0.tar.gz"
  sha256 ""
  license "LGPL-2.1"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "oxipe", shell_output("#{bin}/oxipe --help", 2)
  end
end
