/// Marker trait implemented by generated `oneof` enum types.
///
/// Each `oneof` in a protobuf message generates a Rust enum with one variant
/// per field in the oneof. The field on the containing message uses
/// `Option<OneofEnum>`, with `None` representing the unset state — there is
/// no `NotSet` variant.
pub trait Oneof: Clone + PartialEq + Send + Sync {}
