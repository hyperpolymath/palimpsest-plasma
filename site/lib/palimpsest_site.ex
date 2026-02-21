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

  # ... [Private rendering and path helpers]
end
