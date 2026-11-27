#!/bin/sh
# Install a prebuilt onrecord release binary with SHA-256 verification.
set -eu

REPO="${ONRECORD_REPO:-david-engelmann/onrecord}"
INSTALL_DIR="${ONRECORD_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${ONRECORD_VERSION:-latest}"

need_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "error: required command '$1' not found on PATH" >&2
        exit 1
    fi
}

need_cmd uname
need_cmd tar
need_cmd mktemp

fetch() {
    url="$1"
    dest="$2"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$dest" "$url"
    else
        echo "error: curl or wget is required to download releases" >&2
        exit 1
    fi
}

sha256_file() {
    file="$1"
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        echo "error: sha256sum or shasum is required to verify SHA256SUMS" >&2
        exit 1
    fi
}

os_name="$(uname -s)"
arch_name="$(uname -m)"

case "$os_name" in
    Linux) os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    *)
        echo "error: unsupported operating system '$os_name' (use Windows .zip release or build from source)" >&2
        exit 1
        ;;
esac

case "$arch_name" in
    x86_64|amd64) arch_part="x86_64" ;;
    aarch64|arm64) arch_part="aarch64" ;;
    *)
        echo "error: unsupported architecture '$arch_name'" >&2
        exit 1
        ;;
esac

target="${arch_part}-${os_part}"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT INT TERM

if [ "$VERSION" = "latest" ]; then
    api_json="$tmpdir/release.json"
    fetch "https://api.github.com/repos/${REPO}/releases/latest" "$api_json"
    tag="$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$api_json" | head -n 1)"
    if [ -z "$tag" ]; then
        echo "error: could not determine latest release tag from GitHub" >&2
        exit 1
    fi
else
    tag="$VERSION"
fi

archive="onrecord-${tag}-${target}.tar.gz"
base_url="https://github.com/${REPO}/releases/download/${tag}"

fetch "${base_url}/${archive}" "${tmpdir}/${archive}"
fetch "${base_url}/SHA256SUMS" "${tmpdir}/SHA256SUMS"

expected="$(awk -v name="$archive" '$2 == name || $2 == "*" name {print $1; exit}' "${tmpdir}/SHA256SUMS")"
if [ -z "$expected" ]; then
    echo "error: ${archive} not listed in SHA256SUMS" >&2
    exit 1
fi

actual="$(sha256_file "${tmpdir}/${archive}")"
if [ "$actual" != "$expected" ]; then
    echo "error: SHA-256 mismatch for ${archive} (expected ${expected}, got ${actual})" >&2
    exit 1
fi

tar -xzf "${tmpdir}/${archive}" -C "$tmpdir"
mkdir -p "$INSTALL_DIR"
cp "${tmpdir}/onrecord-${tag}-${target}/onrecord" "${INSTALL_DIR}/onrecord"
chmod 0755 "${INSTALL_DIR}/onrecord"

echo "Installed onrecord ${tag} (${target}) to ${INSTALL_DIR}/onrecord"
