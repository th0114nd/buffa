#!/bin/sh
# Summarize the googleapis stress test results.

echo "=== Google Cloud APIs codegen stress test ==="
echo ""

# Count generated files.
file_count=$(find /results/gen -name '*.rs' 2>/dev/null | wc -l)
total_lines=$(find /results/gen -name '*.rs' -exec cat {} + 2>/dev/null | wc -l)
total_size=$(du -sh /results/gen 2>/dev/null | cut -f1)

echo "Generated files: $file_count"
echo "Total lines:     $total_lines"
echo "Total size:      $total_size"

# Count compiled files (lib.rs includes minus excluded).
compiled=$(grep -c 'include!' /results/lib.rs 2>/dev/null || echo "?")
echo "Compiled files:  $compiled"
echo ""

# Show any errors from the generation log.
if grep -qi "error\|panic\|failed" /results/generate.log; then
    echo "=== GENERATION ERRORS ==="
    grep -i "error\|panic\|failed" /results/generate.log
    echo ""
fi

# Show compilation results.
echo "=== Compilation ==="
if grep -q "Compile exit code: 0" /results/compile.log; then
    echo "PASS — generated code compiles successfully"
else
    echo "FAIL — compilation errors detected"
    echo ""
    # Show the last 50 lines of errors (skip warnings).
    grep -E "^error" /results/compile.log | tail -50
fi
echo ""

# If /out is mounted, copy results there.
if [ -d /out ]; then
    echo "Copying results to /out ..."
    cp -r /results/gen /out/gen 2>/dev/null
    cp /results/generate.log /out/generate.log
    cp /results/compile.log /out/compile.log
    cp /results/lib.rs /out/lib.rs
    echo "Done."
fi
