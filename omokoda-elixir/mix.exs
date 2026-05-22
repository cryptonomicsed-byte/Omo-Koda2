defmodule Yemoja.MixProject do
  use Mix.Project

  def project do
    [
      app: :yemoja,
      version: "0.1.0",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      test_coverage: [tool: ExCoveralls],
      preferred_cli_env: [
        coveralls: :test,
        "coveralls.detail": :test,
        "coveralls.post": :test,
        "coveralls.html": :test
      ]
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
      {:grpc, "~> 0.8"},
      {:protobuf, "~> 0.13"}
    ]
  end
end
