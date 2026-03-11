class Ap < Formula
  desc "Packs C assignment submissions for Canvas upload"
  homepage "https://github.com/cat-forgor/AssignmentPacker"
  version "1.0.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v1.0.0/ap-macos-arm64"
      sha256 "504a7250b0fa5d380fe33bdf87b33889d1540b93de03d8b98d0aa8fbb63670cd"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/cat-forgor/AssignmentPacker/releases/download/v1.0.0/ap-linux-x64"
      sha256 "1e71c8e6cf1982b8f2a8c9df8d2825684820e0174a36d54ea3d3f4a7769ef34d"
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
