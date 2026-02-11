defmodule PalimpsestSite.Page do
  defstruct [:title, :slug, :description, :date, :content]

  def build(_path, attrs, body) do
    %PalimpsestSite.Page{
      title: Map.fetch!(attrs, :title),
      slug: Map.fetch!(attrs, :slug),
      description: Map.get(attrs, :description),
      date: Map.get(attrs, :date),
      content: String.trim(body)
    }
  end
end
