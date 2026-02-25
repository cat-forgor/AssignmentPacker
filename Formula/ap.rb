class Ap < Formula
  desc "Packs C assignment submissions for Canvas upload"
  homepage "https://github.com/cat-forgor/AssignmentPacker"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v#{version}/ap-macos-arm64"
      sha256 "PLACEHOLDER"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v#{version}/ap-linux-x64"
      sha256 "PLACEHOLDER"
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
