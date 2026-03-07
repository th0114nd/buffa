#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz well-known type decoders. Each WKT has different internal
    // structure and serde impls, so fuzzing them individually catches
    // edge cases in the hand-written buffa-types code.

    // Timestamp
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::Timestamp>(data);

    // Duration
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::Duration>(data);

    // Any
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::Any>(data);

    // Struct
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::Struct>(data);

    // Value
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::Value>(data);

    // FieldMask
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::FieldMask>(data);

    // ListValue
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::ListValue>(data);

    // Wrapper types
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::Int32Value>(data);
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::Int64Value>(data);
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::UInt32Value>(data);
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::UInt64Value>(data);
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::FloatValue>(data);
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::DoubleValue>(data);
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::BoolValue>(data);
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::StringValue>(data);
    let _ = buffa_fuzz::roundtrip::<buffa_types::google::protobuf::BytesValue>(data);
});
