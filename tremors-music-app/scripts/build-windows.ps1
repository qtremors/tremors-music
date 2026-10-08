[CmdletBinding()]
param(
    [ValidateSet('Release')]
    [string]$Configuration = 'Release',
    [switch]$SkipTests
)

$ErrorActionPreference = 'Stop'
$appRoot = Split-Path -Parent $PSScriptRoot
$staging = $null
$repositoryRoot = Split-Path -Parent $appRoot
Push-Location $appRoot
try {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        throw 'Install Rust with rustup, then reopen Developer PowerShell for Visual Studio.'
    }
    if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
        throw 'Run this script in Developer PowerShell for Visual Studio 2022 with Desktop development with C++ installed.'
    }
    if (-not $SkipTests) {
        & cargo test --locked -p tremors-core --target x86_64-pc-windows-msvc
        if ($LASTEXITCODE -ne 0) { throw 'Core tests failed; no package was created.' }
    }
    $buildArguments = @('build', '--locked', '-p', 'tremors-music', '--target', 'x86_64-pc-windows-msvc')
    if ($Configuration -eq 'Release') { $buildArguments += '--release' }
    & cargo @buildArguments
    if ($LASTEXITCODE -ne 0) { throw 'Windows build failed; no package was created.' }

    $metadataJson = & cargo metadata --locked --no-deps --format-version 1
    if ($LASTEXITCODE -ne 0) { throw 'Could not read the application version.' }
    $metadata = $metadataJson | ConvertFrom-Json
    $appVersion = ($metadata.packages | Where-Object { $_.name -eq 'tremors-music' }).version
    $profileName = $Configuration.ToLowerInvariant()
    $packageName = "TremorsMusic-$appVersion-windows-x64-portable"
    $distRoot = Join-Path $appRoot 'dist'
    $staging = Join-Path $distRoot (".windows-package-" + [guid]::NewGuid().ToString('N'))
    $destination = Join-Path $staging $packageName
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    $executable = Join-Path $metadata.target_directory "x86_64-pc-windows-msvc\$profileName\tremors-music.exe"
    Copy-Item -LiteralPath $executable -Destination $destination
    foreach ($document in @('LICENSE.md', 'README.md', 'DEVELOPMENT.md', 'CHANGELOG.md', 'TASKS.md', 'PRIVACY.md')) {
        Copy-Item -LiteralPath (Join-Path $repositoryRoot $document) -Destination $destination
    }
    Copy-Item -LiteralPath (Join-Path $appRoot 'installer/licenses/NSIS.txt') -Destination (Join-Path $destination 'NSIS-LICENSES.txt')
    # Include the exact local sources with binaries, including uncommitted changes.
    $sourceFiles = @(& git -C $repositoryRoot ls-files --cached --others --exclude-standard)
    if ($LASTEXITCODE -ne 0) { throw 'Could not list corresponding source files.' }
    $sourceDirectory = Join-Path $staging 'source'
    foreach ($relativePath in $sourceFiles) {
        $sourceFile = Join-Path $repositoryRoot $relativePath
        if (-not (Test-Path -LiteralPath $sourceFile -PathType Leaf)) { continue }
        $copyPath = Join-Path $sourceDirectory $relativePath
        New-Item -ItemType Directory -Path (Split-Path -Parent $copyPath) -Force | Out-Null
        Copy-Item -LiteralPath $sourceFile -Destination $copyPath
    }
    Push-Location (Join-Path $sourceDirectory 'tremors-music-app')
    try {
        $vendorConfig = @(& cargo vendor --locked --versioned-dirs vendor)
        if ($LASTEXITCODE -ne 0) { throw 'Could not include dependency sources.' }
        $cargoConfig = Join-Path $sourceDirectory 'tremors-music-app/.cargo/config.toml'
        Add-Content -LiteralPath $cargoConfig -Value ("`n" + ($vendorConfig -join "`n")) -Encoding UTF8
    }
    finally { Pop-Location }
    # ZipFile preserves dot directories such as .cargo on every host.
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $sourceArchiveName = "TremorsMusic-$appVersion-windows-x64-source.zip"
    $sourceArchive = Join-Path $distRoot $sourceArchiveName
    if (Test-Path -LiteralPath $sourceArchive) { Remove-Item -LiteralPath $sourceArchive -Force }
    [System.IO.Compression.ZipFile]::CreateFromDirectory($sourceDirectory, $sourceArchive)
    $sourceHash = (Get-FileHash -LiteralPath $sourceArchive -Algorithm SHA256).Hash.ToLowerInvariant()
    "$sourceHash  $sourceArchiveName" | Set-Content -LiteralPath "$sourceArchive.sha256" -Encoding ASCII
    @"
Tremors Music $appVersion ($Configuration)

Run tremors-music.exe, then choose Add music folder.
No Python, Node.js, browser, backend service, or account is needed.

Music files are read only. App data is stored under LOCALAPPDATA using a
separate local data directory. TREMORS_DATA_DIR can override that location.
The executable is portable; library data is kept in your Windows user profile.

See DEVELOPMENT.md for the manual test checklists and dependency notices.
This build is unsigned.
Licensed under GNU GPL v3; see LICENSE.md. Complete corresponding application
source, build scripts and vendored dependency sources are available separately in
$sourceArchiveName on the same GitHub release. See SOURCE.txt.
Dependencies retain their own licenses; their notices are included with their sources.
"@ | Set-Content -LiteralPath (Join-Path $destination 'README.txt') -Encoding UTF8
    $installer = Join-Path $distRoot "TremorsMusic-$appVersion-windows-x64-setup.exe"
    & (Join-Path $PSScriptRoot 'build-installer.ps1') -PayloadDirectory $destination `
        -Version $appVersion -OutputPath $installer -SourceArchiveName $sourceArchiveName
    $archive = Join-Path $distRoot "$packageName.zip"
    $payloadFiles = Get-ChildItem -LiteralPath $destination -File
    Compress-Archive -LiteralPath $payloadFiles.FullName -DestinationPath $archive -Force
    $hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $packageName.zip" | Set-Content -LiteralPath "$archive.sha256" -Encoding ASCII
    Write-Host "Created $archive"
    Write-Host "Created $sourceArchive"
}
finally {
    Pop-Location
    if ($staging -and (Test-Path -LiteralPath $staging)) {
        $resolvedStaging = (Resolve-Path -LiteralPath $staging).Path
        $expectedDist = [System.IO.Path]::GetFullPath((Join-Path $appRoot 'dist'))
        if (-not $resolvedStaging.StartsWith($expectedDist + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw 'Refusing to remove staging outside the distribution directory.'
        }
        Remove-Item -LiteralPath $resolvedStaging -Recurse -Force
    }
}
