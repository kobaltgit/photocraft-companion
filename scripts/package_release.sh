#!/usr/bin/env bash
# PhotoCraft Companion - Multiplatform packaging script (Linux & macOS)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
VERSION="1.0.3"
DIST_DIR="${ROOT_DIR}/dist"

cd "${ROOT_DIR}"

echo ">>> Building release binary (photocraft-companion)..."
cargo build --package companion_app --release

echo ">>> Building plugin registry and .pcpl packages..."
cargo run --package plugin_validator -- --validate plugins --build-registry --package-plugins || true

mkdir -p "${DIST_DIR}"
OS_NAME="$(uname -s)"

if [ "${OS_NAME}" = "Darwin" ]; then
    echo ">>> Packaging for macOS..."
    APP_BUNDLE="${DIST_DIR}/PhotoCraft Companion.app"
    rm -rf "${APP_BUNDLE}"
    mkdir -p "${APP_BUNDLE}/Contents/MacOS"
    mkdir -p "${APP_BUNDLE}/Contents/Resources"

    cp target/release/photocraft-companion "${APP_BUNDLE}/Contents/MacOS/"
    cp packaging/macos/Info.plist "${APP_BUNDLE}/Contents/"
    cp -r plugins "${APP_BUNDLE}/Contents/Resources/"
    if [ -f "${DIST_DIR}/registry.json" ]; then
        cp "${DIST_DIR}/registry.json" "${APP_BUNDLE}/Contents/Resources/" || true
    fi

    # Portable tar.gz
    PORTABLE_ARCHIVE="${DIST_DIR}/PhotoCraft-Companion-v${VERSION}-macos-portable.tar.gz"
    tar -czf "${PORTABLE_ARCHIVE}" -C "${DIST_DIR}" "PhotoCraft Companion.app"
    echo ">>> macOS Portable archive created: ${PORTABLE_ARCHIVE}"

    # DMG installer if hdiutil is available
    if command -v hdiutil >/dev/null 2>&1; then
        DMG_PATH="${DIST_DIR}/PhotoCraft-Companion-v${VERSION}-macos.dmg"
        hdiutil create -volname "PhotoCraft Companion" -srcfolder "${APP_BUNDLE}" -ov -format UDZO "${DMG_PATH}"
        echo ">>> macOS DMG installer created: ${DMG_PATH}"
    fi

elif [ "${OS_NAME}" = "Linux" ]; then
    echo ">>> Packaging for Linux..."
    PORTABLE_DIR="${DIST_DIR}/photocraft-companion-v${VERSION}-linux-x86_64-portable"
    rm -rf "${PORTABLE_DIR}"
    mkdir -p "${PORTABLE_DIR}"

    cp target/release/photocraft-companion "${PORTABLE_DIR}/"
    cp -r plugins "${PORTABLE_DIR}/"
    if [ -f "${DIST_DIR}/registry.json" ]; then
        cp "${DIST_DIR}/registry.json" "${PORTABLE_DIR}/" || true
    fi
    cp README.md LICENSE "${PORTABLE_DIR}/"

    PORTABLE_ARCHIVE="${DIST_DIR}/PhotoCraft-Companion-v${VERSION}-linux-x86_64-portable.tar.gz"
    tar -czf "${PORTABLE_ARCHIVE}" -C "${DIST_DIR}" "photocraft-companion-v${VERSION}-linux-x86_64-portable"
    echo ">>> Linux Portable archive created: ${PORTABLE_ARCHIVE}"

    # Simple DEB package layout if dpkg-deb is available
    if command -v dpkg-deb >/dev/null 2>&1; then
        DEB_DIR="${DIST_DIR}/deb_pkg"
        rm -rf "${DEB_DIR}"
        mkdir -p "${DEB_DIR}/DEBIAN"
        mkdir -p "${DEB_DIR}/usr/bin"
        mkdir -p "${DEB_DIR}/usr/share/applications"
        mkdir -p "${DEB_DIR}/usr/share/photocraft-companion/plugins"

        cat <<EOF > "${DEB_DIR}/DEBIAN/control"
Package: photocraft-companion
Version: ${VERSION}
Section: graphics
Priority: optional
Architecture: amd64
Maintainer: kobaltGIT <kobaltmail@gmail.com>
Description: PhotoCraft Companion - floating toolstrip and AI plugin system
EOF

        cp target/release/photocraft-companion "${DEB_DIR}/usr/bin/"
        chmod 0755 "${DEB_DIR}/usr/bin/photocraft-companion"
        cp packaging/linux/photocraft-companion.desktop "${DEB_DIR}/usr/share/applications/"
        if [ -d plugins ]; then
            cp -r plugins/* "${DEB_DIR}/usr/share/photocraft-companion/plugins/" || true
        fi

        # dpkg-deb requires strict permissions
        chmod -R u=rwX,go=rX "${DEB_DIR}"
        chmod 0755 "${DEB_DIR}/DEBIAN"
        chmod 0644 "${DEB_DIR}/DEBIAN/control"

        dpkg-deb --build "${DEB_DIR}" "${DIST_DIR}/PhotoCraft-Companion-v${VERSION}-linux-amd64.deb"
        rm -rf "${DEB_DIR}"
        echo ">>> Linux DEB installer created: ${DIST_DIR}/PhotoCraft-Companion-v${VERSION}-linux-amd64.deb"
    fi
fi

echo "=== Packaging Complete ==="
ls -lh "${DIST_DIR}"
