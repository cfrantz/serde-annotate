#!/bin/bash
# There are _lots_ of hjson files in https://github.com/lowRISC/opentitan/
# This script times the transcoding of all of them.
set -euo pipefail
: "${REPO_TOP:=$(git rev-parse --show-toplevel)}"
: "${OPENTITAN:=${HOME}/opentitan/opentitan}"
: "${PROFILE:=dev}"

cd ${REPO_TOP}
ARGS=("$@")
if [[ ${#ARGS[@]} == 0 ]]; then
    ARGS=($(find ${OPENTITAN} -name "*.hjson" -o -name "*.json"))
fi

case ${PROFILE} in
    release)
        TARGET=target/release
        ;;
    *)
        TARGET=target/debug
        ;;
esac

rm -f /tmp/transcode.txt
cargo build --profile=${PROFILE} --example transcode
for f in ${ARGS[@]}; do
    echo "===== Testing $f ====="
    /usr/bin/time -f "%C %U" -a -o /tmp/transcode.txt ${TARGET}/examples/transcode --color --format hjson $f
done

cat /tmp/transcode.txt | cut -d' ' -f5- | sort -n -k2 >/tmp/transcode.$$.txt
