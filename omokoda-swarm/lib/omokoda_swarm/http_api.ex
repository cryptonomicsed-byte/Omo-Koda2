defmodule OmokodaSwarm.HttpApi do
  @moduledoc """
  Minimal dependency-free HTTP surface for the swarm supervisor tree.

  `mix.exs` intentionally has no deps (this is a pure-OTP application), so
  this uses `:gen_tcp` directly rather than pulling in Phoenix/Cowboy via
  Hex -- a handful of JSON GET/POST routes is not worth the dependency
  weight. Mirrors the CORS behavior of the Rust kernel (:7777), LOOM
  (:8889), and the Julia memory service (:7778) so the Axiom dashboard can
  reach it directly from the browser.

  Routes:
    GET  /health             -> {"ok": true}
    GET  /status              -> OmokodaSwarm.Coordinator.get_status/0
    GET  /rem/last_plan       -> OmokodaSwarm.Memory.RemCycle.last_plan/0
    POST /rem/run_now         -> OmokodaSwarm.Memory.RemCycle.run_now/0
    POST /spawn_agent         -> real subagent: births a genuine guest
                                  agent on the Rust Steward (see
                                  StewardClient.birth/2), then registers a
                                  supervised GenServer for it here so it's
                                  independently crash-isolated and shows up
                                  in /status like any other swarm agent.
                                  Body: {"role": "...", "budget_synapse": N}
                                  -> {"agent_id": "<real rust agent id>"}
                                  This is the endpoint
                                  omokoda-core::bus::clients::HttpYemojaClient
                                  ::spawn_agent posts to -- previously
                                  nonexistent, which meant that Rust-side
                                  client's calls always 404'd.
    GET  /agent_status/:id    -> OmokodaSwarm.Agent.get_state/1 for one agent
  """

  use GenServer
  require Logger

  @port 4000

  def start_link(_opts), do: GenServer.start_link(__MODULE__, [], name: __MODULE__)

  @impl true
  def init([]) do
    case :gen_tcp.listen(@port, [:binary, packet: :raw, active: false, reuseaddr: true]) do
      {:ok, listen_socket} ->
        Logger.info("[HttpApi] listening on :#{@port}")
        pid = spawn_link(fn -> accept_loop(listen_socket) end)
        {:ok, %{listen_socket: listen_socket, acceptor: pid}}

      {:error, reason} ->
        Logger.warning("[HttpApi] failed to bind :#{@port} — #{inspect(reason)}, HTTP surface disabled")
        {:ok, %{listen_socket: nil, acceptor: nil}}
    end
  end

  defp accept_loop(listen_socket) do
    case :gen_tcp.accept(listen_socket) do
      {:ok, socket} ->
        spawn(fn -> handle_conn(socket) end)
        accept_loop(listen_socket)

      {:error, reason} ->
        Logger.warning("[HttpApi] accept failed — #{inspect(reason)}")
    end
  end

  defp handle_conn(socket) do
    with {:ok, data} <- :gen_tcp.recv(socket, 0, 5_000),
         {:ok, method, path} <- parse_request_line(data) do
      respond(socket, method, path, extract_body(data))
    else
      _ -> :ok
    end

    :gen_tcp.close(socket)
  end

  defp parse_request_line(data) do
    case String.split(data, "\r\n", parts: 2) do
      [request_line | _] ->
        case String.split(request_line, " ") do
          [method, path, _version] -> {:ok, method, path}
          _ -> :error
        end

      _ ->
        :error
    end
  end

  # Everything after the blank line separating headers from body. No
  # Transfer-Encoding/chunked support and no re-read if the body arrives
  # in a later TCP segment -- matches this module's existing "small local
  # JSON request, one recv is enough" assumption (already relied on by
  # every route here), just extended to also cover a request body.
  defp extract_body(data) do
    case String.split(data, "\r\n\r\n", parts: 2) do
      [_headers, body] -> body
      _ -> ""
    end
  end

  defp respond(socket, "OPTIONS", _path, _body) do
    write(socket, 204, "")
  end

  defp respond(socket, "GET", "/health", _body) do
    write_json(socket, 200, %{ok: true})
  end

  defp respond(socket, "GET", "/status", _body) do
    try do
      status = OmokodaSwarm.Coordinator.get_status()
      write_json(socket, 200, safe_status(status))
    catch
      :exit, reason -> write_json(socket, 503, %{error: "coordinator unavailable: #{inspect(reason)}"})
    end
  end

  defp respond(socket, "GET", "/rem/last_plan", _body) do
    plan = OmokodaSwarm.Memory.RemCycle.last_plan()
    write_json(socket, 200, %{plan: plan})
  catch
    :exit, reason -> write_json(socket, 503, %{error: "rem cycle unavailable: #{inspect(reason)}"})
  end

  defp respond(socket, "POST", "/rem/run_now", _body) do
    case OmokodaSwarm.Memory.RemCycle.run_now() do
      {:ok, plan} -> write_json(socket, 200, %{plan: plan})
      {:error, reason} -> write_json(socket, 502, %{error: inspect(reason)})
    end
  catch
    :exit, reason -> write_json(socket, 503, %{error: "rem cycle unavailable: #{inspect(reason)}"})
  end

  # Real subagent spawn: births a genuine guest agent on the Rust Steward
  # (StewardClient.birth/2, non-sovereign -- see server.rs's birth_handler),
  # then registers a supervised GenServer here pre-populated with the real
  # agent_id/agent_key so its own future :think/:act dispatches (if driven
  # through Agent.delegate_task) address that same guest. Synchronous:
  # the caller (Rust's HttpYemojaClient::spawn_agent) needs the real
  # agent_id back in the HTTP response, not delivered later via telemetry.
  defp respond(socket, "POST", "/spawn_agent", body) do
    case OmokodaSwarm.JSON.decode(body) do
      {:ok, %{"role" => role} = params} ->
        budget = Map.get(params, "budget_synapse", 0)
        suffix = :crypto.strong_rand_bytes(4) |> Base.encode16(case: :lower)
        birth_name = "subagent-#{role}-#{suffix}"

        case OmokodaSwarm.StewardClient.birth(birth_name, [%{key: "spawned_via", value: "swarm"}]) do
          {:ok, %{"agent_id" => agent_id} = birth_result} when is_binary(agent_id) ->
            agent_key = Map.get(birth_result, "agent_key", agent_id)

            OmokodaSwarm.SwarmSupervisor.start_agent(agent_id, %{
              role: String.to_atom(role),
              budget_synapse: budget,
              guest_agent_id: agent_id,
              guest_agent_key: agent_key
            })

            write_json(socket, 200, %{agent_id: agent_id})

          {:ok, _malformed} ->
            write_json(socket, 502, %{error: "birth succeeded but response had no agent_id"})

          {:error, reason} ->
            write_json(socket, 502, %{error: "steward birth failed: #{inspect(reason)}"})
        end

      {:ok, _no_role} ->
        write_json(socket, 422, %{error: "\"role\" is required"})

      {:error, reason} ->
        write_json(socket, 422, %{error: "invalid JSON body: #{inspect(reason)}"})
    end
  end

  defp respond(socket, "GET", "/agent_status/" <> agent_id, _body) when agent_id != "" do
    case OmokodaSwarm.Agent.get_state(agent_id) do
      {:ok, public} -> write_json(socket, 200, safe_status(public))
      {:error, :agent_not_found} -> write_json(socket, 404, %{error: "unknown agent_id"})
    end
  end

  # SkillForge Creation stage: assemble the SkillManifestEntry. Consolidated
  # here from the smaller, now-retired standalone omokoda-elixir/Yemoja
  # service -- this is the real, deployed Yemoja, so new Yemoja capability
  # lands on this router going forward.
  defp respond(socket, "POST", "/skillforge/manifest", body) do
    case OmokodaSwarm.JSON.decode(body) do
      {:ok, params} ->
        name = Map.get(params, "name", "unknown-skill")
        classification = Map.get(params, "classification", "Unknown")
        language = Map.get(params, "language", "unknown")
        description = Map.get(params, "description", "")
        base_url_hint = Map.get(params, "base_url_hint")
        auth_hint = Map.get(params, "auth_hint")
        candidate_routes = Map.get(params, "candidate_routes", %{})
        risk_signals = Map.get(params, "risk_signals", [])

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

        write_json(socket, 200, %{
          name: name,
          description: "#{description} [forged by SkillForge from #{classification}; lang=#{language}]",
          base_url: base_url,
          auth_header: get_in(auth_hint, ["header"]),
          auth_env: get_in(auth_hint, ["env"]),
          auth_value: nil,
          required_tier: if(write, do: 2, else: 1),
          write: write,
          routes: routes
        })

      {:error, reason} ->
        write_json(socket, 422, %{error: "invalid JSON body: #{inspect(reason)}"})
    end
  end

  defp respond(socket, _method, _path, _body) do
    write_json(socket, 404, %{error: "not found"})
  end

  # `get_status/0`'s return can hold PIDs/refs that aren't JSON-encodable —
  # stringify anything OmokodaSwarm.Json can't handle rather than 500ing.
  defp safe_status(status) when is_map(status) do
    Map.new(status, fn {k, v} -> {k, safe_value(v)} end)
  end

  defp safe_status(status), do: safe_value(status)

  defp safe_value(v) when is_pid(v) or is_reference(v) or is_function(v), do: inspect(v)
  defp safe_value(v) when is_map(v), do: safe_status(v)
  defp safe_value(v) when is_list(v), do: Enum.map(v, &safe_value/1)
  defp safe_value(v), do: v

  defp write_json(socket, status, data) do
    write(socket, status, OmokodaSwarm.JSON.encode!(data), "application/json")
  end

  defp write(socket, status, body, content_type \\ "text/plain") do
    reason = reason_phrase(status)

    headers = [
      "HTTP/1.1 #{status} #{reason}",
      "Content-Type: #{content_type}",
      "Content-Length: #{byte_size(body)}",
      "Access-Control-Allow-Origin: *",
      "Access-Control-Allow-Methods: GET, POST, OPTIONS",
      "Access-Control-Allow-Headers: Content-Type",
      "Connection: close",
      "",
      ""
    ]

    :gen_tcp.send(socket, Enum.join(headers, "\r\n") <> body)
  end

  defp reason_phrase(200), do: "OK"
  defp reason_phrase(204), do: "No Content"
  defp reason_phrase(404), do: "Not Found"
  defp reason_phrase(502), do: "Bad Gateway"
  defp reason_phrase(503), do: "Service Unavailable"
  defp reason_phrase(_), do: "Error"
end
