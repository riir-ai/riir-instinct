#!/bin/sh
# check_readme_path_deps.sh — the public-clone posture pin (Issue 030,
# interim posture (d), delegated Claude verdict 2026-11-12, AGREE).
#
# Every external path dependency in Cargo.toml (`path = "../X/..."`) must
# be named in README's Build section with a public/private visibility
# label on the same line. Without this pin the doc goes stale silently
# the next time a path dep lands — which is exactly how Issue 030
# happened. In-repo vendor paths are exempt. Standalone (instinct
# scripts are standalone until the owner-gated CI wiring); run it before
# any manifest or doc change that touches path deps.

set -eu
cd "$(dirname "$0")/.."

# Sibling repo names from external path deps (vendor/ is in-repo).
deps=$(sed -n 's|.*path = "\.\./\([^/"]*\).*|\1|p' Cargo.toml | sort -u)

if [ -z "$deps" ]; then
    echo "check_readme_path_deps: no external path deps; nothing to pin"
    exit 0
fi

# README's Build section: from the '## Build' heading to the next heading.
section=$(awk '/^## Build/{f=1;next} f && /^## /{f=0} f' README.md)

status=0
for dep in $deps; do
    line=$(printf '%s\n' "$section" | grep -Ei "(^|[^a-z0-9_-])${dep}([^a-z0-9_-]|$)" || true)
    if [ -z "$line" ]; then
        echo "FAIL: path dep '$dep' not named in README Build section" >&2
        status=1
    elif ! printf '%s\n' "$line" | grep -Eiq '(^|[^a-z0-9_-])(public|private)([^a-z0-9_-]|$)'; then
        echo "FAIL: path dep '$dep' named in README Build section without a public/private label" >&2
        status=1
    else
        echo "ok: $dep"
    fi
done

if [ "$status" -eq 0 ]; then
    echo "check_readme_path_deps: PASS"
fi
exit $status
