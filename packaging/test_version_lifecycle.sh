#!/bin/sh
# SPDX-License-Identifier: MIT
# Run only inside a fresh disposable Ubuntu26 container, with package mounts RO.
set -eu
test -f /.dockerenv
. /etc/os-release
test "$ID:$VERSION_ID" = ubuntu:26.04
test "$(id -u)" = 0
test "$#" = 2
old=$1
new=$2
test "$(dpkg-deb -f "$old" Package)" = xmqr-admin
test "$(dpkg-deb -f "$new" Package)" = xmqr-admin
test "$(dpkg-deb -f "$old" Architecture)" = amd64
test "$(dpkg-deb -f "$new" Architecture)" = amd64
test "$(dpkg-deb -f "$old" Version)" = 0.10.0-1
test "$(dpkg-deb -f "$new" Version)" = 0.11.0-1
test ! -e /usr/bin/xmqr-admin
test ! -e /usr/bin/mqtt-admin
private=$(mktemp -d /tmp/xmqr-admin-a22.XXXXXX)
chmod 700 "$private"
printf '#!/bin/sh\nprintf "legacy-test-only\\n"\n' > /usr/bin/mqtt-admin
chmod 755 /usr/bin/mqtt-admin
printf 'synthetic-fixture-not-a-mqtt-document\n' > "$private/state"
chmod 600 "$private/state"
state_hash=$(sha256sum "$private/state" | cut -d ' ' -f1)
dpkg --path-include='/usr/share/doc/xmqr-admin/*' -i "$old"
test "$(xmqr-admin --version)" = 'xmqr-admin 0.10.0'
xmqr-admin token-init --file "$private/access.token"
token_hash=$(sha256sum "$private/access.token" | cut -d ' ' -f1)
preserved() {
    test "$(sha256sum "$private/state" | cut -d ' ' -f1)" = "$state_hash"
    test "$(sha256sum "$private/access.token" | cut -d ' ' -f1)" = "$token_hash"
    test "$(stat -c %a "$private/access.token")" = 600
    test "$(/usr/bin/mqtt-admin)" = legacy-test-only
}
dpkg --path-include='/usr/share/doc/xmqr-admin/*' -i "$new"
test "$(xmqr-admin --version)" = 'xmqr-admin 0.11.0'
test -f /usr/share/doc/xmqr-admin/third-party-notices/manifest.json
test ! -e /usr/lib/systemd/system/xmqr-admin.service
test ! -e /lib/systemd/system/xmqr-admin.service
preserved
dpkg --path-include='/usr/share/doc/xmqr-admin/*' -i "$old"
test "$(xmqr-admin --version)" = 'xmqr-admin 0.10.0'
preserved
dpkg -r xmqr-admin
test ! -e /usr/bin/xmqr-admin
preserved
dpkg -P xmqr-admin
preserved
printf 'A22_PACKAGE_VERSION_LIFECYCLE_PASS\n'
