%{
  title: "Palimpsest Plasma MVP",
  slug: "index",
  description: "The compliance bundle for PMPL-1.0 projects.",
  date: ~D[2026-02-11]
}
---

## Overview

Palimpsest Plasma packages the Palimpsest-MPL License 1.0 assets, badge pack, and audit documentation so projects can adopt **PMPL-1.0-or-later** confidently.

- **License text & metadata:** `LICENSE`, `.machine_readable/ECOSYSTEM.scm`, `.machine_readable/STATE.scm`
- **Audit and compliance:** `docs/release-log.adoc`, `docs/release-notes.adoc`, `docs/mvp-v1.adoc`
- **Badge assets:** `assets/badges/svg/badge-standard.svg`

## Building locally

Install Elixir (1.15+), then from `/var/mnt/eclipse/repos/palimpsest-plasma/site` run `mix site.build`. The command regenerates `/var/mnt/eclipse/repos/palimpsest-plasma/site/_site` which GitHub Actions uploads to Pages.
