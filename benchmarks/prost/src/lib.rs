//! Generated protobuf types for prost benchmarks.

pub mod bench {
    include!(concat!(env!("OUT_DIR"), "/bench.rs"));
    // pbjson-generated Serialize/Deserialize impls.
    include!(concat!(env!("OUT_DIR"), "/bench.serde.rs"));
}

pub mod benchmarks {
    include!(concat!(env!("OUT_DIR"), "/benchmarks.rs"));
    include!(concat!(env!("OUT_DIR"), "/benchmarks.serde.rs"));

    // prost-build names generated files by proto package: `benchmarks.proto3`
    // becomes `benchmarks.proto3.rs`, so the `proto3` module nests here rather
    // than at the crate root (unlike the buffa benchmark crate which uses its
    // own file-name convention).
    pub mod proto3 {
        include!(concat!(env!("OUT_DIR"), "/benchmarks.proto3.rs"));
        include!(concat!(env!("OUT_DIR"), "/benchmarks.proto3.serde.rs"));
    }
}
