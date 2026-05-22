// SPDX-License-Identifier: PPMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// plasma badge — generate shields.io badge markdown for PPMPL-1.0-or-later.

use anyhow::Result;

/// Generate badge markdown for different documentation formats.
pub fn run(style: &str) -> Result<()> {
    let badge_url = format!(
        "https://img.shields.io/badge/License-PMPL--2.0-blue.svg?style={style}"
    );
    let link_url = "https://github.com/hyperpolymath/palimpsest-license";

    println!("Palimpsest-MPL 2.0 Badge");
    println!();

    // Markdown
    println!("Markdown:");
    println!(
        "  [![License: PPMPL-1.0-or-later]({badge_url})]({link_url})"
    );
    println!();

    // AsciiDoc
    println!("AsciiDoc:");
    println!(
        "  image:{badge_url}[License: PPMPL-1.0-or-later,link=\"{link_url}\"]"
    );
    println!();

    // HTML
    println!("HTML:");
    println!(
        "  <a href=\"{link_url}\"><img src=\"{badge_url}\" alt=\"License: PPMPL-1.0-or-later\"></a>"
    );
    println!();

    // reStructuredText
    println!("reStructuredText:");
    println!(
        "  .. image:: {badge_url}\n     :target: {link_url}\n     :alt: License: PPMPL-1.0-or-later"
    );
    println!();

    // With Covenant
    let covenant_url = format!(
        "https://img.shields.io/badge/Covenant-Palimpsest-indigo.svg?style={style}"
    );
    let covenant_link = "https://github.com/hyperpolymath/palimpsest-license/blob/main/PALIMPSEST-COVENANT.md";

    println!("With Covenant badge (Markdown):");
    println!(
        "  [![License: PPMPL-1.0-or-later]({badge_url})]({link_url}) [![Covenant: Palimpsest]({covenant_url})]({covenant_link})"
    );

    Ok(())
}
