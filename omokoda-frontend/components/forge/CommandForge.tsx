'use client';

import { useState } from 'react';
import { wasmBridge, TranslatedStatement, ExecutionResult } from '@/lib/wasm';

interface CommandForgeProps {
  agentId: string;
  tier: number;
  onResult?: (result: ExecutionResult) => void;
}

export function CommandForge({ agentId, tier, onResult }: CommandForgeProps) {
  const [input, setInput] = useState('');
  const [preview, setPreview] = useState<TranslatedStatement | null>(null);
  const [sandbox, setSandbox] = useState(false);
  const [loading, setLoading] = useState(false);
  const [history, setHistory] = useState<string[]>([]);

  const handleInputChange = async (value: string) => {
    setInput(value);
    if (value.trim().length > 2) {
      const translated = await wasmBridge.translate(value).catch(() => null);
      setPreview(translated);
    } else {
      setPreview(null);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim()) return;

    setLoading(true);
    try {
      const translated = await wasmBridge.translate(input);
      const result = await wasmBridge.execute({ ...translated, params: sandbox ? 'sandbox:true' : undefined });
      setHistory(h => [...h.slice(-19), `${input} → ${result.receipt_id ?? 'ok'}`]);
      setInput('');
      setPreview(null);
      onResult?.(result);
    } catch (err) {
      setHistory(h => [...h.slice(-19), `ERROR: ${err}`]);
    } finally {
      setLoading(false);
    }
  };

  const primitiveColor = (p?: string) => {
    if (p === 'birth') return 'text-yellow-400';
    if (p === 'think') return 'text-blue-400';
    if (p === 'act') return 'text-green-400';
    return 'text-gray-400';
  };

  return (
    <div className="bg-gray-950 border border-gray-800 rounded-lg p-4 font-mono space-y-3">
      <div className="flex items-center gap-2 text-xs text-gray-500">
        <span className="text-purple-400">omo-koda</span>
        <span>›</span>
        <span>{agentId || 'no agent'}</span>
        <span className="ml-auto">tier {tier}</span>
        {sandbox && <span className="text-orange-400 border border-orange-400 px-1 rounded text-xs">sandbox</span>}
      </div>

      {/* Translation preview */}
      {preview && (
        <div className="text-xs border border-gray-800 rounded p-2 space-y-1">
          <div className="text-gray-500">Understood →</div>
          <div className={`font-bold ${primitiveColor(preview.primitive)}`}>
            {preview.primitive}
            {preview.name && ` "${preview.name}"`}
            {preview.prompt && ` "${preview.prompt.slice(0, 40)}${preview.prompt.length > 40 ? '…' : ''}"`}
            {preview.tool && ` "${preview.tool}"`}
          </div>
        </div>
      )}

      {/* Input */}
      <form onSubmit={handleSubmit} className="flex gap-2">
        <input
          className="flex-1 bg-transparent text-green-400 outline-none text-sm placeholder-gray-700"
          placeholder='birth "name"  |  think "intent"  |  act "tool"'
          value={input}
          onChange={e => handleInputChange(e.target.value)}
          disabled={loading}
          autoFocus
        />
        <button
          type="submit"
          disabled={loading || !input.trim()}
          className="text-xs text-gray-500 hover:text-green-400 disabled:opacity-30 transition-colors"
        >
          {loading ? '...' : '↵'}
        </button>
      </form>

      {/* Sandbox toggle */}
      <div className="flex items-center gap-2 text-xs">
        <button
          onClick={() => setSandbox(s => !s)}
          className={`border rounded px-2 py-0.5 transition-colors ${sandbox ? 'border-orange-400 text-orange-400' : 'border-gray-700 text-gray-600'}`}
        >
          {sandbox ? '⚠ sandbox on' : 'sandbox off'}
        </button>
        <span className="text-gray-700">sandbox limits blast radius</span>
      </div>

      {/* History */}
      {history.length > 0 && (
        <div className="space-y-0.5 max-h-24 overflow-y-auto text-xs text-gray-600">
          {history.slice(-5).map((h, i) => <div key={i}>{h}</div>)}
        </div>
      )}
    </div>
  );
}
