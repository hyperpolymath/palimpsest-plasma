// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// plasma badge — generate shields.io license badge markup.

use anyhow::Result;

/// Generate badge markup for different documentation formats.
pub fn run(license: &str, style: &str) -> Result<()> {
    // shields.io escapes: '-' in label text becomes '--'.
    let escaped = license.replace('-', "--");
    let badge_url =
        format!("https://img.shields.io/badge/License-{escaped}-blue.svg?style={style}");
    let link_url = format!("https://spdx.org/licenses/{license}.html");

    println!("{license} Badge");
    println!();

    // Markdown
    println!("Markdown:");
    println!("  [![License: {license}]({badge_url})]({link_url})");
    println!();

    // AsciiDoc
    println!("AsciiDoc:");
    println!("  image:{badge_url}[License: {license},link=\"{link_url}\"]");
    println!();

    // HTML
    println!("HTML:");
    println!("  <a href=\"{link_url}\"><img src=\"{badge_url}\" alt=\"License: {license}\"></a>");
    println!();

    // reStructuredText
    println!("reStructuredText:");
    println!("  .. image:: {badge_url}\n     :target: {link_url}\n     :alt: License: {license}");

    Ok(())
}
