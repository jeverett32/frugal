$ErrorActionPreference = "Stop"

$BinName = if ($env:BIN_NAME) { $env:BIN_NAME } else { "fgl.exe" }
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $HOME ".local\bin" }

$BinPath = Join-Path $InstallDir $BinName

if (-not (Test-Path $BinPath)) {
    Write-Host "$BinPath not found — nothing to uninstall"
    exit 0
}

Remove-Item -Path $BinPath -Force
Write-Host "Removed $BinPath"
