class Wutag < Formula
  desc "CLI Tool for tagging and organizing files by tags"
  homepage "https://github.com/wojciechkepka/wutag"
  url "https://github.com/wojciechkepka/wutag/archive/0.3.0.tar.gz"
  sha256 "e1c22f24f3d151d1a0cccc60be2e3d6b7fa083e8fa8c732535db635bd4a294c0"
  license "MIT"

  depends_on "rust" => :build
  depends_on "bash" => :install

  def install
    system "cargo", "install", *std_cargo_args
    wutag_bin = "#{bin}/wutag"
    system "/bin/bash", "-c", "#{wutag_bin} print-completions bash > wutag.bash"
    system "/bin/bash", "-c", "#{wutag_bin} print-completions fish > wutag.fish"
    system "/bin/bash", "-c", "#{wutag_bin} print-completions zsh > _wutag"
    bash_completion.install "wutag.bash"
    fish_completion.install "wutag.fish"
    zsh_completion.install "_wutag"
  end

  test do
    wutag_bin = "#{bin}/wutag"
    assert_equal "wutag 0.3.0\n", shell_output("bash -c '#{wutag_bin} 2>&1 | head -n 1'")
  end
end
