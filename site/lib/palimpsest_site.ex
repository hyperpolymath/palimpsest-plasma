# SPDX-License-Identifier: MPL-2.0
# Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
defmodule PalimpsestSite do
  @moduledoc """
  Palimpsest Plasma — Static Site Generator (SSG).

  This module implements the automated publishing pipeline for the 
  Palimpsest documentation site. It processes Markdown content from the 
  `content/` directory and renders verified HTML using EEx templates.

  ## Pipeline:
  1. **Ingest**: Scans `content/*.md` for pages and metadata.
  2. **Cleanup**: Wipes the `_site/` directory to ensure no stale artifacts.
  3. **Assets**: Synchronizes CSS and image assets to the output path.
  4. **Render**: Evaluates the `default.html.eex` template for every page.
  """

  use NimblePublisher,
    build: PalimpsestSite.Page,
    from: "content/*.md",
    as: :pages

  @output_dir Path.expand("../_site", __DIR__)
  @template_path Path.expand("../templates/default.html.eex", __DIR__)

  @doc """
  BUILD: Orchestrates the full static site generation cycle.
  """
  def build do
    pages = pages()
    template = File.read!(@template_path)

    # REFRESH: Atomic reset of the output directory.
    File.rm_rf!(@output_dir)
    File.mkdir_p!(@output_dir)
    copy_assets()

    # GENERATE: Iteratively render and write each Markdown page.
    Enum.each(pages, fn page ->
      body = render_page(template, page, pages)
      page_path = Path.join(@output_dir, page_filename(page.slug))
      File.mkdir_p!(Path.dirname(page_path))
      File.write!(page_path, body)
    end)
  end

  # Render one page by evaluating the EEx template with the bindings the
  # template expects as bare variables: `page`, `nav` (all pages, for the
  # menu), `content`, and `site_title`.
  defp render_page(template, page, pages) do
    EEx.eval_string(template,
      page: page,
      nav: pages,
      content: page.content,
      site_title: "Palimpsest Plasma"
    )
  end

  # Copy the static asset directory into the output tree, if present.
  defp copy_assets do
    assets_src = Path.expand("../assets", __DIR__)

    if File.dir?(assets_src) do
      File.cp_r!(assets_src, Path.join(@output_dir, "assets"))
    end
  end

  # Map a page slug to its output filename. The "index" slug becomes the
  # site root; everything else gets its own directory-style path.
  defp page_filename("index"), do: "index.html"
  defp page_filename(slug), do: "#{slug}.html"
end
