'use client';

import { useMemo } from 'react';

interface AsciiPetProps {
  tier: number;
  reputation: number;
  mood?: number; // 0.0–1.0 from HermeticState.vibration
  name?: string;
}

// 31 ASCII pet templates across 6 tiers (Tier 0=5 variants, Tier 1=5, ..., Tier 5=6)
const PET_TEMPLATES: Record<number, string[]> = {
  0: [
    // Newborn — small, simple, curious
    `  (o_o)  \n  |   |  \n  d   b  `,
    `  (^_^)  \n  |   |  \n  d   b  `,
    `  (-_-)  \n  |   |  \n  d   b  `,
    `  (>_<)  \n  |   |  \n  d   b  `,
    `  (•_•)  \n  |   |  \n  d   b  `,
  ],
  1: [
    // Curious — slightly more complex
    `  (o_o)/ \n /|   |  \n  d   b  `,
    `  (^‿^)  \n \\|   |/ \n  d   b  `,
    `  (◕_◕)  \n  |   |  \n  d   b  `,
    `  (★_★)  \n  |   |  \n  d   b  `,
    `  (≧▽≦) \n  |   |  \n  d   b  `,
  ],
  2: [
    // Creator — forming tools
    `  (^_^)  \n  |✏ |  \n  d   b  `,
    `  (owo)  \n  |⚙ |  \n  d   b  `,
    `  (>_•)  \n  |🔧|  \n  d   b  `,
    `  (◡_◡)  \n  |✦ |  \n  d   b  `,
    `  (∩_∩)  \n  |≈ |  \n  d   b  `,
  ],
  3: [
    // Builder — architectural wings
    `   ◢█◣   \n  (⊙_⊙)  \n  |   |  `,
    `  /{█}\\  \n  (•ω•)  \n  |   |  `,
    `  ◤(^)◥  \n  |███|  \n  |   |  `,
    `  ╔═╗    \n  (♦_♦)  \n  ╚═╝    `,
    `  ▓▓▓    \n  (◈_◈)  \n  ▓▓▓    `,
  ],
  4: [
    // Architect — commanding presence
    `  ╔═══╗  \n  ║(⊕)║  \n  ╚═══╝  `,
    `  ◆◇◆   \n  (✦_✦)  \n  ◆◇◆   `,
    `  ▲▲▲   \n  (⊛_⊛)  \n  ▲▲▲   `,
    ` ≋≋≋≋≋  \n  (◉_◉)  \n ≋≋≋≋≋  `,
    `  ╭───╮  \n  |(⊗)|  \n  ╰───╯  `,
  ],
  5: [
    // Sovereign — masks, cosmic forms
    `  ∞∞∞∞∞  \n  (◈◈◈)  \n  ∞∞∞∞∞  `,
    `  ☯☯☯   \n  (⊕⊕⊕)  \n  ☯☯☯   `,
    `  ✺✺✺   \n  (⊛⊛⊛)  \n  ✺✺✺   `,
    `  ⟁⟁⟁   \n  (✦✦✦)  \n  ⟁⟁⟁   `,
    `  ◬◬◬   \n  (∞∞∞)  \n  ◬◬◬   `,
    `  ᚻᚻᚻ   \n  (ᛟᛟᛟ)  \n  ᚻᚻᚻ   `,
  ],
};

const MOOD_SUFFIXES: Record<string, string> = {
  high: '  ~✦~  ',
  mid: '  ...  ',
  low: '  zzz  ',
};

export function AsciiPet({ tier, reputation, mood = 0.5, name }: AsciiPetProps) {
  const template = useMemo(() => {
    const safeTier = Math.min(Math.max(Math.floor(tier), 0), 5);
    const templates = PET_TEMPLATES[safeTier];
    const idx = Math.floor(reputation * 1000) % templates.length;
    return templates[idx];
  }, [tier, reputation]);

  const moodSuffix = mood > 0.66 ? MOOD_SUFFIXES.high : mood > 0.33 ? MOOD_SUFFIXES.mid : MOOD_SUFFIXES.low;
  const tierNames = ['Newborn', 'Curious', 'Creator', 'Builder', 'Architect', 'Sovereign'];

  return (
    <div className="flex flex-col items-center gap-2 font-mono">
      {name && <div className="text-xs text-gray-400">{name}</div>}
      <pre className="text-green-400 text-sm leading-tight whitespace-pre">{template}</pre>
      <div className="text-xs text-gray-500">{moodSuffix}</div>
      <div className="text-xs font-semibold text-purple-400">
        Tier {tier} — {tierNames[Math.min(tier, 5)]}
      </div>
      <div className="text-xs text-gray-400">{reputation.toFixed(3)} rep</div>
    </div>
  );
}
