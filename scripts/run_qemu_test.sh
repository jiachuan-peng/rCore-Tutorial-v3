#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 || $# -gt 3 ]]; then
    echo "usage: $0 <app-name> <log-name> [run-seconds]" >&2
    exit 2
fi

SCRIPT_ROOT=$(cd "$(dirname "$0")/.." && pwd)
ROOT=${RC_ROOT:-$SCRIPT_ROOT}
RESULT_ROOT=${RESULT_ROOT:-$SCRIPT_ROOT}
APP=$1
LOG_NAME=$2
RUN_SECONDS=${3:-8}
BOOT_SECONDS=${BOOT_SECONDS:-3}
LOG_DIR="$RESULT_ROOT/thesis-results/logs"
mkdir -p "$LOG_DIR"

case "$LOG_NAME" in
    *.log) ;;
    *) LOG_NAME="${LOG_NAME}.log" ;;
esac

if [[ ${SKIP_BUILD:-0} == 1 ]]; then
    RUN_COMMAND=(
        qemu-system-riscv64
        -machine virt
        -nographic
        -bios "$ROOT/bootloader/rustsbi-qemu.bin"
        -device "loader,file=$ROOT/os/target/riscv64gc-unknown-none-elf/release/os.bin,addr=0x80200000"
    )
else
    RUN_COMMAND=(make -C "$ROOT/os" run)
fi

(
    sleep "$BOOT_SECONDS"
    printf '%s\n' "$APP"
    sleep "$RUN_SECONDS"
    printf '\001x'
) | timeout "$((RUN_SECONDS + 30))" "${RUN_COMMAND[@]}" 2>&1 | tee "$LOG_DIR/$LOG_NAME"

printf 'Saved %s\n' "$LOG_DIR/$LOG_NAME"
