#!/usr/bin/bash

set -euo pipefail

BIN="$1"
echo
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

function restore {
    echo "restoring $revlist from $revlistbackup"
    cat "$revlistbackup" > "$revlist"
}

# backup revocation list
# since it can be a symlink, don't use cp, but shell redirection
revlist=~/.pwget2-revocation
revlistbackup=$(mktemp)
echo "Backing up $revlist to $revlistbackup"
cat "$revlist" > "$revlistbackup"

# empty revocation list
true > "$revlist"



#########
# tests #
#########

trap restore ERR
echo

password='password'
domain='domain'


echo "Acceptance test 1: Basic functionality"
expected="MindDisappointedDoctorAssure_1"
call_xkcdget
assertEquals "$expected" "$xkcdget_output"


echo "Acceptance test 2: Revocation"

echo "Acceptance test 2.1: Short argument"
call_xkcdget '-r'
expected=':&a*5wnoz{0tUw#9U}+!s7qdGlqGo9XhHURZz>r1'
actual=$(tail -1 "$revlist")
assertEquals "$expected" "$actual"

echo "Acceptance test 2.2: Long argument and double revocation"
call_xkcdget '--revoke'
expected='([Z>a9^-KV)T&]R(MH41ykWS>JxWBKIu^Nyhxg{)'
actual=$(tail -1 "$revlist")
assertEquals "$expected" "$actual"



###########
# restore #
###########

restore
