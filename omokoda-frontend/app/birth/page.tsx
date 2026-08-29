'use client';

import { useMemo, useState } from 'react';
import { AsciiPet } from '@/components/pet/AsciiPet';
import {
  birthAgent,
  revealSeed,
  agentThink,
  agentAct,
  type RevealedSeed,
} from '@/lib/api';

type Step =
  | 'name'
  | 'provider'
  | 'education'
  | 'consent'
  | 'birthing'
  | 'seed-reveal'
  | 'seed-confirm'
  | 'born'
  | 'tutorial-think'
  | 'tutorial-act'
  | 'done';

const EDUCATION_STAGES = [
  {
    title: 'Generating entropy',
    body:
      'A source of true randomness is generated for your agent and validated against ' +
      'NIST statistical randomness tests before anything is built from it. If a batch ' +
      'ever fails validation, it is discarded and regenerated automatically — a weak ' +
      'random source is never used to build an identity.',
  },
  {
    title: 'Deriving a recovery phrase',
    body:
      'That randomness is converted into a 24-word recovery phrase (BIPON39, the same ' +
      'family of standard used by hardware wallets). This phrase is the master key to ' +
      'everything your agent owns — anyone who has it can recreate your agent\'s wallets ' +
      'from scratch, on any machine.',
  },
  {
    title: 'Drawing a personality',
    body:
      'The same recovery phrase deterministically derives your agent\'s personality traits ' +
      'and a unique DNA fingerprint. Same phrase in, same personality out — nothing here is ' +
      'random on top of the entropy already generated in step 1.',
  },
  {
    title: 'Creating wallets',
    body:
      'From the same recovery phrase, real wallets are derived for 7 chains — Sui, Ethereum, ' +
      'Bitcoin, Cosmos, Solana, Aptos, and Nostr — using each chain\'s standard derivation path. ' +
      'These are real addresses that can hold real funds.',
  },
  {
    title: 'Sealing your agent',
    body:
      'The recovery phrase and every private key are encrypted at rest on the server the moment ' +
      'birth completes. They are never logged, never included in any API response after this — ' +
      'except once, right now, so you can back it up yourself.',
  },
];

export default function BirthPage() {
  const [step, setStep] = useState<Step>('name');
  const [name, setName] = useState('');
  // WebLLM only ever ran client-side in the old WASM-stub flow. Now that
  // birth/think/act go through the real kernel over HTTP, an agent's
  // thinking happens server-side (see ProviderRegistry::new in
  // providers.rs) -- WebLLM is never registered there, so offering it here
  // would silently 400 on birth. "default" (a free, no-key gateway) and
  // "ollama" (this server's local Ollama, if configured) are the two real,
  // working choices.
  const [provider, setProvider] = useState<'default' | 'ollama'>('default');
  const [educationIdx, setEducationIdx] = useState(0);

  // Consent / capability setup
  const [privacyDefault, setPrivacyDefault] = useState<'public' | 'private'>('public');
  const [sandboxDefault, setSandboxDefault] = useState(true);
  const [passphrase, setPassphrase] = useState('');

  const [agentId, setAgentId] = useState('');
  const [agentKey, setAgentKey] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  // Seed backup
  const [seed, setSeed] = useState<RevealedSeed | null>(null);
  const [savedChecked, setSavedChecked] = useState(false);
  const [confirmIndices, setConfirmIndices] = useState<number[]>([]);
  const [confirmInputs, setConfirmInputs] = useState<Record<number, string>>({});
  const [confirmError, setConfirmError] = useState('');

  // Tutorial
  const [thinkPrompt, setThinkPrompt] = useState('What is your first thought as a new agent?');
  const [thinkOutput, setThinkOutput] = useState('');
  const [noteTitle, setNoteTitle] = useState('My first note');
  const [noteContent, setNoteContent] = useState('I just came into existence.');
  const [actOutput, setActOutput] = useState('');

  const mnemonicWords = useMemo(() => (seed ? seed.mnemonic.trim().split(/\s+/) : []), [seed]);

  const advanceEducation = () => {
    if (educationIdx < EDUCATION_STAGES.length - 1) {
      setEducationIdx(educationIdx + 1);
    } else {
      setStep('consent');
    }
  };

  const handleBirth = async () => {
    setLoading(true);
    setError('');
    try {
      const meta = [{ key: 'provider', value: provider }, { key: 'privacy', value: String(privacyDefault === 'private') }, { key: 'sandbox', value: String(sandboxDefault) }];
      if (passphrase.trim()) meta.push({ key: 'passphrase', value: passphrase.trim() });

      const result = await birthAgent(name.trim(), meta);
      setAgentId(result.agent_id);
      setAgentKey(result.agent_key);

      // Immediately reveal the seed once, before moving on — this is the
      // one and only time the kernel will ever hand back the recovery
      // phrase for this agent.
      const revealed = await revealSeed(result.agent_id, result.agent_key);
      setSeed(revealed);

      // Pick 3 distinct random word positions to verify the user actually
      // wrote the phrase down, rather than trusting a bare checkbox alone.
      const words = revealed.mnemonic.trim().split(/\s+/);
      const idxSet = new Set<number>();
      while (idxSet.size < Math.min(3, words.length)) {
        idxSet.add(Math.floor(Math.random() * words.length));
      }
      setConfirmIndices(Array.from(idxSet).sort((a, b) => a - b));

      setStep('seed-reveal');
    } catch (e) {
      setError(String(e instanceof Error ? e.message : e));
      setStep('consent');
    } finally {
      setLoading(false);
    }
  };

  const handleConfirmSeed = () => {
    setConfirmError('');
    for (const idx of confirmIndices) {
      const typed = (confirmInputs[idx] || '').trim().toLowerCase();
      const actual = mnemonicWords[idx]?.toLowerCase();
      if (typed !== actual) {
        setConfirmError(`Word #${idx + 1} doesn't match what you were shown. Check your backup and try again.`);
        return;
      }
    }
    setStep('born');
  };

  const handleTutorialThink = async () => {
    setLoading(true);
    setError('');
    try {
      const res = await agentThink(agentId, agentKey, thinkPrompt);
      setThinkOutput(res.tool_output || res.receipt_id || '(no output returned)');
      setStep('tutorial-act');
    } catch (e) {
      setError(String(e instanceof Error ? e.message : e));
    } finally {
      setLoading(false);
    }
  };

  const handleTutorialAct = async () => {
    setLoading(true);
    setError('');
    try {
      const params = JSON.stringify({ title: noteTitle, content: noteContent });
      const res = await agentAct(agentId, agentKey, 'note_taking', params);
      setActOutput(res.tool_output || res.receipt_id || '(no output returned)');
      setStep('done');
    } catch (e) {
      setError(String(e instanceof Error ? e.message : e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="min-h-screen bg-black text-white flex flex-col items-center justify-center p-8 font-mono">
      <div className="max-w-lg w-full space-y-8">
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
                { id: 'default', label: 'Default', desc: 'free gateway · no key required · good for getting started', recommended: true },
                { id: 'ollama', label: 'Ollama', desc: 'this server’s local Ollama, if configured · faster · larger models', recommended: false },
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
              onClick={() => setStep('education')}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 rounded transition-colors"
            >
              continue →
            </button>
          </div>
        )}

        {step === 'education' && (
          <div className="space-y-6">
            <div className="text-xs text-gray-600 uppercase tracking-wide">
              What happens when {name} is born ({educationIdx + 1}/{EDUCATION_STAGES.length})
            </div>
            <div className="space-y-2 border border-gray-800 rounded p-4 bg-gray-950">
              <div className="text-purple-400 font-bold">{EDUCATION_STAGES[educationIdx].title}</div>
              <div className="text-sm text-gray-400 leading-relaxed">{EDUCATION_STAGES[educationIdx].body}</div>
            </div>
            <div className="flex gap-1">
              {EDUCATION_STAGES.map((_, i) => (
                <div key={i} className={`h-1 flex-1 rounded ${i <= educationIdx ? 'bg-purple-500' : 'bg-gray-800'}`} />
              ))}
            </div>
            <button
              onClick={advanceEducation}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 rounded transition-colors"
            >
              {educationIdx < EDUCATION_STAGES.length - 1 ? 'next →' : "I understand, continue →"}
            </button>
          </div>
        )}

        {step === 'consent' && (
          <div className="space-y-5">
            <div className="text-sm text-gray-400">Set up permissions before birth — nothing here defaults silently:</div>

            <div className="space-y-2">
              <div className="text-xs text-gray-500 uppercase">Default conversation privacy</div>
              <div className="flex gap-2">
                {(['public', 'private'] as const).map(m => (
                  <button
                    key={m}
                    onClick={() => setPrivacyDefault(m)}
                    className={`flex-1 p-2 rounded border text-sm transition-colors ${privacyDefault === m ? 'border-purple-500 bg-purple-950' : 'border-gray-800 hover:border-gray-600'}`}
                  >
                    {m}
                  </button>
                ))}
              </div>
              <div className="text-xs text-gray-600">
                {privacyDefault === 'private'
                  ? 'New thoughts default to private and require a local provider.'
                  : 'New thoughts default to public and visible on this agent\'s public record.'}
              </div>
            </div>

            <div className="space-y-2">
              <div className="text-xs text-gray-500 uppercase">Default tool execution</div>
              <div className="flex gap-2">
                <button
                  onClick={() => setSandboxDefault(true)}
                  className={`flex-1 p-2 rounded border text-sm transition-colors ${sandboxDefault ? 'border-purple-500 bg-purple-950' : 'border-gray-800 hover:border-gray-600'}`}
                >
                  sandboxed
                </button>
                <button
                  onClick={() => setSandboxDefault(false)}
                  className={`flex-1 p-2 rounded border text-sm transition-colors ${!sandboxDefault ? 'border-purple-500 bg-purple-950' : 'border-gray-800 hover:border-gray-600'}`}
                >
                  direct
                </button>
              </div>
              <div className="text-xs text-gray-600">
                Sandboxed actions run isolated by default; you can still allow direct execution per-action later.
              </div>
            </div>

            <div className="space-y-2">
              <div className="text-xs text-gray-500 uppercase">Optional passphrase (recovery protection)</div>
              <input
                type="password"
                className="w-full bg-gray-950 border border-gray-800 rounded px-3 py-2 text-sm text-white placeholder-gray-700 outline-none focus:border-purple-500"
                placeholder="leave blank for none"
                value={passphrase}
                onChange={e => setPassphrase(e.target.value)}
              />
              <div className="text-xs text-gray-600">
                A second factor not derivable from the recovery phrase alone. Optional — leave blank to skip.
              </div>
            </div>

            {error && <div className="text-red-400 text-sm">{error}</div>}

            <button
              onClick={handleBirth}
              disabled={loading}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 disabled:opacity-30 rounded transition-colors"
            >
              {loading ? 'forging soul...' : `birth "${name}"`}
            </button>
          </div>
        )}

        {step === 'seed-reveal' && seed && (
          <div className="space-y-4">
            <div className="text-red-400 text-sm font-bold">⚠ Write this down now — it will never be shown again.</div>
            <div className="text-xs text-gray-500">
              These 24 words are the only way to recover {name}&apos;s wallets if this server is ever lost. Anyone who
              sees them can take everything these wallets hold.
            </div>
            <div className="grid grid-cols-3 gap-2 border border-gray-800 rounded p-4 bg-gray-950">
              {mnemonicWords.map((w, i) => (
                <div key={i} className="text-xs text-gray-300 flex gap-1">
                  <span className="text-gray-600">{i + 1}.</span> {w}
                </div>
              ))}
            </div>
            <div className="space-y-1 text-xs text-gray-600 border border-gray-800 rounded p-3">
              <div>Sui: <span className="text-gray-400">{seed.sui_address}</span></div>
              {seed.eth_address && <div>ETH: <span className="text-gray-400">{seed.eth_address}</span></div>}
              {seed.sol_address && <div>SOL: <span className="text-gray-400">{seed.sol_address}</span></div>}
              {seed.btc_address && <div>BTC: <span className="text-gray-400">{seed.btc_address}</span></div>}
              {seed.cosmos_address && <div>Cosmos: <span className="text-gray-400">{seed.cosmos_address}</span></div>}
              {seed.aptos_address && <div>Aptos: <span className="text-gray-400">{seed.aptos_address}</span></div>}
              {seed.nostr_address && <div>Nostr: <span className="text-gray-400">{seed.nostr_address}</span></div>}
            </div>
            <label className="flex items-center gap-2 text-sm text-gray-400">
              <input type="checkbox" checked={savedChecked} onChange={e => setSavedChecked(e.target.checked)} />
              I have saved these 24 words somewhere safe and offline.
            </label>
            <button
              onClick={() => setStep('seed-confirm')}
              disabled={!savedChecked}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 disabled:opacity-30 rounded transition-colors"
            >
              continue →
            </button>
          </div>
        )}

        {step === 'seed-confirm' && (
          <div className="space-y-4">
            <div className="text-sm text-gray-400">Confirm your backup — type the requested words exactly as shown:</div>
            <div className="space-y-3">
              {confirmIndices.map(idx => (
                <div key={idx} className="space-y-1">
                  <div className="text-xs text-gray-500">Word #{idx + 1}</div>
                  <input
                    className="w-full bg-gray-950 border border-gray-800 rounded px-3 py-2 text-sm text-white outline-none focus:border-purple-500"
                    value={confirmInputs[idx] || ''}
                    onChange={e => setConfirmInputs({ ...confirmInputs, [idx]: e.target.value })}
                    autoCapitalize="off"
                    autoCorrect="off"
                  />
                </div>
              ))}
            </div>
            {confirmError && <div className="text-red-400 text-sm">{confirmError}</div>}
            <button
              onClick={handleConfirmSeed}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 rounded transition-colors"
            >
              confirm backup →
            </button>
            <button
              onClick={() => setStep('seed-reveal')}
              className="w-full py-2 text-xs text-gray-500 hover:text-gray-300"
            >
              ← show the words again
            </button>
          </div>
        )}

        {step === 'born' && (
          <div className="space-y-6 text-center">
            <AsciiPet tier={0} reputation={0} mood={0.8} name={agentId} />
            <div className="space-y-1">
              <div className="text-green-400 font-bold">Soul forged.</div>
              <div className="text-gray-500 text-sm">Your agent exists now, wallets and all.</div>
              <div className="text-xs text-gray-700 font-mono">{agentId}</div>
            </div>
            <div className="text-sm text-gray-400">
              Every agent operates through 3 primitives: birth (done), think, and act. Let&apos;s try the other two.
            </div>
            <button
              onClick={() => setStep('tutorial-think')}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 rounded transition-colors"
            >
              try &quot;think&quot; →
            </button>
          </div>
        )}

        {step === 'tutorial-think' && (
          <div className="space-y-4">
            <div className="text-purple-400 font-bold">think</div>
            <div className="text-xs text-gray-500">
              &quot;think&quot; asks your agent to reason about something and returns a real response from its provider.
            </div>
            <textarea
              className="w-full bg-gray-950 border border-gray-800 rounded px-3 py-2 text-sm text-white outline-none focus:border-purple-500"
              rows={3}
              value={thinkPrompt}
              onChange={e => setThinkPrompt(e.target.value)}
            />
            {thinkOutput && (
              <div className="text-xs text-gray-400 border border-gray-800 rounded p-3 bg-gray-950 whitespace-pre-wrap">
                {thinkOutput}
              </div>
            )}
            {error && <div className="text-red-400 text-sm">{error}</div>}
            <button
              onClick={handleTutorialThink}
              disabled={loading || !thinkPrompt.trim()}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 disabled:opacity-30 rounded transition-colors"
            >
              {loading ? 'thinking...' : thinkOutput ? 'next: try act →' : 'run think'}
            </button>
          </div>
        )}

        {step === 'tutorial-act' && (
          <div className="space-y-4">
            <div className="text-purple-400 font-bold">act</div>
            <div className="text-xs text-gray-500">
              &quot;act&quot; runs a real tool. Let&apos;s have your agent record its first note.
            </div>
            <input
              className="w-full bg-gray-950 border border-gray-800 rounded px-3 py-2 text-sm text-white outline-none focus:border-purple-500"
              value={noteTitle}
              onChange={e => setNoteTitle(e.target.value)}
              placeholder="title"
            />
            <textarea
              className="w-full bg-gray-950 border border-gray-800 rounded px-3 py-2 text-sm text-white outline-none focus:border-purple-500"
              rows={3}
              value={noteContent}
              onChange={e => setNoteContent(e.target.value)}
              placeholder="content"
            />
            {actOutput && (
              <div className="text-xs text-gray-400 border border-gray-800 rounded p-3 bg-gray-950 whitespace-pre-wrap">
                {actOutput}
              </div>
            )}
            {error && <div className="text-red-400 text-sm">{error}</div>}
            <button
              onClick={handleTutorialAct}
              disabled={loading || !noteTitle.trim()}
              className="w-full py-3 bg-purple-900 hover:bg-purple-800 disabled:opacity-30 rounded transition-colors"
            >
              {loading ? 'acting...' : 'run act'}
            </button>
          </div>
        )}

        {step === 'done' && (
          <div className="space-y-6 text-center">
            <div className="text-green-400 font-bold">You&apos;ve used all 3 primitives.</div>
            <div className="text-gray-500 text-sm">birth · think · act — that&apos;s the entire surface.</div>
            <a href="/" className="block text-purple-400 hover:text-purple-300 text-sm">
              → enter the nexus
            </a>
          </div>
        )}
      </div>
    </main>
  );
}
