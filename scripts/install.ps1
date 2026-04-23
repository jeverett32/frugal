$ErrorActionPreference = "Stop"

$RepoOwner = if ($env:REPO_OWNER) { $env:REPO_OWNER } else { "jeverett32" }
$RepoName = if ($env:REPO_NAME) { $env:REPO_NAME } else { "frugal" }
$BinName = if ($env:BIN_NAME) { $env:BIN_NAME } else { "fgl.exe" }
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $HOME ".local\bin" }
$Version = if ($env:VERSION) { $env:VERSION } else { "latest" }

function Resolve-Version {
    if ($Version -ne "latest") {
        return $Version
    }

    try {
        Invoke-WebRequest -Method Head -MaximumRedirection 0 `
            -Uri "https://github.com/$RepoOwner/$RepoName/releases/latest" | Out-Null
    } catch {
        $location = $_.Exception.Response.Headers.Location
        if ($location) {
            return ($location.ToString() -split "/")[-1]
        }
    }

    throw "could not resolve latest release version"
}

function Main {
    $resolvedVersion = Resolve-Version
    $asset = "frugal-$resolvedVersion-x86_64-pc-windows-msvc.zip"
    $url = "https://github.com/$RepoOwner/$RepoName/releases/download/$resolvedVersion/$asset"
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ("frugal-install-" + [System.Guid]::NewGuid().ToString("N"))

    New-Item -ItemType Directory -Path $tmpDir | Out-Null

    try {
        $zipPath = Join-Path $tmpDir $asset
        $extractDir = Join-Path $tmpDir "extract"

        Write-Host "Installing $BinName $resolvedVersion for x86_64-pc-windows-msvc..."
        Invoke-WebRequest -Uri $url -OutFile $zipPath
        Expand-Archive -Path $zipPath -DestinationPath $extractDir

        $binaryPath = Get-ChildItem -Path $extractDir -Recurse -Filter $BinName | Select-Object -First 1
        if (-not $binaryPath) {
            throw "could not find $BinName in extracted archive"
        }

        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Copy-Item $binaryPath.FullName (Join-Path $InstallDir $BinName) -Force

        Write-Host "Installed to $(Join-Path $InstallDir $BinName)"

        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if (-not ($userPath -split ";" | Where-Object { $_ -eq $InstallDir })) {
            Write-Warning "$InstallDir is not in PATH"
            Write-Host "Add this directory to your user PATH, then restart your terminal:"
            Write-Host "  $InstallDir"
        }
    }
    finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

Main
