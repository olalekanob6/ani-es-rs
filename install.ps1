# Instalador de ani-es-rs para Windows
# Requiere PowerShell 5.1 o superior

param(
    [switch]$Help
)

if ($Help) {
    Write-Host "Instalador de ani-es-rs para Windows"
    Write-Host ""
    Write-Host "Uso: .\install.ps1 [opciones]"
    Write-Host ""
    Write-Host "Opciones:"
    Write-Host "  -Help    Muestra esta ayuda"
    Write-Host ""
    Write-Host "Este script instalará:"
    Write-Host "  - Rust (si no está instalado)"
    Write-Host "  - mpv (reproductor de video)"
    Write-Host "  - fzf (selector interactivo)"
    Write-Host "  - Compilará ani-es-rs en modo release"
    Write-Host ""
    exit 0
}

Write-Host "Instalador de ani-es-rs para Windows"
Write-Host "======================================"
Write-Host ""

# Detectar si estamos en WSL
if (Get-Command wsl.exe -ErrorAction SilentlyContinue) {
    $wsl = wsl.exe -l -v 2>&1 | Select-String "Ubuntu|Debian|Fedora|Arch"
    if ($wsl) {
        Write-Host "Detectado WSL. Redirigiendo al instalador de Linux..."
        Write-Host "Ejecuta: bash install.sh"
        exit 0
    }
}

# Verificar si está en Windows nativo
Write-Host "Sistema detectado: Windows"
Write-Host ""

# Verificar permisos de administrador
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "Se requieren permisos de administrador para instalar dependencias... (acepta el aviso UAC si aparece)"
    Write-Host ""
}

# Función para verificar si un comando existe
function Test-Command {
    param([string]$Command)
    $null -ne (Get-Command $Command -ErrorAction SilentlyContinue)
}

# Instalar dependencias
Write-Host "Verificando dependencias..."
Write-Host ""

# Verificar Rust
if (-not (Test-Command cargo)) {
    Write-Host "Rust no encontrado. Instalando Rust..."
    if (-not (Test-Command winget)) {
        Write-Host "winget no encontrado. Instala Rust manualmente desde:"
        Write-Host "https://rustup.rs/"
        exit 1
    }
    
    winget install --id Rustlang.Rustup --accept-source-agreements --accept-package-agreements
    Write-Host ""
    Write-Host "Por favor, reinicia tu terminal y ejecuta este script nuevamente."
    exit 0
} else {
    Write-Host "✓ Rust encontrado"
}

# Instalar mpv
if (-not (Test-Command mpv)) {
    Write-Host "Instalando mpv..."
    if (Test-Command winget) {
        winget install --id Gyan.FFmpeg --accept-source-agreements --accept-package-agreements
        # winget no instala mpv directamente, descargar manualmente
        Write-Host ""
        Write-Host "Por favor, descarga mpv desde: https://mpv.io/installation/"
        Write-Host "Y añade la ruta al PATH de Windows"
        Write-Host ""
    } else {
        Write-Host "Por favor, descarga mpv desde: https://mpv.io/installation/"
        Write-Host "Y añade la ruta al PATH de Windows"
        Write-Host ""
    }
} else {
    Write-Host "✓ mpv encontrado"
}

# Instalar fzf
if (-not (Test-Command fzf)) {
    Write-Host "Instalando fzf..."
    if (Test-Command winget) {
        winget install --id junegunn.fzf --accept-source-agreements --accept-package-agreements
    } else {
        Write-Host "Por favor, descarga fzf desde: https://github.com/junegunn/fzf"
        Write-Host "Y añade la ruta al PATH de Windows"
        Write-Host ""
    }
} else {
    Write-Host "✓ fzf encontrado"
}

# Verificar curl y jq
if (Test-Command curl) {
    Write-Host "✓ curl encontrado"
} else {
    Write-Host "⚠ curl no encontrado (puede estar incluido en Windows 10+)"
}

if (Test-Command jq) {
    Write-Host "✓ jq encontrado"
} else {
    Write-Host "⚠ jq no encontrado. Descárgalo desde: https://stedolan.github.io/jq/"
}

Write-Host ""
Write-Host "Compilando ani-es-rs..."
cargo build --release

Write-Host ""
Write-Host "¡Instalación completada!"
Write-Host ""
Write-Host "Ejecuta el binario con:"
Write-Host "  .\target\release\ani-es-rs.exe"
Write-Host ""
Write-Host "O añádelo al PATH con:"
Write-Host "  $env:Path += ';' + (Resolve-Path .\target\release)"
Write-Host ""
