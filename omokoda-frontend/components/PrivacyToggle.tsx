"use client";

import React, { useState } from "react";

type PrivacyMode = "public" | "private" | "incognito";

interface ConsentDialogProps {
  mode: PrivacyMode;
  onConfirm: () => void;
  onCancel: () => void;
}

function ConsentDialog({ mode, onConfirm, onCancel }: ConsentDialogProps) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl p-6 max-w-sm w-full shadow-2xl">
        <h2 className="text-lg font-bold mb-2">Switch to {mode} mode?</h2>
        <p className="text-sm text-gray-600 mb-4">
          {mode === "private"
            ? "Private mode routes all inference locally (WebLLM/Ollama). No data leaves your device."
            : "Incognito mode is like private, but also disables session logging entirely."}
        </p>
        <div className="flex gap-2 justify-end">
          <button onClick={onCancel} className="px-4 py-2 rounded-lg border text-sm">
            Cancel
          </button>
          <button onClick={onConfirm} className="px-4 py-2 rounded-lg bg-indigo-600 text-white text-sm">
            Confirm
          </button>
        </div>
      </div>
    </div>
  );
}

interface PrivacyToggleProps {
  onChange?: (mode: PrivacyMode) => void;
  onForget?: () => void;
}

export default function PrivacyToggle({ onChange, onForget }: PrivacyToggleProps) {
  const [mode, setMode] = useState<PrivacyMode>("public");
  const [pendingMode, setPendingMode] = useState<PrivacyMode | null>(null);
  const [showForgetConfirm, setShowForgetConfirm] = useState(false);

  const handleModeClick = (next: PrivacyMode) => {
    if (next === mode) return;
    if (next === "private" || next === "incognito") {
      setPendingMode(next);
    } else {
      applyMode(next);
    }
  };

  const applyMode = (next: PrivacyMode) => {
    setMode(next);
    setPendingMode(null);
    onChange?.(next);
  };

  const modeBadgeColor: Record<PrivacyMode, string> = {
    public: "bg-green-100 text-green-800",
    private: "bg-amber-100 text-amber-800",
    incognito: "bg-gray-900 text-gray-100",
  };

  return (
    <div className="flex flex-col gap-3 p-4 rounded-xl border bg-white shadow-sm max-w-xs">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-gray-700">Privacy Mode</span>
        <span className={`text-xs px-2 py-0.5 rounded-full font-semibold ${modeBadgeColor[mode]}`}>
          {mode}
        </span>
      </div>

      <div className="flex gap-2">
        {(["public", "private", "incognito"] as PrivacyMode[]).map((m) => (
          <button
            key={m}
            onClick={() => handleModeClick(m)}
            className={`flex-1 py-1.5 rounded-lg text-xs font-medium border transition-colors ${
              mode === m
                ? "bg-indigo-600 text-white border-indigo-600"
                : "text-gray-600 border-gray-200 hover:border-indigo-400"
            }`}
          >
            {m}
          </button>
        ))}
      </div>

      <button
        onClick={() => setShowForgetConfirm(true)}
        className="text-xs text-red-500 hover:text-red-700 underline text-left"
      >
        Forget My Data
      </button>

      {pendingMode && (
        <ConsentDialog
          mode={pendingMode}
          onConfirm={() => applyMode(pendingMode)}
          onCancel={() => setPendingMode(null)}
        />
      )}

      {showForgetConfirm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 max-w-sm w-full shadow-2xl">
            <h2 className="text-lg font-bold mb-2">Forget all data?</h2>
            <p className="text-sm text-gray-600 mb-4">
              This permanently deletes your session, memory, and identity from this device.
            </p>
            <div className="flex gap-2 justify-end">
              <button onClick={() => setShowForgetConfirm(false)} className="px-4 py-2 rounded-lg border text-sm">
                Cancel
              </button>
              <button
                onClick={() => { setShowForgetConfirm(false); onForget?.(); }}
                className="px-4 py-2 rounded-lg bg-red-600 text-white text-sm"
              >
                Delete Everything
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
