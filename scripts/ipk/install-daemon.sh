#!/bin/sh
# Run explicitly through root SSH after installing the IPK.
set -eu

if [ "$(id -u)" != 0 ]; then
    echo "Run this script as root on the TV." >&2
    exit 1
fi

app_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
daemon_dir=/var/home/root/lg-hue-sync
daemon="$daemon_dir/lg-hue-sync"
unit="$daemon_dir/lg-hue-sync.service"
hook=/var/lib/webosbrew/init.d/50-lg-hue-sync

# Check the bundled ELF and its loader before changing the installed service.
"$app_dir/payload/lg-hue-sync" --version
mkdir -p "$daemon_dir" /var/lib/webosbrew/init.d /run/systemd/system
staged=$(mktemp "$daemon_dir/.daemon.XXXXXX")
trap 'rm -f "$staged"' EXIT
cp "$app_dir/payload/lg-hue-sync" "$staged"
chmod 755 "$staged"
cp "$app_dir/payload/provision_luna.sh" "$daemon_dir/provision_luna.sh"
chmod 755 "$daemon_dir/provision_luna.sh"
bash "$daemon_dir/provision_luna.sh" local

if [ ! -e "$unit" ]; then
    cp "$app_dir/payload/lg-hue-sync.service" "$unit"
fi
if [ ! -e "$hook" ]; then
    cp "$app_dir/payload/50-lg-hue-sync" "$hook"
    chmod 755 "$hook"
fi
cp "$unit" /run/systemd/system/lg-hue-sync.service
systemctl daemon-reload

previous=false
if [ -x "$daemon" ]; then
    cp -p "$daemon" "$daemon.previous"
    previous=true
fi
mv -f "$staged" "$daemon"
if systemctl restart lg-hue-sync && sleep 2 && systemctl is-active --quiet lg-hue-sync; then
    echo "LG Ambilight Sync is running. Open the app or http://TV_IP:8088/."
    exit 0
fi

systemctl stop lg-hue-sync || true
if [ "$previous" = true ]; then
    cp -p "$daemon.previous" "$staged"
    mv -f "$staged" "$daemon"
    systemctl start lg-hue-sync || true
fi
echo "Service startup failed. Check journalctl -u lg-hue-sync." >&2
exit 1
