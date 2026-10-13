#!/bin/sh
# SPDX-License-Identifier: MIT
# Run only inside a disposable Ubuntu26 container. Never use on the host.
set -eu
test -f /.dockerenv
. /etc/os-release
test "$ID:$VERSION_ID" = ubuntu:26.04
test "$(id -u)" = 0
test "$#" = 2
first=$1
second=$2
test "$(dpkg-deb -f "$first" Package)" = xmqr-admin
test "$(dpkg-deb -f "$first" Version)" = 0.1.0-1
test "$(dpkg-deb -f "$second" Version)" = 0.1.0-2
private=$(mktemp -d /tmp/xmqr-admin-package-lab.XXXXXX)
chmod 700 "$private"
# The legacy name belongs to another program and must remain untouched.
test ! -e /usr/bin/mqtt-admin
printf '#!/bin/sh\nprintf "legacy-test-only\\n"\n' > /usr/bin/mqtt-admin
chmod 755 /usr/bin/mqtt-admin
printf 'private-test-fixture-only\n' > "$private/state"
chmod 600 "$private/state"
state_hash=$(sha256sum "$private/state" | cut -d ' ' -f1)
# Minimal Ubuntu Docker images exclude most docs; retain this package's notices.
dpkg --path-include='/usr/share/doc/xmqr-admin/*' -i "$first"
test "$(xmqr-admin --version)" = 'xmqr-admin 0.1.0'
xmqr-admin token-init --file "$private/access.token"
test "$(stat -c %a "$private/access.token")" = 600
token_hash=$(sha256sum "$private/access.token" | cut -d ' ' -f1)
test -f /usr/share/doc/xmqr-admin/third-party-notices/manifest.json
test ! -e /usr/lib/systemd/system/xmqr-admin.service
test ! -e /lib/systemd/system/xmqr-admin.service
dpkg --path-include='/usr/share/doc/xmqr-admin/*' -i "$second"
test "$(dpkg-query -W -f='${Version}' xmqr-admin)" = 0.1.0-2
test "$(xmqr-admin --version)" = 'xmqr-admin 0.1.0'
dpkg --path-include='/usr/share/doc/xmqr-admin/*' -i "$first"
test "$(dpkg-query -W -f='${Version}' xmqr-admin)" = 0.1.0-1
dpkg -r xmqr-admin
test ! -e /usr/bin/xmqr-admin
test "$(sha256sum "$private/state" | cut -d ' ' -f1)" = "$state_hash"
test "$(sha256sum "$private/access.token" | cut -d ' ' -f1)" = "$token_hash"
test "$(stat -c %a "$private/access.token")" = 600
test "$(/usr/bin/mqtt-admin)" = legacy-test-only
printf 'PASS: install, revision upgrade, rollback, remove; private fixtures and legacy binary preserved.\n'
