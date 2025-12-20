#!/usr/bin/env bash

set -euo pipefail

REVLIST_NAME=.xkcdget-revocation
BIN="${1:-target/release/xkcdget}"

BIN="$(realpath "$BIN")"
if ! [ -x "$BIN" ]; then
    echo "binary doesn't exist or isn't executable: $BIN" >/dev/stderr
    exit 1
fi
echo "Running acceptance test on binary $BIN"

function call_xkcdget {
    cmd="echo -n $password|$BIN $@ $domain"
    echo "Execute: $cmd"
    xkcdget_output=$(eval "$cmd")
}

function assertEquals {
    expected="$1"
    actual="$2"
    if [ "$actual" != "$expected" ]; then
        (>&2 echo "Assertion error. Expected \"$expected\", got \"$actual\".")
        exit 1
    fi
}

# make temporary directory for revocation list
revlistdir="$(mktemp -d)"
export HOME="$revlistdir"

# initialize empty revocation list
revlist="$revlistdir/$REVLIST_NAME"
echo "Using revocation list: $revlist"
true > "$revlist"



#########
# tests #
#########

password='password'
domain='domain'


echo
echo "Acceptance test 1: Basic functionality"

echo
echo "Acceptance test 1.1: Supply domain via argument"
expected="MindDisappointedDoctorAssure_1"
call_xkcdget
assertEquals "$expected" "$xkcdget_output"

echo "Acceptance test 1.2: Supply domain via interactive input"
expected="MindDisappointedDoctorAssure_1"
xkcdget_output=$(echo -en "$domain\n$password"|"$BIN")
assertEquals "$expected" "$xkcdget_output"


echo "Acceptance test 2: Revocation"

echo
echo "Acceptance test 2.1: Shorthand flag"
call_xkcdget '-r'
expected=':&a*5wnoz{0tUw#9U}+!s7qdGlqGo9XhHURZz>r1'
actual=$(tail -1 "$revlist")
assertEquals "$expected" "$actual"

echo
echo "Acceptance test 2.2: Longhand flag, double revocation"
call_xkcdget '--revoke'
expected='([Z>a9^-KV)T&]R(MH41ykWS>JxWBKIu^Nyhxg{)'
actual=$(tail -1 "$revlist")
assertEquals "$expected" "$actual"
