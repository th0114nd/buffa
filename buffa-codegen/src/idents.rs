//! Rust identifier and path construction helpers.
//!
//! These are shared between buffa's codegen and downstream code generators
//! (e.g. `connectrpc-codegen`) that emit Rust code alongside buffa's message
//! types and need identical keyword-escaping and path-tokenization behavior.
//!
//! The guarantee is that if buffa generates `pub struct r#type::Foo { ... }`,
//! downstream callers using [`rust_path_to_tokens`]`("type::Foo")` produce the
//! matching `r#type::Foo` reference.

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

/// Parse a `::`-separated Rust path string into a [`TokenStream`], using raw
/// identifiers (`r#type`) for segments that are Rust keywords.
///
/// Used instead of `syn::parse_str::<syn::Type>` because the latter cannot
/// handle raw identifiers in path position: `"google::type::LatLng"` would
/// fail to parse because `type` is a keyword, but this function correctly
/// produces `google::r#type::LatLng`.
///
/// Path-position keywords (`self`, `super`, `Self`, `crate`) are emitted as
/// plain idents (they're valid in paths) — this differs from
/// [`make_field_ident`], which suffixes them with `_`.
///
/// Leading `::` (absolute path, e.g. `"::buffa::Message"`) is preserved.
///
/// # Panics
///
/// Panics (in debug) if `path` is empty.
pub fn rust_path_to_tokens(path: &str) -> TokenStream {
    debug_assert!(
        !path.is_empty(),
        "rust_path_to_tokens called with empty path"
    );

    // Handle absolute paths (starting with `::`, e.g. extern crate paths).
    let (prefix, rest) = if let Some(stripped) = path.strip_prefix("::") {
        (quote! { :: }, stripped)
    } else {
        (TokenStream::new(), path)
    };

    // For path segments, non-raw-able keywords (`self`, `super`, `Self`,
    // `crate`) are emitted as plain idents because they are valid in path
    // position. This differs from `make_field_ident`, which appends `_` for
    // these keywords since they are invalid as struct field names.
    let segments: Vec<Ident> = rest
        .split("::")
        .map(|seg| {
            if is_rust_keyword(seg) && can_be_raw_ident(seg) {
                Ident::new_raw(seg, Span::call_site())
            } else {
                Ident::new(seg, Span::call_site())
            }
        })
        .collect();

    quote! { #prefix #(#segments)::* }
}

/// Create a field identifier, escaping Rust keywords.
///
/// Most keywords use raw identifiers (`r#type`). The keywords `self`, `super`,
/// `Self`, `crate` cannot be raw identifiers and are suffixed with `_` instead
/// (e.g. `self_`), matching prost's convention.
pub fn make_field_ident(name: &str) -> Ident {
    if is_rust_keyword(name) {
        if can_be_raw_ident(name) {
            Ident::new_raw(name, Span::call_site())
        } else {
            format_ident!("{}_", name)
        }
    } else {
        format_ident!("{}", name)
    }
}

/// Escape a proto package segment for use as a Rust `mod` name.
///
/// Returns `r#` prefix for raw-able keywords, `_` suffix for path-position
/// keywords (which can't be raw), and the name as-is otherwise.
///
/// This is a `String` (not `Ident`) because callers typically emit it into
/// source text (e.g. `pub mod {name} { ... }` via `format!`), not via `quote!`.
pub fn escape_mod_ident(name: &str) -> String {
    if is_rust_keyword(name) {
        if can_be_raw_ident(name) {
            format!("r#{name}")
        } else {
            format!("{name}_")
        }
    } else {
        name.to_string()
    }
}

/// Is `name` a Rust keyword (strict, edition-2018+, edition-2024+, or reserved)?
///
/// Covers all editions up to 2024. See `scripts/check-keywords.py` for the
/// maintenance script that diffs this list against the upstream rustc source.
pub fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        // Strict keywords — all editions
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            // Strict keywords — edition 2018+
            | "async"
            | "await"
            | "dyn"
            // Strict keywords — edition 2024+
            | "gen"
            // Reserved for future use (all editions)
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

/// Can `name` be used as a raw identifier (`r#name`)?
///
/// `self`, `super`, `Self`, `crate` are valid path segments and cannot be
/// prefixed with `r#`. They get a `_` suffix in field/mod position instead.
fn can_be_raw_ident(name: &str) -> bool {
    !matches!(name, "self" | "super" | "Self" | "crate")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_path_simple() {
        assert_eq!(rust_path_to_tokens("Foo").to_string(), "Foo");
    }

    #[test]
    fn rust_path_nested() {
        assert_eq!(
            rust_path_to_tokens("foo::bar::Baz").to_string(),
            "foo :: bar :: Baz"
        );
    }

    #[test]
    fn rust_path_keyword_segment() {
        // `type` is a keyword → raw identifier.
        assert_eq!(
            rust_path_to_tokens("google::type::LatLng").to_string(),
            "google :: r#type :: LatLng"
        );
    }

    #[test]
    fn rust_path_absolute() {
        assert_eq!(
            rust_path_to_tokens("::buffa::Message").to_string(),
            ":: buffa :: Message"
        );
    }

    #[test]
    fn rust_path_super_segment() {
        // `super` is valid in path position → plain ident (no r# or _).
        assert_eq!(
            rust_path_to_tokens("super::super::Foo").to_string(),
            "super :: super :: Foo"
        );
    }

    #[test]
    fn field_ident_normal() {
        assert_eq!(make_field_ident("foo").to_string(), "foo");
    }

    #[test]
    fn field_ident_keyword() {
        assert_eq!(make_field_ident("type").to_string(), "r#type");
    }

    #[test]
    fn field_ident_non_raw_keyword() {
        // `self` can't be r#self → suffixed.
        assert_eq!(make_field_ident("self").to_string(), "self_");
        assert_eq!(make_field_ident("super").to_string(), "super_");
        assert_eq!(make_field_ident("crate").to_string(), "crate_");
        assert_eq!(make_field_ident("Self").to_string(), "Self_");
    }

    #[test]
    fn escape_mod_normal() {
        assert_eq!(escape_mod_ident("foo"), "foo");
    }

    #[test]
    fn escape_mod_keyword() {
        assert_eq!(escape_mod_ident("type"), "r#type");
        assert_eq!(escape_mod_ident("async"), "r#async");
    }

    #[test]
    fn escape_mod_non_raw_keyword() {
        assert_eq!(escape_mod_ident("self"), "self_");
        assert_eq!(escape_mod_ident("super"), "super_");
    }

    #[test]
    fn keyword_coverage() {
        assert!(is_rust_keyword("type"));
        assert!(is_rust_keyword("async"));
        assert!(is_rust_keyword("gen")); // 2024
        assert!(is_rust_keyword("yield")); // reserved
        assert!(!is_rust_keyword("foo"));
        assert!(!is_rust_keyword("Type")); // case-sensitive
    }
}
