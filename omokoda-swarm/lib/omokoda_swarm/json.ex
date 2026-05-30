defmodule OmokodaSwarm.JSON do
  @moduledoc false
  # Minimal JSON encoder/decoder — no external deps required.
  # Encoder handles the maps/lists/primitives we actually send to the Rust/Julia APIs.
  # Decoder handles the responses those APIs return.

  # ---------------------------------------------------------------------------
  # Encode
  # ---------------------------------------------------------------------------

  def encode!(term), do: encode(term)

  defp encode(nil), do: "null"
  defp encode(true), do: "true"
  defp encode(false), do: "false"
  defp encode(n) when is_integer(n), do: Integer.to_string(n)
  defp encode(n) when is_float(n), do: :erlang.float_to_binary(n, [:compact, decimals: 10])
  defp encode(s) when is_binary(s), do: [?", escape(s), ?"] |> IO.iodata_to_binary()
  defp encode(a) when is_atom(a), do: encode(Atom.to_string(a))

  defp encode(list) when is_list(list) do
    inner = Enum.map_join(list, ",", &encode/1)
    "[#{inner}]"
  end

  defp encode(map) when is_map(map) do
    pairs =
      map
      |> Enum.sort_by(fn {k, _} -> to_string(k) end)
      |> Enum.map_join(",", fn {k, v} -> "#{encode(to_string(k))}:#{encode(v)}" end)
    "{#{pairs}}"
  end

  defp escape(s) do
    s
    |> String.replace("\\", "\\\\")
    |> String.replace("\"", "\\\"")
    |> String.replace("\n", "\\n")
    |> String.replace("\r", "\\r")
    |> String.replace("\t", "\\t")
  end

  # ---------------------------------------------------------------------------
  # Decode
  # ---------------------------------------------------------------------------

  def decode!(bin) when is_binary(bin) do
    case decode(bin) do
      {:ok, val} -> val
      {:error, reason} -> raise "JSON decode error: #{inspect(reason)}"
    end
  end

  def decode(bin) when is_binary(bin) do
    try do
      {val, rest} = parse_value(String.trim(bin))
      case String.trim(rest) do
        "" -> {:ok, val}
        r -> {:error, {:trailing, r}}
      end
    rescue
      e -> {:error, e}
    end
  end

  defp parse_value("null" <> rest), do: {nil, rest}
  defp parse_value("true" <> rest), do: {true, rest}
  defp parse_value("false" <> rest), do: {false, rest}

  defp parse_value(<<?" , rest::binary>>) do
    {str, after_str} = parse_string_body(rest, [])
    {str, after_str}
  end

  defp parse_value(<<?{, rest::binary>>) do
    parse_object(String.trim_leading(rest), %{})
  end

  defp parse_value(<<?[, rest::binary>>) do
    parse_array(String.trim_leading(rest), [])
  end

  defp parse_value(bin) do
    parse_number(bin)
  end

  # Object parsing
  defp parse_object(<<?}, rest::binary>>, acc), do: {acc, rest}

  defp parse_object(bin, acc) do
    {key, after_key} = parse_value(String.trim_leading(bin))
    after_colon = String.trim_leading(String.trim_leading(after_key), ":")
    {val, after_val} = parse_value(String.trim_leading(after_colon))
    new_acc = Map.put(acc, key, val)

    case String.trim_leading(after_val) do
      <<?,, rest::binary>> -> parse_object(String.trim_leading(rest), new_acc)
      <<?}, rest::binary>> -> {new_acc, rest}
    end
  end

  # Array parsing
  defp parse_array(<<?], rest::binary>>, acc), do: {Enum.reverse(acc), rest}

  defp parse_array(bin, acc) do
    {val, after_val} = parse_value(String.trim_leading(bin))
    new_acc = [val | acc]

    case String.trim_leading(after_val) do
      <<?,, rest::binary>> -> parse_array(String.trim_leading(rest), new_acc)
      <<?], rest::binary>> -> {Enum.reverse(new_acc), rest}
    end
  end

  # String body (after opening quote)
  defp parse_string_body(<<?", rest::binary>>, acc) do
    {IO.iodata_to_binary(Enum.reverse(acc)), rest}
  end

  defp parse_string_body(<<?\\, ?", rest::binary>>, acc),
    do: parse_string_body(rest, [?" | acc])

  defp parse_string_body(<<?\\, ?\\, rest::binary>>, acc),
    do: parse_string_body(rest, [?\\ | acc])

  defp parse_string_body(<<?\\, ?n, rest::binary>>, acc),
    do: parse_string_body(rest, [?\n | acc])

  defp parse_string_body(<<?\\, ?r, rest::binary>>, acc),
    do: parse_string_body(rest, [?\r | acc])

  defp parse_string_body(<<?\\, ?t, rest::binary>>, acc),
    do: parse_string_body(rest, [?\t | acc])

  defp parse_string_body(<<?\\, ?u, a, b, c, d, rest::binary>>, acc) do
    cp = String.to_integer(<<a, b, c, d>>, 16)
    parse_string_body(rest, [:unicode.characters_to_binary([cp]) | acc])
  end

  defp parse_string_body(<<ch::utf8, rest::binary>>, acc),
    do: parse_string_body(rest, [<<ch::utf8>> | acc])

  # Number parsing — integers and floats
  defp parse_number(bin) do
    {num_str, rest} = split_number(bin)

    if String.contains?(num_str, [".","e","E"]) do
      {Float.parse(num_str) |> elem(0), rest}
    else
      {String.to_integer(num_str), rest}
    end
  end

  defp split_number(bin), do: do_split(bin, [])

  defp do_split(<<ch, rest::binary>>, acc)
       when ch in ?0..?9 or ch == ?- or ch == ?. or ch == ?e or ch == ?E or ch == ?+ do
    do_split(rest, [ch | acc])
  end

  defp do_split(rest, acc), do: {IO.iodata_to_binary(Enum.reverse(acc)), rest}
end
