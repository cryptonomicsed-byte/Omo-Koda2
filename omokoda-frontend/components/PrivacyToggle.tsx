'use client';

/**
 * PrivacyToggle — Controls the agent's privacy mode.
 *
 * Modes:
 * - public: any LLM provider (default)
 * - private: local model only (WebLLM / Ollama); requires consent
 * - incognito: local model + no logging; requires consent
 *
 * "Forget My Data" permanently wipes the agent's private memory.
 *
 * /private enforcement: switching to private or incognito shows a
 * ConsentDialog. The Rust backend will hard-fail any think/act call
 * if a non-local provider is attempted while in private/incognito mode.
 */

import { useState } from 'react';
import { rpcClient, PrivacyMode } from '../lib/rpc_client';

type DialogKind = 'consent' | 'forget' | null;

const MODE_LABELS: Record<PrivacyMode, string> = {
  public: 'Public',
  private: 'Private',
  incognito: 'Incognito',
};

const MODE_DESCRIPTIONS: Record<PrivacyMode, string> = {
  public: 'All providers allowed. Conversations may be logged.',
  private: 'Local model only (WebLLM / Ollama). No external LLM calls.',
  incognito: 'Local model only. No logging of prompts or outputs.',
};

const BADGE_STYLES: Record<PrivacyMode, string> = {
  public: 'bg-green-100 text-green-800 border-green-300',
  private: 'bg-amber-100 text-amber-800 border-amber-300',
  incognito: 'bg-gray-900 text-gray-100 border-gray-700',
};

export default function PrivacyToggle() {
  const [mode, setMode] = useState<PrivacyMode>(rpcClient.getPrivacyMode());
  const [pendingMode, setPendingMode] = useState<PrivacyMode | null>(null);
  const [dialog, setDialog] = useState<DialogKind>(null);
  const [error, setError] = useState<string | null>(null);

  const handleModeClick = (next: PrivacyMode) => {
    if (next === mode) return;
    if (next === 'public') {
      applyMode('public');
      return;
    }
    // private / incognito require explicit consent
    setPendingMode(next);
    setDialog('consent');
  };

  const applyMode = async (next: PrivacyMode) => {
    setError(null);
    try {
      await rpcClient.setPrivacyMode(next);
      setMode(next);
    } catch (e) {
      setError((e as Error).message);
    }
    setDialog(null);
    setPendingMode(null);
  };

  const handleForgetClick = () => setDialog('forget');

  const handleForget = async () => {
    setError(null);
    try {
      await rpcClient.forget();
    } catch (e) {
      setError((e as Error).message);
    }
    setDialog(null);
  };

  return (
    <div className="flex flex-col gap-3 p-4 rounded-lg border border-gray-200 bg-white shadow-sm max-w-sm">
      {/* Current mode badge */}
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-gray-700">Privacy Mode</span>
        <span
          className={`text-xs font-semibold px-2 py-1 rounded-full border ${BADGE_STYLES[mode]}`}
        >
          {MODE_LABELS[mode]}
        </span>
      </div>

      {/* Mode selector buttons */}
      <div className="flex gap-2">
        {(Object.keys(MODE_LABELS) as PrivacyMode[]).map((m) => (
          <button
            key={m}
            onClick={() => handleModeClick(m)}
            aria-pressed={mode === m}
            className={`flex-1 py-1.5 text-xs font-medium rounded border transition-colors
              ${mode === m
                ? 'bg-indigo-600 text-white border-indigo-700'
                : 'bg-gray-50 text-gray-600 border-gray-200 hover:bg-gray-100'
              }`}
          >
            {MODE_LABELS[m]}
          </button>
        ))}
      </div>

      <p className="text-xs text-gray-500">{MODE_DESCRIPTIONS[mode]}</p>

      {/* Forget button */}
      <button
        onClick={handleForgetClick}
        className="text-xs text-red-600 hover:text-red-800 underline text-left"
      >
        Forget My Data
      </button>

      {/* Error display */}
      {error && (
        <p className="text-xs text-red-700 bg-red-50 border border-red-200 rounded p-2">
          {error}
        </p>
      )}

      {/* Consent dialog for private / incognito */}
      {dialog === 'consent' && pendingMode && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl p-6 max-w-xs w-full mx-4">
            <h3 className="font-semibold text-gray-900 mb-2">
              Switch to {MODE_LABELS[pendingMode]}?
            </h3>
            <p className="text-sm text-gray-600 mb-4">
              {MODE_DESCRIPTIONS[pendingMode]}
              {' '}The Rust backend will hard-fail any request that attempts
              to use an external LLM provider in this mode.
            </p>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => { setDialog(null); setPendingMode(null); }}
                className="px-3 py-1.5 text-sm text-gray-600 border border-gray-200 rounded hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={() => applyMode(pendingMode)}
                className="px-3 py-1.5 text-sm text-white bg-indigo-600 rounded hover:bg-indigo-700"
              >
                Confirm
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Forget confirmation dialog */}
      {dialog === 'forget' && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl shadow-xl p-6 max-w-xs w-full mx-4">
            <h3 className="font-semibold text-gray-900 mb-2">Forget All Private Data?</h3>
            <p className="text-sm text-gray-600 mb-4">
              This permanently wipes your agent&apos;s private memory. Public reputation and
              on-chain soul are preserved. This action cannot be undone.
            </p>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => setDialog(null)}
                className="px-3 py-1.5 text-sm text-gray-600 border border-gray-200 rounded hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                onClick={handleForget}
                className="px-3 py-1.5 text-sm text-white bg-red-600 rounded hover:bg-red-700"
              >
                Forget
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
