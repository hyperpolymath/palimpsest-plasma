<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
%{
  title: "Releases",
  slug: "releases",
  description: "Release history and how to build from source.",
  date: ~D[2026-07-02]
}
---

## v0.2.0 — Engine v0

The current release is a deterministic, typed policy engine. See
`docs/release-notes.adoc` and `CHANGELOG.md` for the full list of changes.

Highlights:

* `plasma-engine` — policy AST, versioned schema, pure evaluator
* `plasma check` / `facts` / `policy validate`, plus parser-backed `audit`
* Licensing unified: code MPL-2.0, documentation CC-BY-SA-4.0

## Building from source

```
git clone https://github.com/hyperpolymath/palimpsest-plasma.git
cd palimpsest-plasma
cargo build --release
cargo test --workspace
```

The binary is `target/release/plasma`.
