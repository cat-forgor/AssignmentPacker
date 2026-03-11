class Ap < Formula
  desc "Packs C assignment submissions for Canvas upload"
  homepage "https://github.com/cat-forgor/AssignmentPacker"
  version "1.0.3"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v1.0.3/ap-macos-arm64"
      sha256 "5acb9a705f86e9ce6d9ae7adfe28fdd00b13bdcc4167dc6d0f6cd16943690135"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v1.0.3/ap-linux-x64"
      sha256 "c8d849188940e3e268b76788e44691d3644d0d50cacf0197bba4e9bf5aead6a8"
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
