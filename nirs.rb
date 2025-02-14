class Nirs < Formula
  desc "A Rust implementation of the ni command-line tool for simplified package management"
  homepage "https://github.com/spark/nirs"
  version "0.0.5"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/spa5k/nirs/releases/download/v0.0.5/na-macOS-x86_64", using: CurlDownloadStrategy
      sha256 "67c655514ca783cb959408e5452e9e0395f1a9a8c9528c199449f5b0865579a9"
    end
    if Hardware::CPU.arm?
      url "https://github.com/spa5k/nirs/releases/download/v0.0.5/na-macOS-aarch64", using: CurlDownloadStrategy
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/spa5k/nirs/releases/download/v0.0.5/na-Linux-x86_64", using: CurlDownloadStrategy
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    end
    if Hardware::CPU.arm?
      url "https://github.com/spa5k/nirs/releases/download/v0.0.5/na-Linux-aarch64", using: CurlDownloadStrategy
      sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
    end
  end

  def install
    # Install all binaries
    %w[na nci ni nlx nr nu nun np ns].each do |app|
      os = case RbConfig::CONFIG['host_os']
           when /darwin/
             "macOS"
           when /linux/
             "Linux"
           else
             raise "Unsupported OS: #{RbConfig::CONFIG['host_os']}"
           end
      binary_name = "\#{app}-\#{os}-\#{Hardware::CPU.intel? ? \"x86_64\" : \"aarch64\"}"
      binary_url = "https://github.com/spa5k/nirs/releases/download/v0.0.5/\#{binary_name}"
      
      # Download the binary
      system "curl", "-L", binary_url, "-o", app

      # Make the binary executable
      chmod "+x", app

      # Install the binary
      bin.install app
    end
  end

  test do
    system "\#{bin}/na", "--version"
  end
end
