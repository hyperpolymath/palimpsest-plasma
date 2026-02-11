defmodule PalimpsestSite do
  use NimblePublisher,
    build: PalimpsestSite.Page,
    from: "content/*.md",
    as: :pages

  @output_dir Path.expand("../_site", __DIR__)
  @template_path Path.expand("../templates/default.html.eex", __DIR__)
  @assets_dir Path.expand("../assets", __DIR__)

  def pages, do: @pages

  def build do
    pages = pages()
    template = File.read!(@template_path)

    File.rm_rf!(@output_dir)
    File.mkdir_p!(@output_dir)
    copy_assets()

    Enum.each(pages, fn page ->
      body = render_page(template, page, pages)
      page_path = Path.join(@output_dir, page_filename(page.slug))
      File.mkdir_p!(Path.dirname(page_path))
      File.write!(page_path, body)
    end)
  end

  defp render_page(template, page, pages) do
    assigns = [
      page: page,
      content: page.content,
      nav: nav_items(pages),
      site_title: "Palimpsest Plasma"
    ]

    EEx.eval_string(template, assigns)
  end

  defp nav_items(pages) do
    Enum.map(pages, fn p ->
      %{title: p.title, url: page_filename(p.slug)}
    end)
  end

  defp page_filename("index"), do: "index.html"
  defp page_filename(slug), do: "#{slug}.html"

  defp copy_assets do
    File.cp_r!(@assets_dir, Path.join(@output_dir, "assets"))
  end
end
