#!/usr/bin/env bash
# Lina Desk: replace the public RustDesk rendezvous server and key compiled into
# the hbb_common submodule with the Lina Desk server, so even the last-resort
# fallback in Config::get_rendezvous_servers()/get_key() is our own server.
# (src/lina_policy.rs fixes the same values at runtime.)
set -euo pipefail

SERVER="${LINA_SERVER:-desk.xcode.it.com}"
KEY="${LINA_KEY:-SiKlUxIe00QSAPd1JXGLvb4Vxx8W4mxx4HnfqKWIqQI=}"
f=libs/hbb_common/src/config.rs

sed -i "s|^pub const RENDEZVOUS_SERVERS: &\[&str\] = &\[.*\];|pub const RENDEZVOUS_SERVERS: \&[\&str] = \&[\"${SERVER}\"];|" "$f"
sed -i "s|^pub const RS_PUB_KEY: &str = \".*\";|pub const RS_PUB_KEY: \&str = \"${KEY}\";|" "$f"

grep -n -E '^pub const (RENDEZVOUS_SERVERS|RS_PUB_KEY)' "$f"
grep -q "\"${SERVER}\"" "$f"
grep -q "\"${KEY}\"" "$f"
if grep -q 'rs-ny.rustdesk.com' "$f"; then
  echo "public RustDesk server still present in $f" >&2
  exit 1
fi
echo "Lina Desk server baked in: ${SERVER}"
