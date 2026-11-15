#!/bin/bash

set -e

WORKDIR=$(mktemp -d)
trap 'rm -rf "$WORKDIR"' EXIT

cat > "$WORKDIR/main.cpp" <<'EOF'
#include <iostream>

int main() {
    std::cout << "Hello, World!\n";
}
EOF

chmod 777 "$WORKDIR"

docker run --rm -i \
    --memory=500m \
    --memory-swap=500m \
    --cpus=1 \
    --pids-limit=64 \
    --network=none \
    --cap-drop=ALL \
    --security-opt=no-new-privileges \
    --read-only \
    --user=10001:10001 \
    --mount type=bind,src="$WORKDIR",dst=/work \
    --tmpfs /tmp:rw,exec,nosuid,nodev,size=64m,uid=10001,gid=10001,mode=0700 \
    ide \
    bash -c '
        cd /work || exit 1

        timeout --signal=KILL 2s \
            g++ -std=c++20 -O2 -Wall -Wextra main.cpp -o main

        compile_status=$?

        if [ "$compile_status" -ne 0 ]; then
            exit "$compile_status"
        fi

        timeout --signal=KILL 2s ./main
    '
