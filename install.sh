#!/bin/sh
# Install prebuilt onrecord binary (and optional manpage) from GitHub Releases with SHA-256 verification.
#
# Options:
#   --version TAG   Release tag such as v0.1.0 (default: latest GitHub release)
#   --bin-dir DIR   Target directory for binary (default: $HOME/.local/bin)
#   --man-dir DIR   Optional directory for onrecord.1 manpage (e.g. $HOME/.local/share/man/man1)
#   -h, --help      Show usage and exit
#
# Environment variables:
#   ONRECORD_INSTALL_DIR  Target directory for binary (default: $HOME/.local/bin)
#   ONRECORD_MAN_DIR      Optional target directory for onrecord.1 manual page
#   ONRECORD_VERSION      Release tag (default: latest)
#   ONRECORD_REPO         GitHub owner/repo (default: david-engelmann/onrecord)
#   ONRECORD_BASE_URL     Override release download base URL (for offline mirrors/CI)

set -eu

REPO="${ONRECORD_REPO:-david-engelmann/onrecord}"
INSTALL_DIR="${ONRECORD_INSTALL_DIR:-$HOME/.local/bin}"
MAN_DIR="${ONRECORD_MAN_DIR:-}"
VERSION="${ONRECORD_VERSION:-latest}"

usage() {
    cat <<'EOF'
usage: install.sh [--version vX.Y.Z] [--bin-dir DIR] [--man-dir DIR]

Options:
  --version TAG   Release tag to install (defaults to latest GitHub release)
  --bin-dir DIR   Destination directory for the binary (default: $HOME/.local/bin)
  --man-dir DIR   Optional destination directory for the man(1) page
  -h, --help      Show this help message and exit
EOF
}

err() {
    printf 'onrecord-install: error: %s\n' "$1" >&2
    exit 1
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version)
            [ "$#" -ge 2 ] || err "--version requires a tag argument"
            VERSION="$2"
            shift 2
            ;;
        --bin-dir)
            [ "$#" -ge 2 ] || err "--bin-dir requires a directory argument"
            INSTALL_DIR="$2"
            shift 2
            ;;
        --man-dir)
            [ "$#" -ge 2 ] || err "--man-dir requires a directory argument"
            MAN_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            err "unknown argument: $1 (pass --help for usage)"
            ;;
    esac
done

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || err "required command not found: $1"
}

need_cmd uname
need_cmd tar
need_cmd awk
need_cmd mktemp

detect_target() {
    os_name="$(uname -s 2>/dev/null || echo unknown)"
    arch_name="$(uname -m 2>/dev/null || echo unknown)"

    case "$os_name" in
        Linux)
            if (ldd --version 2>&1 || true) | grep -qi musl || ls /lib/ld-musl-* >/dev/null 2>&1; then
                err "musl Linux detected; prebuilt Linux archives target glibc (gnu). Install from source with: cargo install --git https://github.com/${REPO}.git --locked"
            fi
            os_part="unknown-linux-gnu"
            ;;
        Darwin)
            os_part="apple-darwin"
            ;;
        *)
            err "unsupported operating system: $os_name (on Windows run install.ps1)"
            ;;
    esac

    case "$arch_name" in
        x86_64|amd64) arch_part="x86_64" ;;
        aarch64|arm64) arch_part="aarch64" ;;
        *) err "unsupported CPU architecture: $arch_name" ;;
    esac

    printf '%s-%s' "$arch_part" "$os_part"
}

download_file() {
    url="$1"
    dest="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$dest" "$url"
    else
        err "either curl or wget is required to download release archives"
    fi
}

resolve_latest_tag() {
    api_url="https://api.github.com/repos/${REPO}/releases/latest"
    if command -v curl >/dev/null 2>&1; then
        tag="$(curl -fsSL "$api_url" | awk -F'"' '/"tag_name"[[:space:]]*:/ { print $4; exit }')"
    elif command -v wget >/dev/null 2>&1; then
        tag="$(wget -qO- "$api_url" | awk -F'"' '/"tag_name"[[:space:]]*:/ { print $4; exit }')"
    else
        err "either curl or wget is required"
    fi
    [ -n "$tag" ] || err "could not resolve latest release tag from $api_url"
    printf '%s' "$tag"
}

compute_sha256() {
    file="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print tolower($1)}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{print tolower($1)}'
    else
        err "either sha256sum or shasum is required to verify SHA256SUMS"
    fi
}

TARGET="$(detect_target)"
ARCHIVE="onrecord-${TARGET}.tar.gz"

if [ -n "${ONRECORD_BASE_URL:-}" ]; then
    BASE_URL="${ONRECORD_BASE_URL%/}"
else
    if [ "$VERSION" = "latest" ] || [ -z "$VERSION" ]; then
        TAG="$(resolve_latest_tag)"
    else
        case "$VERSION" in
            v*) TAG="$VERSION" ;;
            *)  TAG="v$VERSION" ;;
        esac
    fi
    BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"
fi

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t onrecord-install)"
STAGED_BIN=""
cleanup() {
    rm -rf "$TMP_DIR"
    if [ -n "$STAGED_BIN" ] && [ -f "$STAGED_BIN" ]; then
        rm -f "$STAGED_BIN"
    fi
}
trap cleanup EXIT INT TERM

printf 'Downloading %s/%s\n' "$BASE_URL" "$ARCHIVE"
download_file "${BASE_URL}/${ARCHIVE}" "${TMP_DIR}/${ARCHIVE}"
download_file "${BASE_URL}/SHA256SUMS" "${TMP_DIR}/SHA256SUMS"

EXPECTED_SHA="$(awk -v name="$ARCHIVE" '
    NF >= 2 {
        file = $2
        sub(/^\*/, "", file)
        sub(/^\.\//, "", file)
        if (file == name) {
            print tolower($1)
            exit
        }
    }
' "${TMP_DIR}/SHA256SUMS")"

[ -n "$EXPECTED_SHA" ] || err "archive ${ARCHIVE} not listed in SHA256SUMS"

ACTUAL_SHA="$(compute_sha256 "${TMP_DIR}/${ARCHIVE}")"
if [ "$ACTUAL_SHA" != "$EXPECTED_SHA" ]; then
    err "SHA-256 mismatch for ${ARCHIVE}: expected ${EXPECTED_SHA}, got ${ACTUAL_SHA}"
fi

mkdir -p "${TMP_DIR}/unpack"
tar -xzf "${TMP_DIR}/${ARCHIVE}" -C "${TMP_DIR}/unpack"

if [ -f "${TMP_DIR}/unpack/onrecord" ]; then
    BIN_SRC="${TMP_DIR}/unpack/onrecord"
elif [ -f "${TMP_DIR}/unpack/onrecord-${TARGET}/onrecord" ]; then
    BIN_SRC="${TMP_DIR}/unpack/onrecord-${TARGET}/onrecord"
else
    err "onrecord binary not found inside ${ARCHIVE}"
fi

mkdir -p "$INSTALL_DIR"
STAGED_BIN="${INSTALL_DIR}/.onrecord.tmp.$$"
cp "$BIN_SRC" "$STAGED_BIN"
chmod 0755 "$STAGED_BIN"
mv -f "$STAGED_BIN" "${INSTALL_DIR}/onrecord"
STAGED_BIN=""

if [ -n "$MAN_DIR" ]; then
    if [ -f "${TMP_DIR}/unpack/man/onrecord.1" ]; then
        MAN_SRC="${TMP_DIR}/unpack/man/onrecord.1"
    elif [ -f "${TMP_DIR}/unpack/onrecord-${TARGET}/man/onrecord.1" ]; then
        MAN_SRC="${TMP_DIR}/unpack/onrecord-${TARGET}/man/onrecord.1"
    else
        MAN_SRC=""
    fi
    if [ -n "$MAN_SRC" ]; then
        mkdir -p "$MAN_DIR"
        cp "$MAN_SRC" "${MAN_DIR}/onrecord.1"
        chmod 0644 "${MAN_DIR}/onrecord.1"
    fi
fi

printf 'Verified SHA-256 (%s) and installed onrecord to %s/onrecord\n' "$ACTUAL_SHA" "$INSTALL_DIR"
