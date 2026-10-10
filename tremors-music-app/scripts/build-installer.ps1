# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 Tremors and contributors
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$PayloadDirectory,
    [Parameter(Mandatory)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version,
    [Parameter(Mandatory)][string]$OutputPath,
    [Parameter(Mandatory)][string]$SourceArchiveName,
    [string]$Compiler = 'makensis'
)
$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$payload = (Resolve-Path -LiteralPath $PayloadDirectory).Path
if ([System.IO.Path]::GetFileName($SourceArchiveName) -ne $SourceArchiveName) {
    throw 'SourceArchiveName must be a filename alongside the installer.'
}
$required = @('tremors-music.exe', 'LICENSE.md', 'README.md', 'DEVELOPMENT.md', 'CHANGELOG.md', 'TASKS.md', 'PRIVACY.md', 'NSIS-LICENSES.txt')
foreach ($file in $required) {
    if (-not (Test-Path -LiteralPath (Join-Path $payload $file) -PathType Leaf)) {
        throw "Missing installer file: $file"
    }
}
# A portable Windows debug binary cannot be installed: verify x64 GUI PE headers.
$binary = [System.IO.File]::ReadAllBytes((Join-Path $payload 'tremors-music.exe'))
if ($binary.Length -lt 256 -or $binary[0] -ne 0x4D -or $binary[1] -ne 0x5A) { throw 'Invalid PE executable.' }
$pe = [BitConverter]::ToInt32($binary, 0x3C)
if ($pe -lt 0 -or $pe + 94 -gt $binary.Length -or
    [BitConverter]::ToUInt32($binary, $pe) -ne 0x4550 -or
    [BitConverter]::ToUInt16($binary, $pe + 4) -ne 0x8664 -or
    [BitConverter]::ToUInt16($binary, $pe + 92) -ne 2) {
    throw 'Installer requires an x64 Windows GUI release executable.'
}
$command = Get-Command $Compiler -ErrorAction SilentlyContinue
if (-not $command -and $env:OS -eq 'Windows_NT') {
    $Compiler = Join-Path ${env:ProgramFiles(x86)} 'NSIS\makensis.exe'
    $command = Get-Command $Compiler -ErrorAction SilentlyContinue
}
if (-not $command) { throw 'Install NSIS 3.11+ and add makensis to PATH, or pass -Compiler.' }
$output = [System.IO.Path]::GetFullPath($OutputPath)
New-Item -ItemType Directory -Path (Split-Path -Parent $output) -Force | Out-Null
@"
Tremors Music $Version is licensed GPL-3.0-only; see LICENSE.md.
Complete corresponding source for this build is available separately in:
$SourceArchiveName
Download it from the same release:
https://github.com/qtremors/tremors-music/releases/tag/v$Version

That archive contains the application, dependency sources and notices, build
scripts, lockfile and shaders. Publish this matching source archive alongside
binary downloads at no extra charge when redistributing under GPL v3.
Project: https://github.com/qtremors/tremors-music
"@ | Set-Content -LiteralPath (Join-Path $payload 'SOURCE.txt') -Encoding UTF8
$size = [int][Math]::Ceiling(($required | ForEach-Object {
    (Get-Item -LiteralPath (Join-Path $payload $_)).Length
} | Measure-Object -Sum).Sum / 1024)
$switch = if ($env:OS -eq 'Windows_NT') { '/D' } else { '-D' }
& $command.Source "${switch}APP_VERSION=$Version" "${switch}PAYLOAD_DIR=$payload" `
    "${switch}REPOSITORY_ROOT=$repositoryRoot" "${switch}OUTPUT_FILE=$output" `
    "${switch}INSTALLED_SIZE_KB=$size" (Join-Path (Split-Path -Parent $PSScriptRoot) 'installer/windows.nsi')
if ($LASTEXITCODE -ne 0) { throw 'NSIS installer compilation failed.' }
$hash = (Get-FileHash -LiteralPath $output -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $([System.IO.Path]::GetFileName($output))" | Set-Content "$output.sha256" -Encoding ASCII
Write-Host "Created $output"
