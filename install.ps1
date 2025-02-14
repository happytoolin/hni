# PowerShell script to install all binaries from the latest GitHub release

# GitHub repository details
$RepoOwner = "spa5k"
$RepoName = "nirs"
$AppNames = @("na", "nci", "ni", "nlx", "nr", "nu", "nun", "np", "ns")

# Installation directory preferences
$UserBin = Join-Path $env:LOCALAPPDATA "Programs\nirs"
$SystemBin = "C:\Program Files\nirs"

# Function to determine the OS and architecture
function Get-OSArch {
  $os = Get-WmiObject -Class Win32_OperatingSystem | Select-Object -ExpandProperty Caption
  $arch = Get-WmiObject -Class Win32_ComputerSystem | Select-Object -ExpandProperty SystemType

  if ($os -like "*Windows*") {
    $os = "Windows"
  } else {
    Write-Host "Unsupported OS: $os"
    exit 1
  }

  if ($arch -like "*x64*") {
    $arch = "x86_64"
  } elseif ($arch -like "*ARM64*" -or $arch -like "*AArch64*") {
    $arch = "aarch64"
  } else {
    Write-Host "Unsupported architecture: $arch"
    exit 1
  }

  return "$os-$arch"
}

# Determine OS and architecture
$OSArch = Get-OSArch

# Function to construct the binary filename
function Construct-BinaryName {
  param (
    [string]$AppName
  )
  $BinaryName = "$AppName-$OSArch.exe"
  return $BinaryName
}

# Function to download the binary from GitHub releases
function Download-Binary {
  param (
    [string]$AppName
  )

  # Construct the binary filename
  $BinaryName = Construct-BinaryName -AppName $AppName

  # Construct the API URL
  $ApiUrl = "https://api.github.com/repos/$RepoOwner/$RepoName/releases/latest"

  # Get the release information
  try {
    $ReleaseInfo = Invoke-RestMethod -Uri $ApiUrl
  } catch {
    Write-Host "Error: Failed to retrieve release information from GitHub API."
    return $false
  }

  # Find the download URL for the binary
  $BinaryUrl = $ReleaseInfo.assets | Where-Object { $_.name -eq $BinaryName } | Select-Object -ExpandProperty browser_download_url

  if (-not $BinaryUrl) {
    Write-Host "Error: Binary '$BinaryName' not found in the latest release."
    Write-Host "Please check the release page to ensure the binary exists."
    return $false
  }

  Write-Host "Downloading '$BinaryName' from '$BinaryUrl'..."

  # Download the binary
  try {
    Invoke-WebRequest -Uri $BinaryUrl -OutFile $BinaryName
  } catch {
    Write-Host "Error: Failed to download '$BinaryName'."
    return $false
  }
  return $true
}

# Function to install the binary
function Install-Binary {
  param (
    [string]$AppName,
    [string]$InstallDir
  )

  # Construct the binary filename
  $BinaryName = Construct-BinaryName -AppName $AppName

  # Create the installation directory if it doesn't exist
  if (-not (Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir
  }

  # Copy the binary to the installation directory
  Copy-Item -Path "./$BinaryName" -Destination "$InstallDir\$AppName.exe"

  Write-Host "Binary '$BinaryName' installed to '$InstallDir\$AppName.exe'"
}

# Function to add the installation directory to the PATH
function Add-ToPath {
  param (
    [string]$InstallDir
  )

  # Get the current PATH environment variable
  $Path = [Environment]::GetEnvironmentVariable("Path", "User")

  # Check if the PATH already contains the installation directory
  if ($Path -like "*$InstallDir*") {
    Write-Host "PATH already contains '$InstallDir'. Skipping PATH modification."
  } else {
    # Add the installation directory to the PATH
    $Path = "$InstallDir;$Path"
    [Environment]::SetEnvironmentVariable("Path", $Path, "User")
    Write-Host "Added '$InstallDir' to PATH."
  }

  Write-Host "Please restart your shell or system for the changes to fully take effect."
}

# Install all binaries
foreach ($AppName in $AppNames) {
  # Construct the binary filename
  $BinaryName = Construct-BinaryName -AppName $AppName

  # Download the binary
  if (-not (Download-Binary -AppName $AppName)) {
    Write-Host "Skipping installation of '$AppName'."
    continue
  }

  # Attempt to install to the user's bin directory
  Install-Binary -AppName $AppName -InstallDir $UserBin
  if ($?) {
    Write-Host "Installed '$AppName' to user bin directory."
  } else {
    # If user bin install fails, attempt to install to the system bin directory
    Write-Host "Attempting to install '$AppName' to system bin directory (requires Administrator)."
    if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
      Write-Host "This script must be run as an administrator to install to the system directory."
    } else {
      Install-Binary -AppName $AppName -InstallDir $SystemBin
      if ($?) {
        Write-Host "Installed '$AppName' to system bin directory."
      } else {
        Write-Host "Installation of '$AppName' failed."
      }
    }
  }
  Remove-Item -Path "./$BinaryName"
}

Add-ToPath -InstallDir $UserBin

Write-Host "Installation complete."
Write-Host "Please restart your shell or system for the changes to fully take effect."
exit 0
