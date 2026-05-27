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
    []
  end
end
