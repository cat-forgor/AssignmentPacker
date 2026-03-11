class Ap < Formula
  desc "Packs C assignment submissions for Canvas upload"
  homepage "https://github.com/cat-forgor/AssignmentPacker"
  version "1.0.2"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v1.0.2/ap-macos-arm64"
      sha256 "57e5c1f18e546aaaea4c07abf7aae76534d4aad710b0d155e6c15bf9c1f73b24"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v1.0.2/ap-linux-x64"
      sha256 "0cd9d4580916fbf85bd4f989ad56ec69c12d8f1e3964170665d3386e3ed3a443"
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
