#!/bin/bash

# Script to install all binaries from the latest GitHub release

# GitHub repository details
REPO_OWNER="spa5k"
REPO_NAME="nirs"
APP_NAMES=("na" "nci" "ni" "nlx" "nr" "nu" "nun")

# Installation directory preferences
USER_BIN="$HOME/bin"
SYSTEM_BIN="/usr/local/bin"

# Function to determine the OS and architecture
get_os_arch() {
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Linux*)
      os="Linux"
      ;;
    Darwin*)
      os="macOS"
      ;;
    Windows*)
      os="Windows"
      ;;
    *)
      echo "Unsupported OS: $os"
      exit 1
      ;;
  esac

  case "$arch" in
    x86_64*)
      arch="x86_64"
      ;;
    aarch64*)
      arch="aarch64"
      ;;
    arm64*)
      arch="aarch64" # macOS uses arm64, Linux might use aarch64
      ;;
    *)
      echo "Unsupported architecture: $arch"
      exit 1
      ;;
  esac

  echo "$os-$arch"
}

# Determine OS and architecture
os_arch=$(get_os_arch)

# Function to construct the binary filename
construct_binary_name() {
  local app_name="$1"
  local binary_name="$app_name-$os_arch"
  if [[ "$os" == "Windows" ]]; then
    binary_name="$binary_name.exe"
  fi
  echo "$binary_name"
}

# Function to download the binary from GitHub releases
download_binary() {
  local app_name="$1"
  local binary_name=$(construct_binary_name "$app_name")
  local binary_url
  binary_url=$(
    curl -s "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest" |
    grep "browser_download_url.*$binary_name" |
    cut -d '"' -f 4
  )

  if [[ -z "$binary_url" ]]; then
    echo "Error: Binary '$binary_name' not found in the latest release."
    echo "Please check the release page to ensure the binary exists."
    return 1
  fi

  echo "Downloading '$binary_name' from '$binary_url'..."
  curl -L "$binary_url" -o "$binary_name"
  if [[ $? -ne 0 ]]; then
    echo "Error: Failed to download '$binary_name'."
    return 1
  fi
  return 0
}

# Function to install the binary
install_binary() {
  local app_name="$1"
  local install_dir="$2"
  local binary_name=$(construct_binary_name "$app_name")

  # Create the installation directory if it doesn't exist
  mkdir -p "$install_dir"

  # Copy the binary to the installation directory
  cp "./$binary_name" "$install_dir/$app_name"

  # Make the binary executable
  chmod +x "$install_dir/$app_name"

  echo "Binary '$binary_name' installed to '$install_dir/$app_name'"
}

# Function to add the installation directory to the PATH
add_to_path() {
  local install_dir="$1"
  # Determine the shell configuration file
  if [[ -f "$HOME/.bashrc" ]]; then
    profile_file="$HOME/.bashrc"
  elif [[ -f "$HOME/.zshrc" ]]; then
    profile_file="$HOME/.zshrc"
  elif [[ -f "$HOME/.profile" ]]; then
    profile_file="$HOME/.profile"
  else
    profile_file="$HOME/.bashrc"
    touch "$profile_file"
  fi

  # Check if the PATH already contains the installation directory
  if grep -q "export PATH=\"$install_dir:\$PATH\"" "$profile_file"; then
    echo "PATH already contains '$install_dir'. Skipping PATH modification."
  else
    # Add the installation directory to the PATH
    echo "export PATH=\"$install_dir:\$PATH\"" >> "$profile_file"
    echo "Added '$install_dir' to PATH in '$profile_file'"
  fi

  # Source the profile file to update the PATH in the current shell
  if [[ -n "$profile_file" ]]; then
    source "$profile_file"
    echo "Sourced '$profile_file' to update PATH in the current shell."
  fi

  echo "Please open a new terminal or run 'source $profile_file' for the changes to fully take effect."
}

# Install all binaries
for app_name in "${APP_NAMES[@]}"; do
  # Construct the binary filename
  binary_name=$(construct_binary_name "$app_name")

  # Download the binary
  if ! download_binary "$app_name"; then
    echo "Skipping installation of '$app_name'."
    continue
  fi

  # Attempt to install to the user's bin directory
  install_binary "$app_name" "$USER_BIN"
  if [[ $? -eq 0 ]]; then
    echo "Installed '$app_name' to user bin directory."
  else
    # If user bin install fails, attempt to install to the system bin directory
    echo "Attempting to install '$app_name' to system bin directory (requires sudo)."
    sudo install_binary "$app_name" "$SYSTEM_BIN"
    if [[ $? -eq 0 ]]; then
      echo "Installed '$app_name' to system bin directory."
    else
      echo "Installation of '$app_name' failed."
    fi
  fi
  rm -f "$binary_name"
done

add_to_path "$USER_BIN"

echo "Installation complete."
echo "Please open a new terminal or run 'source $profile_file' for the changes to fully take effect."
exit 0
