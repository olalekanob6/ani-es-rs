# Instalador de ani-es-rs para Windows (PowerShell 5.1+)
[CmdletBinding()]
param(
    [switch]$Help,
    [switch]$FromSource,
    [switch]$Interactive,
    [switch]$Uninstall,
    [string]$Version = 'latest',
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\ani-es-rs')
)

$ErrorActionPreference = 'Stop'
$repo = 'olalekanob6/ani-es-rs'

if ($Help) {
    $helpText = @'
ani-es-rs — Watch anime from your terminal

Uso: .\install.ps1 [-Version <vX.Y.Z>] [-InstallDir <carpeta>]
                    [-FromSource] [-Interactive] [-Uninstall] [-Help]

Por defecto descarga el binario oficial de GitHub Releases y comprueba SHA-256.
-FromSource compila con Rust/Cargo. No instala dependencias ni edita PATH.
mpv es necesario para reproducir; fzf es opcional para los menús interactivos.
'@
    Write-Host $helpText
    exit 0
}

Write-Host '╭────────────────────────────────────────────╮' -ForegroundColor Cyan
Write-Host '│ ani-es-rs — Watch anime from your terminal │' -ForegroundColor Cyan
Write-Host '╰────────────────────────────────────────────╯' -ForegroundColor Cyan

if ($Interactive -and -not $Uninstall) {
    if ([Console]::IsInputRedirected -or [Console]::IsOutputRedirected) {
        throw '-Interactive requiere una terminal interactiva.'
    }
    Write-Host ''
    Write-Host '  1) Instalar la última versión estable'
    Write-Host '  2) Elegir versión'
    Write-Host '  3) Desinstalar'
    Write-Host '  0) Cancelar'
    $choice = Read-Host 'Opción [1]'
    switch ($choice) {
        { $_ -eq '' -or $_ -eq '1' } { break }
        '2' { $Version = Read-Host 'Versión (por ejemplo, v0.1.3)'; break }
        '3' { $Uninstall = $true; break }
        '0' { Write-Host 'Cancelado.'; exit 0 }
        default { throw 'Opción no válida.' }
    }
}

$destination = Join-Path $InstallDir 'ani-es-rs.exe'
if ($Uninstall) {
    if (Test-Path -LiteralPath $destination -PathType Leaf) {
        Remove-Item -LiteralPath $destination -Force
        Write-Host "Desinstalado: $destination" -ForegroundColor Green
    } else {
        Write-Host "No encontré el ejecutable en $destination"
    }
    exit 0
}

$missing = @()
if (-not (Get-Command mpv -ErrorAction SilentlyContinue)) { $missing += 'mpv' }
if (-not (Get-Command fzf -ErrorAction SilentlyContinue)) { $missing += 'fzf' }
if ($missing.Count -gt 0) {
    Write-Host "Dependencias no detectadas: $($missing -join ', ')"
    Write-Host 'mpv es necesario para reproducir; fzf solo para menús interactivos.'
    Write-Host 'No se instalarán programas automáticamente.'
} else {
    Write-Host 'Dependencias detectadas: mpv y fzf.'
}

$tempDir = Join-Path ([IO.Path]::GetTempPath()) ('ani-es-rs-install-' + [Guid]::NewGuid().ToString('N'))
$staged = $null
New-Item -ItemType Directory -Path $tempDir | Out-Null
try {
    if ($FromSource) {
        $repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
        $cargo = Get-Command cargo -ErrorAction SilentlyContinue
        if (-not $cargo) { throw 'Cargo no está instalado. Instala Rust desde https://rustup.rs/ o usa el instalador de release.' }
        Push-Location $repoRoot
        try {
            Write-Host 'Compilando desde el código fuente...'
            & cargo build --release --locked
            if ($LASTEXITCODE -ne 0) { throw "La compilación falló con código $LASTEXITCODE." }
            $sourceBinary = Join-Path $repoRoot 'target\release\ani-es-rs.exe'
            if (-not (Test-Path -LiteralPath $sourceBinary -PathType Leaf)) { throw "No se encontró el ejecutable compilado: $sourceBinary" }
            Copy-Item -LiteralPath $sourceBinary -Destination (Join-Path $tempDir 'ani-es-rs.exe')
        } finally {
            Pop-Location
        }
        $installedVersion = 'source'
    } else {
        $architecture = if ($env:PROCESSOR_ARCHITEW6432) { $env:PROCESSOR_ARCHITEW6432 } else { $env:PROCESSOR_ARCHITECTURE }
        if ($architecture -ne 'AMD64') {
            throw "No hay binario Windows x86_64 publicado para la arquitectura del sistema $architecture."
        }
        if ($Version -ne 'latest' -and $Version -notmatch '^v?\d+\.\d+\.\d+$') {
            throw "Versión estable no válida: $Version"
        }
        if ($Version -ne 'latest' -and -not $Version.StartsWith('v')) { $Version = "v$Version" }

        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        if ($Version -eq 'latest') {
            $releaseUri = "https://api.github.com/repos/$repo/releases/latest"
        } else {
            $releaseUri = "https://api.github.com/repos/$repo/releases/tags/$([Uri]::EscapeDataString($Version))"
        }
        Write-Host "Consultando release: $Version"
        $release = Invoke-RestMethod -Method Get -Uri $releaseUri -Headers @{ 'User-Agent' = 'ani-es-rs-installer' } -TimeoutSec 30
        $tag = [string]$release.tag_name
        if ($tag -notmatch '^v\d+\.\d+\.\d+$') { throw "Tag de release inesperado: $tag" }
        $assetName = 'ani-es-rs-windows-x86_64.exe'
        $binaryAsset = @($release.assets | Where-Object { $_.name -ceq $assetName })
        $sumAsset = @($release.assets | Where-Object { $_.name -ceq 'SHA256SUMS' })
        if ($binaryAsset.Count -ne 1 -or $sumAsset.Count -ne 1) { throw "La release $tag no contiene exactamente el binario y SHA256SUMS requeridos." }
        foreach ($asset in @($binaryAsset[0], $sumAsset[0])) {
            if ([long]$asset.size -gt 134217728) { throw "Asset demasiado grande: $($asset.name)" }
            $assetUri = [Uri]$asset.browser_download_url
            if ($assetUri.Scheme -ne 'https' -or $assetUri.Host -ne 'github.com') { throw "URL de asset no confiable: $($asset.browser_download_url)" }
        }

        $sumPath = Join-Path $tempDir 'SHA256SUMS'
        $binaryPath = Join-Path $tempDir $assetName
        Write-Host "Descargando ani-es-rs $tag para Windows x86_64..."
        Invoke-WebRequest -UseBasicParsing -Uri $sumAsset[0].browser_download_url -OutFile $sumPath -TimeoutSec 60
        Invoke-WebRequest -UseBasicParsing -Uri $binaryAsset[0].browser_download_url -OutFile $binaryPath -TimeoutSec 300

        if ((Get-Item -LiteralPath $binaryPath).Length -gt 134217728) { throw 'El binario excede el tamaño máximo permitido.' }
        $expected = $null
        $matchCount = 0
        foreach ($line in Get-Content -LiteralPath $sumPath -Encoding ASCII) {
            if ($line -match '^\s*([0-9a-fA-F]{64})\s+\*?(.+?)\s*$' -and $Matches[2] -ceq $assetName) {
                $expected = $Matches[1].ToLowerInvariant()
                $matchCount++
            }
        }
        if ($matchCount -ne 1) { throw "SHA256SUMS no contiene exactamente una suma válida para $assetName." }
        $actual = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -cne $expected) { throw 'La verificación SHA-256 falló; no se instalará el ejecutable.' }
        Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $tempDir 'ani-es-rs.exe')
        $installedVersion = $tag
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $staged = Join-Path $InstallDir ('.ani-es-rs.exe.' + [Guid]::NewGuid().ToString('N') + '.tmp')
    Copy-Item -LiteralPath (Join-Path $tempDir 'ani-es-rs.exe') -Destination $staged
    if ([IO.File]::Exists($destination)) {
        [IO.File]::Replace($staged, $destination, $null)
    } else {
        [IO.File]::Move($staged, $destination)
    }
    $staged = $null
    Write-Host ''
    Write-Host 'Instalación completada.' -ForegroundColor Green
    Write-Host "Versión: $installedVersion"
    Write-Host "Ejecutable: $destination"
    if (($env:PATH -split ';') -notcontains $InstallDir) {
        Write-Host "Añade esta carpeta a tu PATH de usuario si quieres ejecutar ani-es-rs desde cualquier terminal: $InstallDir"
    }
} finally {
    if ($staged -and (Test-Path -LiteralPath $staged)) { Remove-Item -LiteralPath $staged -Force -ErrorAction SilentlyContinue }
    Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}
