class Ap < Formula
  desc "Packs C assignment submissions for Canvas upload"
  homepage "https://github.com/cat-forgor/AssignmentPacker"
  version "1.0.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v1.0.1/ap-macos-arm64"
      sha256 "d25295d743fcef0306676bf9017a2abbb051206e1a15a6bc0c97437c30e145b6"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v1.0.1/ap-linux-x64"
      sha256 "47df19c257e59037217bdb84955d3ae1d4bf3ec42db3147382c82065fc569dc2"
    end
  end

  def install
    binary = Dir["ap-*"].first || "ap"
    mv binary, "ap"
    bin.install "ap"
  end

  test do
    assert_match "assignment_packer", shell_output("#{bin}/ap --version")
  end
end
