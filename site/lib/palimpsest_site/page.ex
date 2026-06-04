# SPDX-License-Identifier: MPL-2.0
# Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
defmodule PalimpsestSite.Page do
  @moduledoc """
  Page Data Model — Static Site Content.

  This module defines the internal representation of a documentation page. 
  It acts as the schema for Markdown files processed by NimblePublisher.
  """

  defstruct [:title, :slug, :description, :date, :content]

  @doc """
  CONSTRUCTOR: Hydrates a Page record from parsed Markdown metadata.
  - `attrs`: Key-value pairs extracted from the YAML front-matter.
  - `body`: The rendered HTML content of the page.
  """
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
