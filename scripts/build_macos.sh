#!/usr/bin/env bash
set -euo pipefail

echo "==> Building optimized release binary..."
cargo build --release

APP_NAME="Compare"
DIST_DIR="dist"
APP_DIR="${DIST_DIR}/${APP_NAME}.app"
STAGE_DIR="/tmp/${APP_NAME}_dmg_stage"
DMG_PATH="${DIST_DIR}/${APP_NAME}.dmg"

echo "==> Assembling ${APP_DIR}..."
mkdir -p "${DIST_DIR}"
rm -rf "${APP_DIR}"
mkdir -p "${APP_DIR}/Contents/MacOS"
mkdir -p "${APP_DIR}/Contents/Resources"

cp "target/release/diffitrust" "${APP_DIR}/Contents/MacOS/${APP_NAME}"
chmod +x "${APP_DIR}/Contents/MacOS/${APP_NAME}"
cp "assets/icon.icns" "${APP_DIR}/Contents/Resources/AppIcon.icns"

cat << 'PLIST' > "${APP_DIR}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Compare</string>
    <key>CFBundleDisplayName</key>
    <string>Compare</string>
    <key>CFBundleIdentifier</key>
    <string>com.compare.app</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleExecutable</key>
    <string>Compare</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2026 Compare</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>All Files</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.item</string>
                <string>public.content</string>
                <string>public.data</string>
                <string>public.plain-text</string>
                <string>public.text</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
PLIST

plutil -lint "${APP_DIR}/Contents/Info.plist"

echo "==> Code signing bundle..."
codesign --force --deep --sign - "${APP_DIR}"

echo "==> Registering with LaunchServices..."
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "${APP_DIR}" || true


echo "==> Building DMG installer..."
rm -rf "${STAGE_DIR}" "${DMG_PATH}"
mkdir -p "${STAGE_DIR}"
cp -R "${APP_DIR}" "${STAGE_DIR}/"
ln -s /Applications "${STAGE_DIR}/Applications"

hdiutil create \
    -volname "${APP_NAME}" \
    -srcfolder "${STAGE_DIR}" \
    -ov \
    -format UDZO \
    "${DMG_PATH}"

rm -rf "${STAGE_DIR}"

echo "==> Successfully created:"
echo "    App: ${APP_DIR}"
echo "    DMG: ${DMG_PATH}"
