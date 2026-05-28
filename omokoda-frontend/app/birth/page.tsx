'use client';

import { useState } from 'react';
import { wasmBridge } from '@/lib/wasm';
import { AsciiPet } from '@/components/pet/AsciiPet';

type Step = 'name' | 'provider' | 'born';

export default function BirthPage() {
  const [step, setStep] = useState<Step>('name');
  const [name, setName] = useState('');
  const [provider, setProvider] = useState<'webllm' | 'ollama'>('webllm');
  const [agentId, setAgentId] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleBirth = async () => {
    if (!name.trim()) return;
    setLoading(true);
    setError('');
    try {
      await wasmBridge.configure_provider({ mode: provider });
      const result = await wasmBridge.create_agent(name.trim(), '');
      setAgentId(result.agent_id);
      setStep('born');
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="min-h-screen bg-black text-white flex flex-col items-center justify-center p-8 font-mono">
      <div className="max-w-md w-full space-y-8">
        <div className="text-center space-y-2">
          <h1 className="text-2xl font-bold text-purple-400">birth</h1>
          <p className="text-gray-500 text-sm">Who are you?</p>
        </div>

        {step === 'name' && (
          <div className="space-y-4">
            <input
              className="w-full bg-gray-950 border border-gray-800 rounded px-4 py-3 text-white placeholder-gray-700 outline-none focus:border-purple-500 transition-colors"
              placeholder="agent name"
              value={name}
              onChange={e => setName(e.target.value)}
              onKeyDown={e => e.key === 'Enter' && name.trim() && setStep('provider')}
              autoFocus
              maxLength={64}
            />
            <button
              onClick={() => name.trim() && setStep('provider')}
              disabled={!name.trim()}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 disabled:opacity-30 rounded transition-colors"
            >
              continue →
            </button>
          </div>
        )}

        {step === 'provider' && (
          <div className="space-y-4">
            <div className="text-sm text-gray-400">Choose your cognitive substrate:</div>
            <div className="space-y-2">
              {([
                { id: 'webllm', label: 'WebLLM', desc: 'sovereign · browser-local · offline · free', recommended: true },
                { id: 'ollama', label: 'Ollama', desc: 'sovereign · your machine · faster · larger models', recommended: false },
              ] as const).map(p => (
                <button
                  key={p.id}
                  onClick={() => setProvider(p.id)}
                  className={`w-full text-left p-3 rounded border transition-colors ${provider === p.id ? 'border-purple-500 bg-purple-950' : 'border-gray-800 hover:border-gray-600'}`}
                >
                  <div className="flex items-center gap-2">
                    <span className="font-bold">{p.label}</span>
                    {p.recommended && <span className="text-xs text-purple-400">(recommended)</span>}
                  </div>
                  <div className="text-xs text-gray-500">{p.desc}</div>
                </button>
              ))}
            </div>
            <button
              onClick={handleBirth}
              disabled={loading}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 disabled:opacity-30 rounded transition-colors"
            >
              {loading ? 'forging soul...' : `birth "${name}"`}
            </button>
            {error && <div className="text-red-400 text-sm">{error}</div>}
          </div>
        )}

        {step === 'born' && (
          <div className="space-y-6 text-center">
            <AsciiPet tier={0} reputation={0} mood={0.8} name={agentId} />
            <div className="space-y-1">
              <div className="text-green-400 font-bold">Soul forged.</div>
              <div className="text-gray-500 text-sm">Your agent exists now.</div>
              <div className="text-xs text-gray-700 font-mono">{agentId}</div>
            </div>
            <a href="/" className="block text-purple-400 hover:text-purple-300 text-sm">
              → enter the nexus
            </a>
          </div>
        )}
      </div>
    </main>
  );
}
