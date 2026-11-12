#!/usr/bin/env bash
# Instalador de Jiyu Launcher para Linux.
#
#   ./instalar.sh               instala o actualiza (pregunta antes de instalar paquetes del sistema)
#   ./instalar.sh --si          igual, sin preguntar
#   ./instalar.sh --sin-horizon no baja Horizon aunque falte
#   ./instalar.sh --desinstalar quita el launcher (tus instancias, ajustes y skins no se tocan)
#
# Qué hace, por orden:
#   1. Instala lo que falte del sistema: git, curl, un compilador de C y las bibliotecas de ventana
#      (Wayland, X11, xkbcommon, Vulkan). Usa pacman, apt, dnf o zypper.
#   2. Instala Rust con rustup si no tienes cargo (solo para tu usuario, en ~/.cargo).
#   3. Compila el launcher en modo release.
#   4. Lo copia a ~/.local/bin/jiyu-launcher y lo pone en el menú de aplicaciones.
#   5. Si no tienes Horizon, baja la última versión oficial de github.com/horizon-rust/horizon,
#      comprueba su SHA256 y la deja en ~/Applications/horizon-launcher-linux-x86_64/.
set -euo pipefail

SI=0
CON_HORIZON=1
DESINSTALAR=0
for a in "$@"; do
  case "$a" in
    --si|-y) SI=1 ;;
    --sin-horizon) CON_HORIZON=0 ;;
    --desinstalar) DESINSTALAR=1 ;;
    -h|--help) sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "No conozco la opción «$a». Mira ./instalar.sh --help" >&2; exit 2 ;;
  esac
done

BIN_DIR="${XDG_BIN_HOME:-$HOME/.local/bin}"
DATOS="${XDG_DATA_HOME:-$HOME/.local/share}"
APPS="$DATOS/applications"
ICONOS="$DATOS/icons/hicolor/scalable/apps"
HORIZON_DIR="$HOME/Applications/horizon-launcher-linux-x86_64"
REPO="https://github.com/ToxBox8/jiyu-launcher.git"

c_ok=$'\e[32m'; c_av=$'\e[33m'; c_mal=$'\e[31m'; c_n=$'\e[1m'; c_0=$'\e[0m'
[ -t 1 ] || { c_ok=; c_av=; c_mal=; c_n=; c_0=; }
paso() { printf '%s==>%s %s\n' "$c_n" "$c_0" "$*"; }
bien() { printf '  %s✓%s %s\n' "$c_ok" "$c_0" "$*"; }
aviso() { printf '  %s!%s %s\n' "$c_av" "$c_0" "$*"; }
falla() { printf '%s✗ %s%s\n' "$c_mal" "$*" "$c_0" >&2; exit 1; }
pregunta() {
  [ "$SI" = 1 ] && return 0
  local r
  read -r -p "  $1 [S/n] " r </dev/tty || r=s
  [[ -z "$r" || "$r" =~ ^[sSyY] ]]
}

# ---------- Desinstalar ----------
if [ "$DESINSTALAR" = 1 ]; then
  paso "Quitando Jiyu Launcher"
  rm -f "$BIN_DIR/jiyu-launcher" "$APPS/jiyu-launcher.desktop" "$ICONOS/jiyu-launcher.svg"
  command -v update-desktop-database >/dev/null && update-desktop-database "$APPS" 2>/dev/null || true
  bien "Hecho. Tus ajustes siguen en ~/.config/jiyu-launcher y tus skins en $DATOS/jiyu-launcher."
  bien "Horizon y Prism no se han tocado."
  exit 0
fi

[ "$(uname -s)" = Linux ] || falla "Este instalador es para Linux."
[ "$(uname -m)" = x86_64 ] || aviso "Solo se ha probado en x86_64; sigo, pero Horizon puede no tener versión para $(uname -m)."

# ---------- 1. Paquetes del sistema ----------
paso "Comprobando lo que hace falta del sistema"
falta=()
for p in git curl cc sha1sum; do command -v "$p" >/dev/null || falta+=("$p"); done
tiene_lib() {
  # Sin «grep -q»: con pipefail, cortar la tubería pronto haría fallar la comprobación.
  { ldconfig -p 2>/dev/null || /sbin/ldconfig -p 2>/dev/null || true; } | grep -F "$1" >/dev/null
}
for l in libwayland-client.so libxkbcommon.so libX11.so libXcursor.so libXi.so libXrandr.so libvulkan.so; do
  tiene_lib "$l" || falta+=("$l")
done

if [ "${#falta[@]}" -eq 0 ]; then
  bien "Está todo"
else
  aviso "Falta: ${falta[*]}"
  sudo_=""; [ "$(id -u)" -ne 0 ] && sudo_="sudo"
  if command -v pacman >/dev/null; then
    cmd=($sudo_ pacman -S --needed --noconfirm git curl base-devel wayland libxkbcommon libx11 libxcursor libxi libxrandr vulkan-icd-loader coreutils)
  elif command -v apt-get >/dev/null; then
    cmd=($sudo_ apt-get install -y git curl build-essential libwayland-client0 libxkbcommon0 libx11-6 libxcursor1 libxi6 libxrandr2 libvulkan1 coreutils)
  elif command -v dnf >/dev/null; then
    cmd=($sudo_ dnf install -y git curl gcc wayland-devel libxkbcommon libX11 libXcursor libXi libXrandr vulkan-loader coreutils)
  elif command -v zypper >/dev/null; then
    cmd=($sudo_ zypper install -y git curl gcc libwayland-client0 libxkbcommon0 libX11-6 libXcursor1 libXi6 libXrandr2 libvulkan1 coreutils)
  else
    falla "No reconozco tu gestor de paquetes. Instala a mano: ${falta[*]}"
  fi
  echo "  Voy a ejecutar: ${cmd[*]}"
  if pregunta "¿Instalo esos paquetes?"; then
    if "${cmd[@]}"; then bien "Paquetes instalados"; else aviso "No se pudieron instalar; sigo y te aviso si algo falla."; fi
  else
    aviso "Sigo sin instalarlos: si falla la compilación o no abre la ventana, es por esto."
  fi
fi
command -v zenity >/dev/null || aviso "Sin zenity no podrás elegir archivos de skin con diálogo (opcional)."

# ---------- 2. Rust ----------
paso "Comprobando Rust"
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
if command -v cargo >/dev/null; then
  bien "$(cargo --version)"
else
  aviso "No tienes Rust."
  pregunta "¿Instalo Rust con rustup (solo para tu usuario, en ~/.cargo)?" || falla "Hace falta Rust para compilar."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
  . "$HOME/.cargo/env"
  bien "$(cargo --version)"
fi

# ---------- 3. El código y la compilación ----------
paso "Compilando Jiyu Launcher"
AQUI="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || true)"
if [ -n "$AQUI" ] && [ -f "$AQUI/Cargo.toml" ] && grep -q 'name = "jiyu-launcher"' "$AQUI/Cargo.toml"; then
  FUENTE="$AQUI"
else
  # Ejecutado suelto (curl … | bash): se clona en ~/.local/share/jiyu-launcher/codigo.
  FUENTE="$DATOS/jiyu-launcher/codigo"
  if [ -d "$FUENTE/.git" ]; then
    git -C "$FUENTE" pull --ff-only
  else
    mkdir -p "$(dirname "$FUENTE")"
    git clone --depth 1 "$REPO" "$FUENTE" || falla "No pude clonar $REPO (si el repo es privado, inicia sesión con «gh auth login»)."
  fi
fi
( cd "$FUENTE" && cargo build --release --locked 2>&1 | tail -n 3 ) || falla "La compilación falló: mira el error de arriba."
bien "Compilado"

# ---------- 4. Instalar en tu usuario ----------
paso "Instalando"
mkdir -p "$BIN_DIR" "$APPS" "$ICONOS"
install -m 755 "$FUENTE/target/release/jiyu-launcher" "$BIN_DIR/jiyu-launcher"
install -m 644 "$FUENTE/assets/escritorio/jiyu-launcher.svg" "$ICONOS/jiyu-launcher.svg"
sed "s#@BIN@#$BIN_DIR/jiyu-launcher#" "$FUENTE/assets/escritorio/jiyu-launcher.desktop" > "$APPS/jiyu-launcher.desktop"
command -v update-desktop-database >/dev/null && update-desktop-database "$APPS" 2>/dev/null || true
bien "$BIN_DIR/jiyu-launcher"
bien "En el menú de aplicaciones como «Jiyu Launcher»"
case ":$PATH:" in *":$BIN_DIR:"*) ;; *) aviso "$BIN_DIR no está en tu PATH: ábrelo desde el menú o añade esa carpeta al PATH." ;; esac

# ---------- 5. Horizon ----------
paso "Comprobando Horizon"
if command -v horizon-launcher >/dev/null || [ -x "$HORIZON_DIR/horizon-launcher" ] || [ -x "$HOME/.local/bin/horizon-launcher" ]; then
  bien "Ya lo tienes"
elif [ "$CON_HORIZON" = 0 ]; then
  aviso "No lo bajo (--sin-horizon). Sin Horizon solo verás tus instancias de Prism."
elif pregunta "No tienes Horizon (hace falta para jugar). ¿Lo bajo de github.com/horizon-rust/horizon?"; then
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  tag="$(curl -fsSL "https://api.github.com/repos/horizon-rust/horizon/releases?per_page=30" \
        | grep -o '"tag_name": *"launcher@[^"]*"' | head -n1 | sed 's/.*"\(launcher@[^"]*\)"/\1/')"
  [ -n "$tag" ] || falla "No encontré la última versión de Horizon."
  base="https://github.com/horizon-rust/horizon/releases/download/${tag//@/%40}"
  echo "  Bajando $tag…"
  curl -fL --progress-bar -o "$tmp/horizon.tar.gz" "$base/horizon-launcher-linux-x86_64.tar.gz"
  curl -fsSL -o "$tmp/SHA256SUMS" "$base/SHA256SUMS"
  esperado="$(grep ' horizon-launcher-linux-x86_64.tar.gz$' "$tmp/SHA256SUMS" | awk '{print $1}')"
  real="$(sha256sum "$tmp/horizon.tar.gz" | awk '{print $1}')"
  [ -n "$esperado" ] && [ "$esperado" = "$real" ] || falla "La suma SHA256 de Horizon no coincide: no lo instalo."
  bien "Suma SHA256 correcta"
  mkdir -p "$HORIZON_DIR"
  tar -xzf "$tmp/horizon.tar.gz" -C "$tmp"
  hl="$(find "$tmp" -type f -name horizon-launcher | head -n1)"
  [ -n "$hl" ] || falla "El paquete de Horizon no trae horizon-launcher."
  cp -r "$(dirname "$hl")"/. "$HORIZON_DIR"/
  chmod +x "$HORIZON_DIR/horizon-launcher"
  bien "Horizon $tag en $HORIZON_DIR"
  aviso "Para jugar necesitas una cuenta de Microsoft con Minecraft: Java Edition (inicia sesión desde Cuentas)."
else
  aviso "Sin Horizon solo verás tus instancias de Prism."
fi

echo
printf '%sListo.%s Ábrelo desde el menú o con: %sjiyu-launcher%s\n' "$c_ok" "$c_0" "$c_n" "$c_0"
