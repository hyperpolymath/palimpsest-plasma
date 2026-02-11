defmodule PalimpsestSite.MixProject do
  use Mix.Project

  def project do
    [
      app: :palimpsest_site,
      version: "0.1.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      aliases: aliases()
    ]
  end

  def application do
    [extra_applications: [:logger]]
  end

  defp deps do
    [
      {:nimble_publisher, "~> 0.1"},
      {:earmark, "~> 1.4"}
    ]
  end

  defp aliases do
    [
      "site.build": ["run -e 'PalimpsestSite.build()'"]
    ]
  end
end
