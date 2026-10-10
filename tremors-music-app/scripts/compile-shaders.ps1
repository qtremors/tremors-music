# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 Tremors and contributors
[CmdletBinding()]
param([string]$Compiler = $env:GPUI_FXC_PATH)
$ErrorActionPreference = 'Stop'
if (-not $Compiler) {
    $sdkRoot = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
    $candidates = @(Get-ChildItem $sdkRoot -Filter fxc.exe -Recurse |
        Where-Object { $_.Directory.Name -eq 'x64' -and $_.Directory.Parent.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
        Sort-Object { [version]$_.Directory.Parent.Name } -Descending)
    if ($candidates.Count -eq 0) { throw 'Install Windows SDK fxc.exe, or pass -Compiler.' }
    $Compiler = $candidates[0].FullName
}
$root = Join-Path (Split-Path -Parent $PSScriptRoot) 'third-party/gpui-pre-windows'
$output = Join-Path $root 'shaders'
New-Item -ItemType Directory -Path $output -Force | Out-Null
$modules = @('quad', 'shadow', 'path_rasterization', 'path_sprite', 'underline',
    'monochrome_sprite', 'subpixel_sprite', 'polychrome_sprite', 'emoji_rasterization')
foreach ($module in $modules) {
    $sourceName = if ($module -eq 'emoji_rasterization') { 'color_text_raster.hlsl' } else { 'shaders.hlsl' }
    foreach ($stage in @(@('vs', 'vertex'), @('ps', 'fragment'))) {
        $path = Join-Path $output "$($module)_$($stage[0]).cso"
        & $Compiler /nologo /O3 /T "$($stage[0])_4_1" /E "$($module)_$($stage[1])" /Fo $path (Join-Path $root "src/$sourceName")
        if ($LASTEXITCODE -ne 0) { throw "Shader compilation failed: $module $($stage[0])" }
    }
}
Get-ChildItem $output -Filter '*.cso' | Sort-Object Name | ForEach-Object {
    "$((Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant())  $($_.Name)"
} | Set-Content (Join-Path $output 'SHA256SUMS') -Encoding ASCII
Write-Host 'Regenerated 18 release shaders. Review the bytecode and SHA256SUMS changes.'
