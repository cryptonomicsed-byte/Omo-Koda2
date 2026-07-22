defmodule Yemoja.Router do
  @moduledoc """
  HTTP API for the Yemọja swarm service.
  Consumed by HttpYemojaClient in omokoda-core/src/bus/clients.rs.

  Routes:
    POST /spawn_agent          ← {role, budget_synapse}  → {agent_id, status}
    GET  /agent_status/:id     → {agent_id, status}
    GET  /mesh/presence/:block → {agents: [...]}
    POST /mesh/broadcast/:block ← event JSON → {broadcast: true}
    POST /mesh/consensus/:block ← proposal JSON → {result: ...}
    POST /mesh/handoff          ← {agent_id, target_node} → {ok: true}
  """

  use Plug.Router

  plug Plug.Parsers,
    parsers: [:json],
    json_decoder: Jason

  plug :match
  plug :dispatch

  post "/spawn_agent" do
    role   = Map.get(conn.body_params, "role", "worker")
    _budget = Map.get(conn.body_params, "budget_synapse", 1000.0)
    agent_id = "agent-#{:erlang.unique_integer([:positive, :monotonic])}"

    case DynamicSupervisor.start_child(
           Yemoja.AgentSupervisor,
           {Yemoja.AgentWorker, [id: agent_id, model: String.to_atom(role)]}
         ) do
      {:ok, _pid} ->
        json(conn, 200, %{agent_id: agent_id, status: "spawned"})

      {:error, reason} ->
        json(conn, 500, %{error: inspect(reason)})
    end
  end

  get "/agent_status/:agent_id" do
    status =
      case Registry.lookup(Yemoja.Registry, {Yemoja.AgentWorker, agent_id}) do
        [{pid, _}] -> if Process.alive?(pid), do: "running", else: "complete"
        []         -> "idle"
      end

    json(conn, 200, %{agent_id: agent_id, status: status})
  end

  get "/mesh/presence/:block_id" do
    agents =
      Yemoja.Registry
      |> Registry.select([{{:"$1", :"$2", :"$3"}, [], [{{:"$1", :"$2"}}]}])
      |> Enum.flat_map(fn {{Yemoja.AgentWorker, id}, _pid} ->
        [%{agent_id: id, role: "agent", status: "running", block_id: block_id}]
      end)

    json(conn, 200, %{agents: agents})
  end

  post "/mesh/broadcast/:block_id" do
    require Logger
    Logger.debug("mesh_broadcast block=#{block_id} event=#{inspect(conn.body_params)}")
    json(conn, 200, %{broadcast: true, block_id: block_id})
  end

  post "/mesh/consensus/:block_id" do
    proposal = conn.body_params
    agents =
      Registry.select(Yemoja.Registry, [{{:"$1", :"$2", :"$3"}, [], [{{:"$1", :"$2"}}]}])

    votes =
      Enum.map(agents, fn {{Yemoja.AgentWorker, _id}, _pid} ->
        :accept
      end)

    accepted = Enum.count(votes, &(&1 == :accept))
    quorum   = max(1, div(length(votes), 2) + 1)
    result   = if accepted >= quorum, do: "accepted", else: "rejected"

    json(conn, 200, %{block_id: block_id, proposal: proposal, result: result, votes: length(votes)})
  end

  post "/mesh/handoff" do
    agent_id    = Map.get(conn.body_params, "agent_id", "")
    target_node = Map.get(conn.body_params, "target_node", "")

    case Registry.lookup(Yemoja.Registry, {Yemoja.AgentWorker, agent_id}) do
      [{_pid, _}] ->
        require Logger
        Logger.info("mesh_handoff agent=#{agent_id} target=#{target_node}")
        json(conn, 200, %{ok: true, agent_id: agent_id, target_node: target_node})

      [] ->
        json(conn, 404, %{error: "agent not found", agent_id: agent_id})
    end
  end

  get "/health" do
    json(conn, 200, %{ok: true, service: "yemoja"})
  end

  # SkillForge Creation stage: build the SkillManifestEntry from Analysis
  # (Clojure classification) + Memory (Julia dedup) results. Elixir's role
  # here is genuinely a fit for "Creation" — assembling a manifest from
  # component parts is exactly the supervision-tree-style composition this
  # service already does for agent lifecycle, just applied to a data
  # structure instead of a process tree.
  post "/skillforge/manifest" do
    body = conn.body_params
    name = Map.get(body, "name", "unknown-skill")
    classification = Map.get(body, "classification", "Unknown")
    language = Map.get(body, "language", "unknown")
    description = Map.get(body, "description", "")
    base_url_hint = Map.get(body, "base_url_hint")
    auth_hint = Map.get(body, "auth_hint")
    candidate_routes = Map.get(body, "candidate_routes", %{})
    risk_signals = Map.get(body, "risk_signals", [])

    write =
      candidate_routes
      |> Map.values()
      |> Enum.any?(&(not String.starts_with?(&1, "GET")))
      |> Kernel.or(risk_signals != [])

    base_url =
      base_url_hint || "${#{String.upcase(String.replace(name, "-", "_"))}_URL}"

    routes =
      if map_size(candidate_routes) == 0 do
        %{"health" => "GET /health", "discover" => "GET /"}
      else
        candidate_routes
      end

    manifest = %{
      name: name,
      description: "#{description} [forged by SkillForge from #{classification}; lang=#{language}]",
      base_url: base_url,
      auth_header: get_in(auth_hint, ["header"]),
      auth_env: get_in(auth_hint, ["env"]),
      auth_value: nil,
      required_tier: if(write, do: 2, else: 1),
      write: write,
      routes: routes
    }

    json(conn, 200, manifest)
  end

  match _ do
    json(conn, 404, %{error: "not found", path: conn.request_path})
  end

  defp json(conn, status, body) do
    conn
    |> put_resp_content_type("application/json")
    |> send_resp(status, Jason.encode!(body))
  end
end
