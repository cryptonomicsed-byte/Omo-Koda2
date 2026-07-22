defmodule Yemoja.MixProject do
  use Mix.Project

  def project do
    [
      app: :yemoja,
      version: "0.1.0",
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      extra_applications: [:logger, :crypto],
      mod: {Yemoja.Application, []}
    ]
  end

  defp deps do
    [
      # plug_cowboy 2.6 alone resolves plug >=1.16, which requires Elixir
      # >=1.15 -- the VPS runs 1.14.0, so plug is pinned explicitly to the
      # last release that still supports it.
      {:plug, "~> 1.15.0", override: true},
      {:plug_cowboy, "~> 2.6"},
      {:jason, "~> 1.4"},
    ]
  end
end
