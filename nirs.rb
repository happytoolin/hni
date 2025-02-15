class Nirs < Formula
  desc "A Rust implementation of the ni command-line tool for simplified package management"
  homepage "https://github.com/spark/nirs"
  version "{{VERSION}}.0.7"

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/spa5k/nirs/releases/download/v{{MACOS_X86_64_URL}}.0.7/na-macOS-x86_64", using: CurlDownloadStrategy
      sha256 "d5a244c0808cef108ec109095fcfd0acecbaf5d9ccefdf2640868ed452601c87"
    end
    if Hardware::CPU.arm?
      url "https://github.com/spa5k/nirs/releases/download/v{{MACOS_AARCH64_URL}}.0.7/na-macOS-aarch64", using: CurlDownloadStrategy
      sha256 "639c09570a6c307d4fe4f5af04fa1ccb783274151b13601543552e750f7bfcf8"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/spa5k/nirs/releases/download/v{{LINUX_X86_64_URL}}.0.7/na-Linux-x86_64", using: CurlDownloadStrategy
      sha256 "69166ffc8bb2d483d38c9daccc95151f99951b930c8f48d4ef959cce9363a13f"
    end
    if Hardware::CPU.arm?
      url "https://github.com/spa5k/nirs/releases/download/v{{LINUX_AARCH64_URL}}.0.7/na-Linux-aarch64", using: CurlDownloadStrategy
      sha256 "e5590eb6a40ea3e901d21c8abcf37c76e2f889bb49fb0af563b94596602e2a8f"
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
      arch = Hardware::CPU.intel? ? "x86_64" : "aarch64"
      binary_name = "\#{app}-\#{os}-\#{arch}"
      binary_url = "https://github.com/spa5k/nirs/releases/download/v{{BASE_URL}}.0.7/\#{binary_name}"
      
      # Download the binary
      system "curl", "-L", binary_url, "-o", app

      # Make the binary executable
      chmod +x", app

      # Install the binary
      bin.install app
    end
  end

  test do
    system "\#{bin}/na", "--version"
  end
end
