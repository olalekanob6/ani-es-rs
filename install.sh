#!/bin/bash
set -e

echo "Instalador de ani-es-rs"
echo "========================"
echo ""

# Detectar sistema operativo
OS="unknown"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    OS="windows"
fi

echo "Sistema detectado: $OS"
echo ""

# Función para instalar en Linux
install_linux() {
    echo "Instalando dependencias en Linux..."
    
    # Detectar gestor de paquetes
    if command -v apt-get &> /dev/null; then
        PKG_MANAGER="apt"
    elif command -v dnf &> /dev/null; then
        PKG_MANAGER="dnf"
    elif command -v yum &> /dev/null; then
        PKG_MANAGER="yum"
    elif command -v zypper &> /dev/null; then
        PKG_MANAGER="zypper"
    elif command -v pacman &> /dev/null; then
        PKG_MANAGER="pacman"
    else
        echo "Gestor de paquetes no soportado."
        echo "Instala manualmente: mpv, fzf, curl, jq"
        exit 1
    fi
    
    echo "Gestor de paquetes detectado: $PKG_MANAGER"
    echo ""
    
    # Actualizar paquetes
    case $PKG_MANAGER in
        apt)
            echo "Se requiere contraseña de sudo para instalar dependencias... (escribe tu contraseña y presiona Enter)" && sudo apt-get update -y
            sudo apt-get install -y mpv fzf curl jq
            ;;
        dnf)
            echo "Se requiere contraseña de sudo para instalar dependencias... (escribe tu contraseña y presiona Enter)" && sudo dnf install -y mpv fzf curl jq
            ;;
        yum)
            echo "Se requiere contraseña de sudo para instalar dependencias... (escribe tu contraseña y presiona Enter)" && sudo yum install -y mpv fzf curl jq
            ;;
        zypper)
            echo "Se requiere contraseña de sudo para instalar dependencias... (escribe tu contraseña y presiona Enter)" && sudo zypper install -y mpv fzf curl jq
            ;;
        pacman)
            echo "Se requiere contraseña de sudo para instalar dependencias... (escribe tu contraseña y presiona Enter)" && sudo pacman -S --noconfirm mpv fzf curl jq
            ;;
    esac
    
    echo ""
    echo "Dependencias instaladas correctamente."
}

# Función para instalar en macOS
install_macos() {
    echo "Instalando dependencias en macOS..."
    
    if ! command -v brew &> /dev/null; then
        echo "Homebrew no encontrado. Instálalo desde https://brew.sh"
        exit 1
    fi
    
    brew install mpv fzf curl jq
    echo ""
    echo "Dependencias instaladas correctamente."
}

# Función para instalar en Windows (nativo, no WSL)
install_windows() {
    echo "Instalando dependencias en Windows..."
    echo ""
    echo "Por favor, instala manualmente:"
    echo "1. mpv: https://mpv.io/installation/"
    echo "   - Descarga el binario y añade la ruta al PATH"
    echo "   - O usa: winget install mpv"
    echo ""
    echo "2. fzf: https://github.com/junegunn/fzf"
    echo "   - Descarga el binario y añade la ruta al PATH"
    echo "   - O usa: winget install junegunn.fzf"
    echo ""
    echo "3. curl y jq: suelen estar incluidos en Windows 10+"
    echo "   - Si no, instálalos desde: https://curl.se/windows/ y https://stedolan.github.io/jq/"
    echo ""
    echo "Después de instalar las dependencias, ejecuta:"
    echo "  cargo build --release"
    echo ""
}

# Instalar según el sistema
case $OS in
    linux)
        install_linux
        ;;
    macos)
        install_macos
        ;;
    windows)
        install_windows
        ;;
    *)
        echo "Sistema operativo no detectado: $OSTYPE"
        echo "Instala manualmente: mpv, fzf, curl, jq"
        exit 1
        ;;
esac

# Compilar el proyecto
echo ""
echo "Compilando ani-es-rs..."
if command -v cargo &> /dev/null; then
    cargo build --release
    echo ""
    echo "¡Instalación completada!"
    echo ""
    echo "Ejecuta el binario con:"
    echo "  ./target/release/ani-es-rs"
    echo ""
    echo "O instálalo en el PATH con:"
    echo "  sudo cp target/release/ani-es-rs /usr/local/bin/"
else
    echo "Cargo no encontrado. Instala Rust desde: https://rustup.rs/"
    exit 1
fi
