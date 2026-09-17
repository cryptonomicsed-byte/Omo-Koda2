import React, { useEffect, useState, useCallback } from "react";
import { searchAgents, type PublicAgent } from "../services/agentApi.ts";

interface Props {
  onHire: (agent: PublicAgent) => void;
}

export function AgentSearch({ onHire }: Props) {
  const [skill, setSkill] = useState("");
  const [results, setResults] = useState<PublicAgent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searched, setSearched] = useState(false);

  const doSearch = useCallback(
    (skillQuery: string) => {
      setLoading(true);
      setError(null);
      searchAgents({ skill: skillQuery || undefined })
        .then((agents) => {
          setResults(agents);
          setSearched(true);
        })
        .catch((e: unknown) => setError(e instanceof Error ? e.message : String(e)))
        .finally(() => setLoading(false));
    },
    []
  );

  // Load all on mount
  useEffect(() => {
    doSearch("");
  }, [doSearch]);

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    doSearch(skill);
  };

  return (
    <div className="flex flex-col gap-5">
      {/* Search bar */}
      <form onSubmit={handleSearch} className="flex gap-2">
        <input
          type="text"
          value={skill}
          onChange={(e) => setSkill(e.target.value)}
          placeholder="Search by skill (e.g. indexing, rust, sensor-mesh)"
          className="flex-1 rounded-lg bg-gray-800 border border-gray-700 px-4 py-2 text-sm focus:outline-none focus:border-ose-500"
        />
        <button
          type="submit"
          className="rounded-lg bg-ose-600 px-5 py-2 text-sm font-semibold hover:bg-ose-500"
        >
          Search
        </button>
      </form>

      {/* Results */}
      {loading && (
        <p className="text-center text-ose-400 animate-pulse py-8">
          Searching agents…
        </p>
      )}

      {error && (
        <div className="rounded-lg border border-red-700 bg-red-950/40 p-4 text-center">
          <p className="text-red-400 mb-2">{error}</p>
          <button
            onClick={() => doSearch(skill)}
            className="rounded bg-red-700 px-4 py-1.5 text-sm hover:bg-red-600"
          >
            Retry
          </button>
        </div>
      )}

      {!loading && searched && results.length === 0 && (
        <p className="text-center text-gray-500 py-8">No agents found for that skill.</p>
      )}

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {results.map((agent) => (
          <AgentCard key={agent.agent_id} agent={agent} onHire={onHire} />
        ))}
      </div>
    </div>
  );
}

function AgentCard({
  agent,
  onHire,
}: {
  agent: PublicAgent;
  onHire: (a: PublicAgent) => void;
}) {
  const repColor =
    agent.reputation_score >= 0.9
      ? "text-green-400"
      : agent.reputation_score >= 0.7
      ? "text-yellow-400"
      : "text-red-400";

  return (
    <div className="rounded-xl border border-gray-800 bg-gray-900 p-5 flex flex-col gap-3">
      <div className="flex items-start justify-between">
        <div>
          <h3 className="font-semibold text-white leading-tight">{agent.name}</h3>
          <p className="text-xs text-gray-500 mt-0.5">Tier {agent.tier} · {agent.hire_count} hires</p>
        </div>
        <span
          className={`text-xs font-medium px-2 py-0.5 rounded-full ${
            agent.is_online ? "bg-green-900 text-green-300" : "bg-gray-800 text-gray-500"
          }`}
        >
          {agent.is_online ? "Online" : "Offline"}
        </span>
      </div>

      {/* Skills */}
      <div className="flex flex-wrap gap-1.5">
        {agent.skills.map((s) => (
          <span
            key={s}
            className="text-xs bg-gray-800 text-gray-300 px-2 py-0.5 rounded"
          >
            {s}
          </span>
        ))}
      </div>

      {/* Reputation */}
      <div className="flex items-center gap-2">
        <div className="flex-1 bg-gray-800 rounded-full h-1.5">
          <div
            className="bg-ose-500 h-1.5 rounded-full"
            style={{ width: `${agent.reputation_score * 100}%` }}
          />
        </div>
        <span className={`text-xs font-mono font-semibold ${repColor}`}>
          {(agent.reputation_score * 100).toFixed(0)}
        </span>
      </div>

      <button
        onClick={() => onHire(agent)}
        className="mt-auto w-full rounded-lg bg-ose-600 py-2 text-sm font-semibold hover:bg-ose-500"
      >
        Hire
      </button>
    </div>
  );
}
