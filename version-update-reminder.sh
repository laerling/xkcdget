#!/usr/bin/env bash

#######################
# version match check #
#######################

package_version="$(grep ^version Cargo.toml|cut -d\" -f2)"
code_version="$(grep ^const\ XKCDGET_VERSION src/main.rs|cut -d\" -f2)"
if [ "$package_version" != "$code_version" ]; then
    echo "Package version ($package_version) doesn't match code version ($code_version)" >/dev/stderr
    exit 1
fi


##############
# diff check #
##############

diff="$(git diff)"

# don't check if there isn't any diff
[ -z "$diff" ] && exit 0

# check that version has been updated
echo -n "$diff" | grep -q '^+const XKCDGET_VERSION' || \
    echo "WARNING: Looks like you have some diff but forgot to update the semantic version!" >/dev/stderr
