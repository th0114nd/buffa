//! Cached encoded size for efficient serialization.
//!
//! Protobuf's wire format requires knowing the encoded size of sub-messages
//! before writing them (for length-delimited encoding). Without caching, this
//! requires recomputing sizes at every nesting level, leading to O(depth^2)
//! or even exponential time for deeply nested messages.
//!
//! `CachedSize` stores the computed encoded size in the message struct itself.
//! The serialization pipeline becomes:
//! 1. `compute_size()` — walk the message tree, compute and cache all sizes.
//! 2. `write_to()` — walk the tree again, using cached sizes for length prefixes.
//!
//! Both passes are O(n) in the total message size, making serialization linear.
//!
//! # Design note: `AtomicU32` over `Cell<u32>`
//!
//! An earlier design used `Cell<u32>` to avoid "atomic overhead", which made
//! generated message structs `!Sync`. In practice, `Relaxed`-ordered atomic
//! loads and stores compile to identical instructions as plain memory accesses
//! on every major platform (x86/x86_64, ARM64, ARM32, RISC-V) — the compiler
//! barrier `Relaxed` adds is free at runtime. Switching to `AtomicU32` makes
//! messages `Sync`, which enables `Arc<Message>` and parallel read access
//! without any measurable serialization overhead.

use core::sync::atomic::{AtomicU32, Ordering};

/// A cached encoded byte size, stored in each generated message struct.
///
/// Uses `AtomicU32` with `Relaxed` ordering so that generated message structs
/// are `Sync`. On all major platforms `Relaxed` load/store compiles to a plain
/// memory access — there is no runtime cost compared to `Cell<u32>`.
///
/// The maximum protobuf message size is 2 GiB, so `u32` is sufficient.
///
/// # Thread safety
///
/// `CachedSize` is `Send + Sync`. Serialization (`compute_size` followed by
/// `write_to`) must still be performed sequentially on one thread per message;
/// `Sync` simply means a message can be placed in an `Arc` or behind a shared
/// reference when no serialization is in progress.
#[derive(Debug)]
pub struct CachedSize {
    size: AtomicU32,
}

impl CachedSize {
    /// Create a new `CachedSize` with no cached value.
    #[inline]
    pub const fn new() -> Self {
        Self {
            size: AtomicU32::new(0),
        }
    }

    /// Get the cached size. Returns 0 if not yet computed.
    #[inline]
    pub fn get(&self) -> u32 {
        self.size.load(Ordering::Relaxed)
    }

    /// Set the cached size.
    #[inline]
    pub fn set(&self, size: u32) {
        self.size.store(size, Ordering::Relaxed);
    }
}

impl Default for CachedSize {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for CachedSize {
    #[inline]
    fn clone(&self) -> Self {
        // Don't propagate the cached value — the clone may diverge.
        Self::new()
    }
}

impl PartialEq for CachedSize {
    #[inline]
    fn eq(&self, _other: &Self) -> bool {
        // Cached size is not part of message equality.
        true
    }
}

impl Eq for CachedSize {}

impl core::hash::Hash for CachedSize {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, _state: &mut H) {
        // Cached size is not part of message identity; consistent with
        // `PartialEq` always returning `true`. Lets generated messages
        // derive `Hash`.
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for CachedSize {
    fn arbitrary(_u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(CachedSize::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time proof that CachedSize is Send + Sync.
    const _: fn() = || {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CachedSize>();
    };

    #[test]
    fn test_new_returns_zero() {
        assert_eq!(CachedSize::new().get(), 0);
    }

    #[test]
    fn test_default_returns_zero() {
        assert_eq!(CachedSize::default().get(), 0);
    }

    #[test]
    fn test_set_and_get_roundtrip() {
        let cs = CachedSize::new();
        cs.set(42);
        assert_eq!(cs.get(), 42);
        cs.set(0);
        assert_eq!(cs.get(), 0);
        cs.set(u32::MAX);
        assert_eq!(cs.get(), u32::MAX);
    }

    #[test]
    fn test_clone_resets_to_zero() {
        let cs = CachedSize::new();
        cs.set(100);
        let cloned = cs.clone();
        // Clone resets to zero so a diverged clone recomputes its size.
        assert_eq!(cloned.get(), 0);
        // Original is unchanged.
        assert_eq!(cs.get(), 100);
    }

    #[test]
    fn test_partial_eq_ignores_value() {
        let a = CachedSize::new();
        let b = CachedSize::new();
        a.set(1);
        b.set(99);
        // Cached size is not part of message equality.
        assert_eq!(a, b);
    }

    #[test]
    fn test_hash_ignores_value() {
        use core::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        fn hash(x: &CachedSize) -> u64 {
            let mut h = DefaultHasher::new();
            x.hash(&mut h);
            h.finish()
        }

        let a = CachedSize::new();
        let b = CachedSize::new();
        a.set(1);
        b.set(99);
        // Hash is consistent with Eq (both ignore the cached value).
        assert_eq!(hash(&a), hash(&b));
    }

    // Compile-time proof that a struct containing CachedSize and
    // UnknownFields can derive Hash.
    #[allow(dead_code)]
    #[derive(Hash)]
    struct MessageLike {
        value: i32,
        __cached: CachedSize,
        __unknown: crate::UnknownFields,
    }
}
