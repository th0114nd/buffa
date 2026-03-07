#![no_main]

//! Fuzzes the hand-written WKT string parsers that are only reachable via
//! JSON, not the binary wire format:
//!
//! - `Timestamp`  — RFC 3339 parsing (`parse_rfc3339`)
//! - `Duration`   — decimal-seconds parsing (`parse_duration_string`)
//! - `FieldMask`  — camelCase↔snake_case conversion
//!
//! The existing `decode_wkt` target only exercises binary decode, so these
//! parsers are never reached. `json_roundtrip` uses `TestAllTypesProto3`
//! (which has these types as fields) but the fuzzer feeds raw bytes that
//! rarely form valid JSON with a WKT string payload — the probability of
//! finding `"2024-02-30T25:61:61Z"` by byte mutation is essentially zero.
//!
//! This target feeds the raw fuzzer string **directly** as a JSON string
//! value to each WKT's `Deserialize` impl, so every iteration exercises
//! the parser. When parsing succeeds, we verify:
//!
//!   1. Serialize → re-parse produces the same value (format round-trip).
//!   2. Binary encode → decode produces the same value (wire round-trip).
//!   3. The parsed values are in the spec-defined valid range.
//!
//! This catches: panics in parsers, range-check gaps, format/parse asymmetry.

use buffa::Message;
use buffa_types::google::protobuf::{Duration, FieldMask, Timestamp};
use libfuzzer_sys::fuzz_target;

/// Timestamp seconds range: [0001-01-01T00:00:00Z, 9999-12-31T23:59:59Z].
/// Per protobuf spec; serialize() also enforces this.
const MIN_TS_SECS: i64 = -62_135_596_800;
const MAX_TS_SECS: i64 = 253_402_300_799;

/// Duration seconds range: ±10000 years. Per protobuf spec.
const MAX_DUR_SECS: i64 = 315_576_000_000;

fn fuzz_timestamp(s: &str) {
    // Wrap the fuzzer string as a JSON string value and try to deserialize.
    let json = serde_json::Value::String(s.to_string());
    let ts: Timestamp = match serde_json::from_value(json) {
        Ok(t) => t,
        Err(_) => return, // Invalid RFC 3339 — expected for most inputs.
    };

    // If parse succeeded, verify the result is in-range.
    // parse_rfc3339 should never accept a string that produces out-of-range
    // seconds (it knows the year bounds), and nanos must be [0, 1e9).
    assert!(
        (MIN_TS_SECS..=MAX_TS_SECS).contains(&ts.seconds),
        "Timestamp seconds {} out of range for input: {s:?}",
        ts.seconds
    );
    assert!(
        (0..1_000_000_000).contains(&ts.nanos),
        "Timestamp nanos {} out of range for input: {s:?}",
        ts.nanos
    );

    // Format → re-parse round-trip. If parse accepted it, serialize must
    // succeed (seconds is in-range, checked above).
    let formatted = serde_json::to_string(&ts)
        .unwrap_or_else(|e| panic!("Timestamp serialize failed for {ts:?} (input {s:?}): {e}"));
    let reparsed: Timestamp = serde_json::from_str(&formatted)
        .unwrap_or_else(|e| panic!("Timestamp format→parse failed: {formatted} → {e}"));
    assert_eq!(
        (reparsed.seconds, reparsed.nanos),
        (ts.seconds, ts.nanos),
        "Timestamp format→parse changed value: {s:?} → {ts:?} → {formatted} → {reparsed:?}"
    );

    // Binary round-trip (same as decode_wkt, but seeded by a known-good parse).
    let encoded = ts.encode_to_vec();
    let decoded =
        Timestamp::decode_from_slice(&encoded).expect("Timestamp binary round-trip decode failed");
    assert_eq!((decoded.seconds, decoded.nanos), (ts.seconds, ts.nanos));
}

fn fuzz_duration(s: &str) {
    let json = serde_json::Value::String(s.to_string());
    let dur: Duration = match serde_json::from_value(json) {
        Ok(d) => d,
        Err(_) => return,
    };

    // Range checks per protobuf Duration spec: |seconds| ≤ 10000 years,
    // |nanos| < 1e9, and seconds/nanos must have the same sign (or one is zero).
    assert!(
        dur.seconds.unsigned_abs() <= MAX_DUR_SECS as u64,
        "Duration seconds {} out of range for input: {s:?}",
        dur.seconds
    );
    assert!(
        dur.nanos.unsigned_abs() < 1_000_000_000,
        "Duration nanos {} out of range for input: {s:?}",
        dur.nanos
    );
    // Sign consistency: if both are non-zero, they must have the same sign.
    if dur.seconds != 0 && dur.nanos != 0 {
        assert_eq!(
            dur.seconds.signum() as i32,
            dur.nanos.signum(),
            "Duration seconds/nanos sign mismatch: {dur:?} for input: {s:?}"
        );
    }

    // Format → re-parse round-trip.
    let formatted = serde_json::to_string(&dur)
        .unwrap_or_else(|e| panic!("Duration serialize failed for {dur:?} (input {s:?}): {e}"));
    let reparsed: Duration = serde_json::from_str(&formatted)
        .unwrap_or_else(|e| panic!("Duration format→parse failed: {formatted} → {e}"));
    assert_eq!(
        (reparsed.seconds, reparsed.nanos),
        (dur.seconds, dur.nanos),
        "Duration format→parse changed value: {s:?} → {dur:?} → {formatted} → {reparsed:?}"
    );

    // Binary round-trip.
    let encoded = dur.encode_to_vec();
    let decoded =
        Duration::decode_from_slice(&encoded).expect("Duration binary round-trip decode failed");
    assert_eq!((decoded.seconds, decoded.nanos), (dur.seconds, dur.nanos));
}

fn fuzz_field_mask(s: &str) {
    let json = serde_json::Value::String(s.to_string());
    let fm: FieldMask = match serde_json::from_value(json) {
        Ok(m) => m,
        Err(_) => return,
    };

    // If parse succeeded, paths are snake_case (camel_to_snake applied).
    // Serialize validates that paths round-trip through snake→camel→snake,
    // so serialize may legitimately fail if a path we parsed from camelCase
    // doesn't round-trip (e.g. "aBBc" → "a_b_bc" → "aBBc" is OK, but
    // ambiguous inputs exist). When serialize succeeds, verify the round-trip.
    let Ok(formatted) = serde_json::to_string(&fm) else {
        // Serialize rejected — the parsed paths contain something that can't
        // round-trip through camelCase. Not a bug; the JSON FieldMask spec
        // allows receiving paths that can't be re-serialized losslessly.
        return;
    };

    let reparsed: FieldMask = serde_json::from_str(&formatted)
        .unwrap_or_else(|e| panic!("FieldMask format→parse failed: {formatted} → {e}"));
    assert_eq!(
        reparsed.paths, fm.paths,
        "FieldMask format→parse changed paths: {s:?} → {:?} → {formatted} → {:?}",
        fm.paths, reparsed.paths
    );

    // Binary round-trip.
    let encoded = fm.encode_to_vec();
    let decoded =
        FieldMask::decode_from_slice(&encoded).expect("FieldMask binary round-trip decode failed");
    assert_eq!(decoded.paths, fm.paths);
}

fuzz_target!(|data: &[u8]| {
    // Use the first byte to select the target parser, which lets libFuzzer
    // learn per-parser input structure and avoid wasting cycles on one
    // parser while finding interesting inputs for another.
    //
    // The remaining bytes are the UTF-8 string to parse. Invalid UTF-8
    // short-circuits (serde_json would reject it anyway, but this keeps
    // iteration speed high).
    if data.is_empty() {
        return;
    }
    let selector = data[0];
    let Ok(s) = std::str::from_utf8(&data[1..]) else {
        return;
    };

    // Distribute roughly evenly across the three parsers, with a small
    // remainder going to "all three" for cross-parser exploration.
    match selector % 4 {
        0 => fuzz_timestamp(s),
        1 => fuzz_duration(s),
        2 => fuzz_field_mask(s),
        _ => {
            // Exercise all three on the same string.
            fuzz_timestamp(s);
            fuzz_duration(s);
            fuzz_field_mask(s);
        }
    }
});
