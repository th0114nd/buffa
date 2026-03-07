//! Generated protobuf descriptor types for bootstrapping.
//!
//! These types are generated from `google/protobuf/descriptor.proto` and
//! `google/protobuf/compiler/plugin.proto` using buffa-codegen itself.
//! This makes buffa-codegen fully self-hosted — no external protobuf
//! library is needed to decode protoc's `CodeGeneratorRequest` — and gives
//! direct access to edition features (`FeatureSet`, `Edition`, etc.).
//!
//! To regenerate:
//! ```sh
//! protoc --descriptor_set_out=/tmp/descriptor_set.pb --include_imports \
//!     -I <protobuf-src>/src \
//!     google/protobuf/descriptor.proto \
//!     google/protobuf/compiler/plugin.proto
//! cargo run --bin gen_descriptor_types -- /tmp/descriptor_set.pb
//! ```

#[allow(
    clippy::all,
    dead_code,
    missing_docs,
    unused_imports,
    unreachable_patterns,
    non_camel_case_types
)]
pub mod descriptor {
    // Re-export the buffa crate so `::buffa::` paths in generated code resolve.
    use buffa;
    include!("google.protobuf.descriptor.rs");
}

// Re-export the specific descriptor types referenced via `super::` from the
// compiler module (cross-package references in generated code).
#[allow(unused_imports)]
pub use descriptor::{FileDescriptorProto, GeneratedCodeInfo};

#[allow(
    clippy::all,
    dead_code,
    missing_docs,
    unused_imports,
    unreachable_patterns,
    non_camel_case_types
)]
pub mod compiler {
    // Re-export GeneratedCodeInfo so `super::GeneratedCodeInfo` resolves from
    // nested sub-modules (e.g. `code_generator_response::File`).
    #[allow(unused_imports)]
    pub use crate::generated::descriptor::GeneratedCodeInfo;

    use buffa;
    include!("google.protobuf.compiler.plugin.rs");
}
