//! Generated protobuf types for buffa benchmarks.

pub mod bench {
    include!(concat!(env!("OUT_DIR"), "/bench_messages.rs"));
}

pub mod benchmarks {
    include!(concat!(env!("OUT_DIR"), "/benchmarks.rs"));
}

pub mod proto3 {
    include!(concat!(env!("OUT_DIR"), "/benchmark_message1_proto3.rs"));
}
