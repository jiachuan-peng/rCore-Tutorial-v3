#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT="$ROOT/thesis-results/env"
mkdir -p "$OUT" "$ROOT/thesis-results/logs" "$ROOT/thesis-results/data" "$ROOT/thesis-results/code"

uname -a >"$OUT/uname.txt"
lsb_release -a >"$OUT/ubuntu.txt" 2>&1
lscpu >"$OUT/lscpu.txt"
rustc -Vv >"$OUT/rustc.txt"
cargo -V >"$OUT/cargo.txt"
rustup show >"$OUT/rustup.txt"
qemu-system-riscv64 --version >"$OUT/qemu.txt"
git -C "$ROOT" rev-parse HEAD >"$OUT/commit.txt"
git -C "$ROOT" status --short >"$OUT/git-status.txt"
git -C "$ROOT" diff >"$OUT/experiment.patch"
while IFS= read -r file; do
    git -C "$ROOT" diff --no-index -- /dev/null "$ROOT/$file" \
        >>"$OUT/experiment.patch" || [[ $? -eq 1 ]]
done < <(
    git -C "$ROOT" ls-files --others --exclude-standard |
        grep -E '^(os/src/|user/src/|scripts/|thesis-results/README\.md$)'
)

cp "$ROOT/os/src/task/stats.rs" "$ROOT/thesis-results/code/stats.rs"
cp "$ROOT"/user/src/bin/rms_*.rs "$ROOT/thesis-results/code/"
cp "$ROOT/user/src/bin/sched_common_workload.rs" "$ROOT/thesis-results/code/"
cp "$ROOT/user/src/bin/waitpid_common.rs" "$ROOT/thesis-results/code/"

if [[ -d /home/siyun/rCore-baseline ]]; then
    mkdir -p "$ROOT/thesis-results/code/baseline"
    cp /home/siyun/rCore-baseline/os/src/task/stats.rs \
        "$ROOT/thesis-results/code/baseline/stats.rs"
    cp /home/siyun/rCore-baseline/user/src/bin/sched_common_workload.rs \
        "$ROOT/thesis-results/code/baseline/"
    cp /home/siyun/rCore-baseline/user/src/bin/waitpid_common.rs \
        "$ROOT/thesis-results/code/baseline/"
fi

printf 'Environment captured in %s\n' "$OUT"
