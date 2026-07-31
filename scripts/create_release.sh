#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 3 ]]; then
    echo "Usage: $0 <app-path> <screen-width> <screen-height>" >&2
    exit 1
fi

APP_PATH="$1"
SCREEN_WIDTH="$2"
SCREEN_HEIGHT="$3"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

if [[ ! -f "$APP_PATH/Cargo.toml" ]]; then
    echo "App path does not contain Cargo.toml: $APP_PATH" >&2
    exit 1
fi

APP_PATH="$(cd "$APP_PATH" && pwd)"
APP_NAME="$(basename "$APP_PATH")"
# Release names use the product name: strip the repo-convention "rust-app-" prefix.
APP_NAME="${APP_NAME#rust-app-}"
APP_VERSION="$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$APP_PATH/Cargo.toml" | head -1)"
PROJECT_VERSION="$(tr -d '[:space:]' < "$REPO_ROOT/VERSION.txt")"
PLATFORM="$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m)"

if [[ -z "$APP_VERSION" ]]; then
    APP_VERSION="unknown"
fi

BUILD_DIR="$REPO_ROOT/build-release"
RELEASE_NAME="${APP_NAME}-${APP_VERSION}-screen-desktop-${PROJECT_VERSION}-${PLATFORM}"
RELEASES_DIR="$REPO_ROOT/releases"
PACKAGE_DIR="$RELEASES_DIR/$RELEASE_NAME"
ARCHIVE_PATH="$RELEASES_DIR/$RELEASE_NAME.zip"

cmake -S "$REPO_ROOT" -B "$BUILD_DIR" \
    -DAPP_PATH="$APP_PATH" \
    -DSDL_HOR_RES="$SCREEN_WIDTH" \
    -DSDL_VER_RES="$SCREEN_HEIGHT" \
    -DCMAKE_BUILD_TYPE=Release
cmake --build "$BUILD_DIR" -j

rm -rf "$PACKAGE_DIR" "$ARCHIVE_PATH"
mkdir -p "$PACKAGE_DIR"
cp "$BUILD_DIR/screen_desktop" "$PACKAGE_DIR/"

if [[ -d "$APP_PATH/fs" ]]; then
    cp -R "$APP_PATH/fs" "$PACKAGE_DIR/"
fi

for config in simulator_config.json agent_config.json; do
    if [[ -f "$APP_PATH/$config" ]]; then
        cp "$APP_PATH/$config" "$PACKAGE_DIR/"
    fi
done

if [[ -d "$APP_PATH/simulator_layouts" ]]; then
    cp -R "$APP_PATH/simulator_layouts" "$PACKAGE_DIR/"
fi

cat > "$PACKAGE_DIR/release-info.txt" <<EOF
screen-desktop-version=$PROJECT_VERSION
app-name=$APP_NAME
app-version=$APP_VERSION
screen-width=$SCREEN_WIDTH
screen-height=$SCREEN_HEIGHT
platform=$PLATFORM
EOF

(
    cd "$RELEASES_DIR"
    cmake -E tar cf "$RELEASE_NAME.zip" --format=zip "$RELEASE_NAME"
)

echo "Release created: $ARCHIVE_PATH"