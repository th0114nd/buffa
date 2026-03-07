#!/bin/bash
set -e

# The conformance_test_runner always runs two suites per invocation:
#   1. Binary + JSON (our tests — expect thousands of successes)
#   2. Text format (883 tests — all skipped, textproto is not supported)
# The "0 successes, 883 skipped" output from the second suite is expected.
#
# When CONFORMANCE_OUT is set (e.g. via docker run -v /tmp:/out -e
# CONFORMANCE_OUT=/out), each run's output is tee'd to a log file there
# for post-hoc analysis of failures.

run_suite() {
    local name="$1"
    local log="${CONFORMANCE_OUT:+$CONFORMANCE_OUT/conformance-$name.log}"
    shift
    echo "=== Conformance: $name ==="
    if [ -n "$log" ]; then
        "$@" 2>&1 | tee "$log"
    else
        "$@"
    fi
    echo ""
}

run_suite std \
    conformance_test_runner \
    --failure_list /known_failures.txt \
    --maximum_edition 2024 \
    /usr/local/bin/buffa-conformance

run_suite nostd \
    conformance_test_runner \
    --failure_list /known_failures_nostd.txt \
    --maximum_edition 2024 \
    /usr/local/bin/buffa-conformance-nostd

# Via-view mode: routes binary input through decode_view → to_owned_message.
# JSON I/O is skipped (views have no serde). Verifies owned/view decoder parity.
BUFFA_VIA_VIEW=1 run_suite view \
    conformance_test_runner \
    --failure_list /known_failures_view.txt \
    --maximum_edition 2024 \
    /usr/local/bin/buffa-conformance
