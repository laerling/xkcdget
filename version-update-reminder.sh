#!/usr/bin/env bash

diff="$(git diff)"

# don't check if there isn't any diff
[ -z "$diff" ] && exit 0

# check that version has been updated
echo -n "$diff" | grep -q '^+const XKCDGET_VERSION' || \
    echo "WARNING: Looks like you have some diff but forgot to update the semantic version!" >/dev/stderr
