#!/usr/bin/env bash
# Run all fuzz targets in parallel with output redirected to log files.
# Prints periodic progress summaries.
#
# Usage: scripts/fuzz-all.sh [max_total_time]
#   max_total_time: seconds per target (default: 28800 = 8 hours)
#
# Logs are written to /tmp/buffa-fuzz/<target>.log
# Crashes are saved to fuzz/artifacts/<target>/

set -euo pipefail

MAX_TIME="${1:-28800}"
TARGETS=(decode_proto3 decode_proto2 decode_wkt json_roundtrip encode_proto3 wkt_json_strings)
LOG_DIR="/tmp/buffa-fuzz"
FUZZ_DIR="fuzz"
STATUS_INTERVAL=300  # seconds between progress reports

# Per-target extra libFuzzer flags.
#
# encode_proto3: uses Arbitrary<TestAllTypesProto3> — a few input bytes can
# generate structs with many heap allocations (Vecs, Strings, HashMaps, nested
# Boxes). Under ASan's allocator this causes ~2KB/iter unrecovered RSS growth
# (heap fragmentation; ASan never returns to OS). Hit the 2GB rss limit after
# ~1.6M iterations (~26 min) — see oom-f1a736fc, Mar 2026. -fork=1 spawns
# child processes that reset memory per job. Verified: oom=0 over 13 jobs.
# Trade-off: ~45% throughput loss from per-job corpus reload.
#
# The other 5 targets take bounded raw-byte/string input (max 4KB from
# libFuzzer's default -max_len), so their allocations are naturally capped
# and they run 14hr+ without fragmentation OOM.
target_extra_flags() {
    case "$1" in
        encode_proto3) echo "-fork=1 -ignore_ooms=0 -ignore_crashes=0 -ignore_timeouts=0" ;;
        *)             echo "" ;;
    esac
}

mkdir -p "$LOG_DIR"

# Build all targets first (sequentially, to avoid parallel compilation issues).
echo "Building fuzz targets..."
for target in "${TARGETS[@]}"; do
    cargo +nightly fuzz build --fuzz-dir "$FUZZ_DIR" "$target" 2>&1 \
        | tail -1
done
echo ""

# Launch all targets in parallel.
# Output is filtered through a helper that:
#   - Keeps only important lines (crashes, errors, final stats) in the log
#   - Maintains a "last status" file with the most recent progress line
PIDS=()
for target in "${TARGETS[@]}"; do
    log="$LOG_DIR/$target.log"
    status_file="$LOG_DIR/$target.status"
    echo "Starting $target (log: $log, max_time: ${MAX_TIME}s)"
    : >"$log"
    : >"$status_file"
    extra_flags=$(target_extra_flags "$target")
    # shellcheck disable=SC2086  # extra_flags is intentionally word-split
    cargo +nightly fuzz run --fuzz-dir "$FUZZ_DIR" "$target" \
        -- -max_total_time="$MAX_TIME" -print_final_stats=1 $extra_flags \
        2>&1 | while IFS= read -r line; do
            # Always save the most recent progress line for status reports.
            # Non-fork mode: "#NNN ACTION cov: ..."; fork mode: "#NNN: cov: ...".
            if [[ "$line" =~ ^#[0-9] ]]; then
                echo "$line" >"$status_file"
            fi
            # Log important lines only: crashes, errors, stats, summary.
            if [[ "$line" =~ SUMMARY|ERROR|CRASH|ALARM|panic|assertion|stat::|BINGO|Done[[:space:]] ]]; then
                echo "$line" >>"$log"
            fi
        done &
    PIDS+=($!)
done

echo ""
echo "All ${#TARGETS[@]} targets running. Logs in $LOG_DIR/"
echo "Press Ctrl-C to stop all targets."
echo ""

# Trap Ctrl-C to kill all children.
cleanup() {
    echo ""
    echo "Stopping all fuzz targets..."
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
    done
    wait 2>/dev/null
    print_summary
    exit 0
}
trap cleanup INT TERM

# Read the most recent progress line from the status file.
parse_stats() {
    local target="$1"
    local status_file="$LOG_DIR/$target.status"
    if [[ ! -f "$status_file" ]]; then
        echo "not started"
        return
    fi
    local last
    last=$(cat "$status_file" 2>/dev/null || true)
    if [[ -n "$last" ]]; then
        # Trim to the useful part: #NNN ACTION cov: X ft: Y
        echo "$last" | grep -oP '#\d+\s+\S+\s+cov: \d+\s+ft: \d+' || echo "$last"
    else
        echo "starting up..."
    fi
}

print_summary() {
    echo "── Fuzz progress $(date +%H:%M:%S) ──"
    for i in "${!TARGETS[@]}"; do
        local target="${TARGETS[$i]}"
        local pid="${PIDS[$i]}"
        local log="$LOG_DIR/$target.log"
        local status
        if kill -0 "$pid" 2>/dev/null; then
            status="running"
        else
            wait "$pid" 2>/dev/null && status="finished" || status="CRASHED"
        fi
        local stats
        stats=$(parse_stats "$target")
        printf "  %-20s [%s] %s\n" "$target" "$status" "$stats"

        # Check for crash artifacts.
        local artifact_dir="fuzz/artifacts/$target"
        if [[ -d "$artifact_dir" ]]; then
            local crashes
            crashes=$(find "$artifact_dir" -name 'crash-*' -o -name 'oom-*' -o -name 'timeout-*' 2>/dev/null | wc -l)
            if [[ "$crashes" -gt 0 ]]; then
                printf "    *** %d crash artifact(s) in %s ***\n" "$crashes" "$artifact_dir"
            fi
        fi
    done
    echo ""
}

# Wait loop: periodic status reports until all targets finish.
# Check every 5 seconds whether targets are still running, but only
# print a summary every STATUS_INTERVAL seconds.
elapsed=0
while true; do
    all_done=true
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            all_done=false
            break
        fi
    done

    if $all_done; then
        print_summary
        echo "All targets finished."
        break
    fi

    sleep 5
    elapsed=$((elapsed + 5))
    if [[ $elapsed -ge $STATUS_INTERVAL ]]; then
        print_summary
        elapsed=0
    fi
done
