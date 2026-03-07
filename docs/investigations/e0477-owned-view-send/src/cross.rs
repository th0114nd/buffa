//! Cross-boundary ergonomic probes for `async fn` + auto-trait `OwnedView`.
//! These are the patterns that might surprise someone moving beyond a
//! self-contained handler.

use crate::{some_io, Owned, ServiceA, View};
use std::future::Future;

// ── P1: &req across await ────────────────────────────────────────────────
// The future holds `&Owned<View<'static>>`. For the future to be Send,
// that needs `Owned<View<'static>>: Sync`. The old manual Sync impl had
// the same `V: 'static` bound as Send — so if auto-Sync doesn't cover
// this, we'd have traded one E0477 for another.

struct P1;
impl ServiceA for P1 {
    async fn handle(&self, req: Owned<View<'static>>) -> String {
        let r: &Owned<View<'static>> = &req;
        some_io().await; // &req is live across this await
        r.0.name.to_string()
    }
}

// ── P2: helper takes OwnedView by value ──────────────────────────────────
// Standalone async fn always worked; confirming the chain composes.

async fn helper_by_value(req: Owned<View<'static>>) -> String {
    some_io().await;
    req.0.name.to_string()
}

struct P2;
impl ServiceA for P2 {
    async fn handle(&self, req: Owned<View<'static>>) -> String {
        helper_by_value(req).await
    }
}

// ── P3: helper takes &OwnedView ──────────────────────────────────────────
// The reborrowed `&req` is live while helper runs. Same Sync requirement
// as P1, just threaded through a call.

async fn helper_by_ref(req: &Owned<View<'static>>) -> usize {
    some_io().await;
    req.0.name.len()
}

struct P3;
impl ServiceA for P3 {
    async fn handle(&self, req: Owned<View<'static>>) -> String {
        let len = helper_by_ref(&req).await;
        // req is still usable — helper only borrowed.
        format!("{}: {}", req.0.name, len)
    }
}

// ── P4: generic helper over V ────────────────────────────────────────────
// What bound does the user need to write? Before: `V: Send + 'static`.
// Now: just `V: Send` (and `V: Sync` if borrowing across await).

async fn generic_helper<V: Send>(req: Owned<V>) -> Owned<V> {
    some_io().await;
    req
}

struct P4;
impl ServiceA for P4 {
    async fn handle(&self, req: Owned<View<'static>>) -> String {
        let req = generic_helper(req).await;
        req.0.name.to_string()
    }
}

// ── P5: nested trait impl (middleware-ish) ───────────────────────────────
// A second trait with the same RPITIT + Send shape, called from within
// the first. Both layers need to not hit E0477.

pub trait Inner: Send + Sync + 'static {
    fn inner(&self, req: Owned<View<'static>>) -> impl Future<Output = String> + Send;
}

struct InnerImpl;
impl Inner for InnerImpl {
    async fn inner(&self, req: Owned<View<'static>>) -> String {
        some_io().await;
        req.0.name.to_string()
    }
}

struct P5 {
    inner: InnerImpl,
}
impl ServiceA for P5 {
    async fn handle(&self, req: Owned<View<'static>>) -> String {
        self.inner.inner(req).await
    }
}

// ── P6: store in a struct, spawn later ──────────────────────────────────
// OwnedView is 'static, so storing it works. This isn't new — confirming
// nothing regressed.

struct Pending {
    req: Owned<View<'static>>,
}

fn _assert_pending_send(p: Pending) -> impl Send {
    p
}

// ── P7 (DOC CASE, expected to fail): extract a borrow, move THAT across
//     an async boundary separately from the OwnedView ────────────────────
// `req.0.name` is `&str` tied to `req`'s lifetime. You can't move the
// borrow into a spawned task because it doesn't outlive the handler.
// This is a fundamental borrow-checker constraint, not a Send/Sync issue.
// Correctly covered by the guide's "when to_owned_message() is needed".

#[cfg(feature = "doc_p7_fails")]
struct P7;
#[cfg(feature = "doc_p7_fails")]
impl ServiceA for P7 {
    async fn handle(&self, req: Owned<View<'static>>) -> String {
        let name: &str = req.0.name;
        // Can't do this — `name` borrows from `req`, which drops at end of fn.
        tokio::spawn(async move { name.to_string() });
        String::new()
    }
}
