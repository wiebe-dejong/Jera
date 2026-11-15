#!/bin/sh
# Bouwt Jera.app, met het icoon, en zet hem in /Applications.
set -e
cd "$(dirname "$0")"
~/.cargo/bin/cargo build --release

app=/Applications/Jera.app
iconset="$(mktemp -d)/Jera.iconset"
mkdir -p "$iconset"
for m in 16 32 128 256 512; do
  sips -z $m $m iconen/dock.png --out "$iconset/icon_${m}x${m}.png" >/dev/null
  sips -z $((m * 2)) $((m * 2)) iconen/dock.png --out "$iconset/icon_${m}x${m}@2x.png" >/dev/null
done

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp target/release/jera "$app/Contents/MacOS/Jera"
iconutil -c icns "$iconset" -o "$app/Contents/Resources/Jera.icns"
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Jera</string>
  <key>CFBundleDisplayName</key><string>Jera</string>
  <key>CFBundleExecutable</key><string>Jera</string>
  <key>CFBundleIdentifier</key><string>local.jera</string>
  <key>CFBundleIconFile</key><string>Jera</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
EOF
codesign --force --sign - "$app"
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$app"
echo "$app"
