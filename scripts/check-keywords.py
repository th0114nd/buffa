#!/usr/bin/env python3
"""
Compare is_rust_keyword in buffa-codegen/src/idents.rs against the
authoritative keyword list in the Rust compiler source.

Downloads compiler/rustc_span/src/symbol.rs from the rust-lang/rust
stable branch on GitHub (no local toolchain configuration required).
"""

import re
import sys
import urllib.request

SYMBOL_RS_URL = (
    "https://raw.githubusercontent.com/rust-lang/rust/stable"
    "/compiler/rustc_span/src/symbol.rs"
)
IDENTS_RS_PATH = "buffa-codegen/src/idents.rs"

# Entries inside the Keywords block that are compiler-internal tokens,
# not keywords in the language sense.
EXCLUDE = {
    "_",       # Underscore — a pattern wildcard, not a keyword
}


def fetch_symbol_rs() -> str:
    print(f"Fetching {SYMBOL_RS_URL} ...", flush=True)
    try:
        req = urllib.request.Request(
            SYMBOL_RS_URL,
            headers={"User-Agent": "buffa-check-keywords/1.0"},
        )
        with urllib.request.urlopen(req, timeout=15) as resp:
            return resp.read().decode()
    except Exception as exc:
        print(f"error: failed to fetch symbol.rs: {exc}", file=sys.stderr)
        sys.exit(1)


def extract_rustc_keywords(content: str) -> set[str]:
    """
    Extract strict and reserved keyword strings from the Keywords { } block
    in symbol.rs, excluding weak keywords.

    Each entry has the form:
        IdentifierName:    "keyword_string",
    We want only plain-word strings (letters/digits/underscore); entries
    like "$crate" and "{{root}}" are compiler-internal and excluded.

    Weak keywords (union, default, auto, ...) appear after the comment
    "Weak keywords, have special meaning only in specific contexts." and
    are excluded because they can be used as identifiers without r#.
    """
    m = re.search(r"Keywords \{(.*?)\n    \}", content, re.DOTALL)
    if not m:
        print("error: could not locate Keywords { } block in symbol.rs", file=sys.stderr)
        sys.exit(1)
    block = m.group(1)

    # Discard everything from the weak-keywords subsection onwards.
    weak_start = re.search(r"Weak keywords", block)
    if weak_start:
        block = block[: weak_start.start()]

    all_quoted = re.findall(r'"([^"]+)"', block)
    keywords = {s for s in all_quoted if re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", s)}
    return keywords - EXCLUDE


def extract_our_keywords(path: str) -> set[str]:
    """Extract the string literals from the matches! block in is_rust_keyword."""
    with open(path) as f:
        content = f.read()
    m = re.search(
        r"fn is_rust_keyword\(name: &str\) -> bool \{.*?matches!\s*\(\s*\n?\s*name,\s*(.*?)\s*\)\s*\}",
        content,
        re.DOTALL,
    )
    if not m:
        print(f"error: could not locate is_rust_keyword in {path}", file=sys.stderr)
        sys.exit(1)
    return set(re.findall(r'"([^"]+)"', m.group(1)))


def main() -> None:
    rustc_keywords = extract_rustc_keywords(fetch_symbol_rs())
    our_keywords = extract_our_keywords(IDENTS_RS_PATH)

    missing = rustc_keywords - our_keywords
    extra = our_keywords - rustc_keywords

    ok = True

    if missing:
        print("Keywords in rustc but MISSING from is_rust_keyword:")
        for kw in sorted(missing):
            print(f"  {kw!r}")
        ok = False

    if extra:
        print("Keywords in is_rust_keyword but NOT in rustc Keywords block:")
        for kw in sorted(extra):
            print(f"  {kw!r}")
        ok = False

    if ok:
        print(f"OK: is_rust_keyword covers all {len(rustc_keywords)} rustc keywords.")

    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
