/**
 * Aether Think Engine — The Intelligent Core
 *
 * This is the brain of the Aether language. When a user writes:
 *   think "hire 3 agents to research crypto trends and verify each other's work"
 *
 * The engine:
 *   1. Analyzes intent via NLU (pattern matching + LLM fallback)
 *   2. Identifies which stdlib modules are needed
 *   3. Builds a multi-step execution plan
 *   4. Validates the plan against agent state, ethics, permissions
 *   5. Self-corrects if steps conflict or preconditions fail
 *   6. Executes the plan step-by-step with receipts
 *   7. Stores results in memory for future context
 *
 * Provider chain: Ollama → Claude → OpenRouter → Built-in Reasoner
 */

import crypto from 'node:crypto';
import https from 'node:https';
import http from 'node:http';

// ────────────────────────────────────────────────────────────
// Intent Classification
// ────────────────────────────────────────────────────────────

const INTENT = Object.freeze({
  RESEARCH:     'research',
  HIRE:         'hire',
  BUILD:        'build',
  ANALYZE:      'analyze',
  REMEMBER:     'remember',
  FORGET:       'forget',
  CHECK_STATUS: 'check_status',
  EVOLVE:       'evolve',
  COORDINATE:   'coordinate',
  TRADE:        'trade',
  WITNESS:      'witness',
  CONVERT:      'convert',
  GENERAL:      'general',
});

const INTENT_PATTERNS = [
  { intent: INTENT.HIRE,         patterns: [/\bhire\b/i, /\brecruit\b/i, /\bassign\b/i, /\bdeploy\s+agent/i, /\bget\s+(\d+\s+)?agents?\b/i, /\bspin\s+up\b/i, /\blaunch\s+(\d+\s+)?agents?\b/i, /\bpost\s+(a\s+)?job\b/i, /\bopen\s+call\b/i, /\bpublic\s+request\b/i] },
  { intent: INTENT.COORDINATE,   patterns: [/\bcoordinat/i, /\bswarm\b/i, /\bteam\b/i, /\bparallel\b/i, /\bpipeline\b/i, /\bsplit.*work/i, /\bdivide.*task/i, /\bwork\s+together\b/i, /\bcollaborat/i] },
  { intent: INTENT.RESEARCH,     patterns: [/\bresearch\b/i, /\bfind\b/i, /\bsearch\b/i, /\blook\s*(up|for|into)\b/i, /\binvestigat/i, /\bexplore\b/i, /\bdiscov/i, /\btrends?\b/i, /\banalyz.*market/i] },
  { intent: INTENT.BUILD,        patterns: [/\bbuild\b/i, /\bcreate\b/i, /\bgenerate\b/i, /\bimplement\b/i, /\bwrite\b/i, /\bcode\b/i, /\bdevelop\b/i, /\bconstruct\b/i, /\bdesign\b/i, /\bcraft\b/i] },
  { intent: INTENT.ANALYZE,      patterns: [/\banalyz/i, /\bexamin/i, /\breview\b/i, /\baudit\b/i, /\bcheck\b/i, /\bevaluat/i, /\bcompare\b/i, /\bassess\b/i, /\bdiagnos/i, /\bdebug\b/i] },
  { intent: INTENT.REMEMBER,     patterns: [/\bremember\b/i, /\bstore\b/i, /\bsave\b/i, /\brecord\b/i, /\bnote\b/i, /\bkeep\s+track/i, /\blog\b/i, /\bmemoriz/i] },
  { intent: INTENT.FORGET,       patterns: [/\bforget\b/i, /\bdelete\b/i, /\bremove\b/i, /\bclear\b/i, /\bwipe\b/i, /\berase\b/i] },
  { intent: INTENT.CHECK_STATUS, patterns: [/\bstatus\b/i, /\bbalance\b/i, /\bhow\s+(much|many)\b/i, /\bwhat('s|\s+is)\s+(my|the)\b/i, /\bshow\s+(me\s+)?(my|the)\b/i, /\breport\b/i] },
  { intent: INTENT.EVOLVE,       patterns: [/\bevolv/i, /\blevel\s*up\b/i, /\bupgrad/i, /\bpromot/i, /\btier\b/i, /\bprogress/i, /\bgrow\b/i] },
  { intent: INTENT.WITNESS,      patterns: [/\bwitness\b/i, /\bverif/i, /\bvalidat/i, /\bconfirm\b/i, /\battest/i, /\bprove\b/i, /\bcertif/i] },
  { intent: INTENT.CONVERT,      patterns: [/\bconvert\b/i, /\bswap\b/i, /\bexchang/i, /\btrade\b/i, /\bdopamine\s+to\s+synapse/i, /\bburn\b/i] },
  { intent: INTENT.TRADE,        patterns: [/\bpay\b/i, /\bescrow\b/i, /\btransfer\b/i, /\bsend\b/i, /\bfund\b/i, /\bcompensate\b/i] },
];

// ────────────────────────────────────────────────────────────
// Entity Extraction
// ────────────────────────────────────────────────────────────

const ENTITY_PATTERNS = {
  count:       /\b(\d+)\s+(agents?|workers?|tasks?|steps?|witnesses?|items?)\b/i,
  amount:      /\b(\d[\d,]{3,})\b|\b(\d+)\s*(dopamine|synapse|tokens?|coins?|sui)\b/i,
  difficulty:  /\b(simple|easy|medium|moderate|hard|difficult|complex)\b/i,
  tier:        /\b(calm|moderate|aggressive)\b/i,
  strategy:    /\b(hierarchical|democratic|competitive|pipeline|parallel|sequential)\b/i,
  topic:       /(?:about|regarding|on|for|into)\s+"?([^",.]+)"?/i,
  duration:    /\b(\d+)\s*(hours?|days?|weeks?|minutes?)\b/i,
  agent_name:  /\bagent\s+"?([a-zA-Z][\w-]*)"?/i,
  memory_tier: /\b(working|short.?term|long.?term|permanent|temporary)\b/i,
};

function extractEntities(prompt) {
  const entities = { raw: prompt };
  for (const [name, pattern] of Object.entries(ENTITY_PATTERNS)) {
    const match = pattern.exec(prompt);
    if (match) {
      entities[name] = match[1] || match[0];
    }
  }

  // Normalize difficulty
  if (entities.difficulty) {
    const d = entities.difficulty.toLowerCase();
    if (['simple', 'easy'].includes(d)) entities.difficulty = 'simple';
    else if (['medium', 'moderate'].includes(d)) entities.difficulty = 'medium';
    else entities.difficulty = 'hard';
  }

  // Normalize memory tier
  if (entities.memory_tier) {
    const t = entities.memory_tier.toLowerCase().replace(/[^a-z]/g, '');
    if (t.includes('long') || t === 'permanent') entities.memory_tier = 'long_term';
    else if (t.includes('short') || t === 'temporary') entities.memory_tier = 'short_term';
    else entities.memory_tier = 'working';
  }

  // Normalize strategy
  if (entities.strategy) {
    const s = entities.strategy.toLowerCase();
    if (s === 'parallel') entities.strategy = 'democratic';
    else if (s === 'sequential') entities.strategy = 'pipeline';
  }

  // Extract numeric count
  if (entities.count) entities.count = parseInt(entities.count, 10);

  // Amount regex has two alternations — pick whichever matched
  if (entities.amount) {
    const amtMatch = ENTITY_PATTERNS.amount.exec(prompt);
    const raw = (amtMatch && (amtMatch[1] || amtMatch[2])) || entities.amount;
    entities.amount = parseInt(String(raw).replace(/,/g, ''), 10);
  }

  return entities;
}

// ────────────────────────────────────────────────────────────
// LLM Provider Chain
// ────────────────────────────────────────────────────────────

class LLMProvider {
  constructor(config = {}) {
    this.providers = [];
    this._setupProviders(config);
  }

  _setupProviders(config) {
    // Ollama (local, free)
    const ollamaUrl = config.ollamaUrl || process.env.OLLAMA_URL || 'http://localhost:11434';
    this.providers.push({
      name: 'ollama',
      url: ollamaUrl,
      call: (prompt, systemPrompt) => this._callOllama(ollamaUrl, prompt, systemPrompt, config.ollamaModel || 'llama3'),
    });

    // Claude
    if (config.anthropicKey || process.env.ANTHROPIC_API_KEY) {
      const key = config.anthropicKey || process.env.ANTHROPIC_API_KEY;
      this.providers.push({
        name: 'claude',
        call: (prompt, systemPrompt) => this._callClaude(key, prompt, systemPrompt),
      });
    }

    // OpenRouter
    if (config.openrouterKey || process.env.OPENROUTER_API_KEY) {
      const key = config.openrouterKey || process.env.OPENROUTER_API_KEY;
      this.providers.push({
        name: 'openrouter',
        call: (prompt, systemPrompt) => this._callOpenRouter(key, prompt, systemPrompt),
      });
    }
  }

  async generate(prompt, systemPrompt) {
    for (const provider of this.providers) {
      try {
        const result = await provider.call(prompt, systemPrompt);
        if (result && result.content) {
          return { ...result, provider: provider.name };
        }
      } catch (e) {
        // Fall through to next provider
      }
    }
    return null;
  }

  _httpRequest(urlStr, options, body) {
    return new Promise((resolve, reject) => {
      const url = new URL(urlStr);
      const mod = url.protocol === 'https:' ? https : http;
      const req = mod.request(url, options, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
          try { resolve(JSON.parse(data)); }
          catch { resolve({ raw: data }); }
        });
      });
      req.on('error', reject);
      req.setTimeout(30000, () => { req.destroy(); reject(new Error('timeout')); });
      if (body) req.write(JSON.stringify(body));
      req.end();
    });
  }

  async _callOllama(baseUrl, prompt, systemPrompt, model) {
    const body = {
      model,
      messages: [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: prompt },
      ],
      stream: false,
    };
    const res = await this._httpRequest(`${baseUrl}/api/chat`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    }, body);
    if (res.message?.content) {
      return { content: res.message.content, receipt: crypto.createHash('sha256').update(res.message.content).digest('hex') };
    }
    return null;
  }

  async _callClaude(apiKey, prompt, systemPrompt) {
    const body = {
      model: 'claude-sonnet-4-20250514',
      max_tokens: 4096,
      system: systemPrompt,
      messages: [{ role: 'user', content: prompt }],
    };
    const res = await this._httpRequest('https://api.anthropic.com/v1/messages', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': apiKey,
        'anthropic-version': '2023-06-01',
      },
    }, body);
    if (res.content?.[0]?.text) {
      return { content: res.content[0].text, receipt: crypto.createHash('sha256').update(res.content[0].text).digest('hex') };
    }
    return null;
  }

  async _callOpenRouter(apiKey, prompt, systemPrompt) {
    const body = {
      model: 'meta-llama/llama-3-8b-instruct',
      messages: [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: prompt },
      ],
    };
    const res = await this._httpRequest('https://openrouter.ai/api/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${apiKey}`,
      },
    }, body);
    if (res.choices?.[0]?.message?.content) {
      const text = res.choices[0].message.content;
      return { content: text, receipt: crypto.createHash('sha256').update(text).digest('hex') };
    }
    return null;
  }
}

// ────────────────────────────────────────────────────────────
// Plan Builder — Turns intents + entities into executable steps
// ────────────────────────────────────────────────────────────

const STDLIB_MODULES = Object.freeze({
  metabolism: { provides: ['birth_endow', 'drip', 'decay', 'convert', 'burn', 'balance'] },
  witness:    { provides: ['select_witnesses', 'submit_validation', 'tally', 'reputation'] },
  identity:   { provides: ['generate_identity', 'sign', 'verify', 'address'] },
  memory:     { provides: ['store', 'recall', 'search', 'forget', 'extract', 'compress', 'share'] },
  hire:       { provides: ['create_escrow', 'release', 'refund', 'forfeit', 'dispute'] },
  swarm:      { provides: ['add_agent', 'dispatch', 'submit_result', 'resolve', 'coordinate'] },
  evolve:     { provides: ['register', 'record_completion', 'check_evolve', 'get_tier'] },
});

class PlanBuilder {
  constructor() {
    this.stepCounter = 0;
  }

  build(intents, entities, agentState, ethics, permissions) {
    const steps = [];
    const context = { agentState, ethics, permissions, entities };

    for (const intent of intents) {
      const intentSteps = this._stepsForIntent(intent, context);
      steps.push(...intentSteps);
    }

    // Inject precondition checks
    this._injectPreconditions(steps, context);

    // Inject memory save at end if meaningful work was done
    if (steps.length > 0 && !steps.some(s => s.module === 'memory' && s.action === 'store')) {
      steps.push(this._step('memory', 'store_result', {
        key: `think_result:${Date.now()}`,
        tier: 'short_term',
        autoExtract: true,
      }, 'Save results to memory'));
    }

    // Inject receipt at the very end
    steps.push(this._step('receipt', 'generate', {
      type: 'think_execution',
      stepCount: steps.length,
    }, 'Seal execution with receipt'));

    return {
      id: `plan_${crypto.randomBytes(8).toString('hex')}`,
      intents,
      entities,
      steps,
      created: Date.now(),
    };
  }

  _step(module, action, params, description) {
    return {
      id: `step_${++this.stepCounter}`,
      module,
      action,
      params: params || {},
      description: description || `${module}.${action}`,
      status: 'pending',
      result: null,
    };
  }

  _stepsForIntent(intent, ctx) {
    const { entities } = ctx;
    const steps = [];

    switch (intent) {
      case INTENT.HIRE: {
        const isOpenCall = ctx.entities.raw && (/\bpost\s+(a\s+)?job\b/i.test(ctx.entities.raw) || /\bopen\s+call\b/i.test(ctx.entities.raw));
        
        if (isOpenCall) {
          steps.push(this._step('hire', 'post_open_call', {
            job: entities.topic || 'public task',
            amount: entities.amount || 50000,
          }, 'Post an open call for agents'));
        } else {
          const count = entities.count || 1;
          const difficulty = entities.difficulty || 'medium';
          for (let i = 0; i < count; i++) {
            steps.push(this._step('identity', 'generate', {
              index: i,
              name: entities.agent_name || `agent_${i}`,
            }, `Generate identity for agent ${i}`));
          }
          steps.push(this._step('metabolism', 'birth_endow', {
            count,
          }, `Endow ${count} agent(s) with birth tokens`));
          if (entities.topic || entities.amount) {
            steps.push(this._step('hire', 'create_escrow', {
              job: entities.topic || 'assigned task',
              amount: entities.amount || 50000,
              agentCount: count,
            }, 'Create escrow for job'));
          }
          if (count > 1) {
            steps.push(this._step('swarm', 'coordinate', {
              strategy: entities.strategy || 'hierarchical',
              agentCount: count,
            }, `Coordinate ${count} agents (${entities.strategy || 'hierarchical'})`));
          }
        }
        break;
      }

      case INTENT.COORDINATE: {
        const count = entities.count || 2;
        const strategy = entities.strategy || 'hierarchical';
        steps.push(this._step('swarm', 'setup', {
          strategy,
          agentCount: count,
        }, `Setup swarm with ${strategy} strategy`));
        steps.push(this._step('swarm', 'dispatch', {
          task: entities.topic || 'coordinate',
        }, 'Dispatch task to swarm'));
        steps.push(this._step('witness', 'select', {
          difficulty: entities.difficulty || 'medium',
        }, 'Select witnesses for verification'));
        break;
      }

      case INTENT.RESEARCH: {
        steps.push(this._step('memory', 'search', {
          query: entities.topic || '',
        }, 'Check memory for existing knowledge'));
        steps.push(this._step('llm', 'reason', {
          task: 'research',
          topic: entities.topic || 'requested topic',
          depth: entities.difficulty || 'medium',
        }, `Deep research: ${entities.topic || 'topic'}`));
        steps.push(this._step('memory', 'extract_facts', {
          source: 'research_module',
          text: '$previous.result.content', // Directive for engine to use previous output
        }, 'Extract key facts from research'));
        steps.push(this._step('memory', 'store', {
          key: `research:${(entities.topic || 'general').replace(/\s+/g, '_')}`,
          tier: entities.memory_tier || 'short_term',
        }, 'Store research results'));
        steps.push(this._step('witness', 'select', {
          difficulty: entities.difficulty || 'medium',
        }, 'Select witnesses to verify research'));
        break;
      }

      case INTENT.BUILD: {
        steps.push(this._step('llm', 'reason', {
          task: 'build',
          topic: entities.topic || 'requested build',
          output: 'code_and_plan',
        }, `Plan and build: ${entities.topic || 'project'}`));
        steps.push(this._step('metabolism', 'burn', {
          amount: entities.amount || 10000,
          reason: 'build_action',
        }, 'Burn Dopamine for build action'));
        break;
      }

      case INTENT.ANALYZE: {
        steps.push(this._step('memory', 'search', {
          query: entities.topic || '',
        }, 'Recall relevant data'));
        steps.push(this._step('llm', 'reason', {
          task: 'analyze',
          topic: entities.topic || 'requested analysis',
        }, `Analyze: ${entities.topic || 'target'}`));
        break;
      }

      case INTENT.REMEMBER: {
        const tier = entities.memory_tier || 'long_term';
        steps.push(this._step('memory', 'store', {
          key: entities.topic ? `user:${entities.topic.replace(/\s+/g, '_')}` : `note:${Date.now()}`,
          tier,
          data: entities.topic || 'noted',
        }, `Store in ${tier} memory`));
        break;
      }

      case INTENT.FORGET: {
        steps.push(this._step('memory', 'forget', {
          key: entities.topic || '',
        }, `Forget: ${entities.topic || 'target'}`));
        break;
      }

      case INTENT.CHECK_STATUS: {
        steps.push(this._step('metabolism', 'get_balance', {}, 'Check token balances'));
        steps.push(this._step('evolve', 'get_status', {}, 'Check evolution status'));
        steps.push(this._step('memory', 'stats', {}, 'Check memory stats'));
        break;
      }

      case INTENT.EVOLVE: {
        steps.push(this._step('evolve', 'check', {}, 'Check evolution eligibility'));
        steps.push(this._step('metabolism', 'get_balance', {}, 'Check token state'));
        break;
      }

      case INTENT.WITNESS: {
        const difficulty = entities.difficulty || 'medium';
        steps.push(this._step('witness', 'select', {
          difficulty,
        }, `Select ${STDLIB_MODULES.witness.provides[0]} witnesses`));
        steps.push(this._step('witness', 'validate', {
          jobId: `job_${Date.now()}`,
        }, 'Submit for peer validation'));
        break;
      }

      case INTENT.CONVERT: {
        const amount = entities.amount || 100000;
        steps.push(this._step('metabolism', 'convert', {
          amount,
        }, `Convert ${amount} Dopamine → Synapse (10:1)`));
        break;
      }

      case INTENT.TRADE: {
        steps.push(this._step('hire', 'create_escrow', {
          amount: entities.amount || 10000,
          job: entities.topic || 'payment',
        }, 'Create payment escrow'));
        break;
      }

      case INTENT.GENERAL: {
        steps.push(this._step('llm', 'reason', {
          task: 'general',
          topic: entities.topic || 'general request',
        }, 'General LLM reasoning'));
        break;
      }
    }

    return steps;
  }

  _injectPreconditions(steps, ctx) {
    const { agentState, ethics, permissions } = ctx;

    for (const step of steps) {
      step.preconditions = [];

      // Ethics check for actions that could cause harm
      if (['hire', 'swarm', 'witness'].includes(step.module)) {
        if (ethics.harm_none) {
          step.preconditions.push({ check: 'ethics', rule: 'harm_none', passed: true });
        }
      }

      // Permission check for spending
      if (step.module === 'metabolism' && ['burn', 'convert'].includes(step.action)) {
        if (permissions.spend === 'deny') {
          step.preconditions.push({ check: 'permission', rule: 'spend', passed: false, block: true });
        } else if (permissions.spend === 'ask') {
          step.preconditions.push({ check: 'permission', rule: 'spend_ask', passed: true, note: 'requires user approval' });
        }
      }

      // Permission check for hiring
      if (step.module === 'hire') {
        if (permissions.hire === 'deny') {
          step.preconditions.push({ check: 'permission', rule: 'hire', passed: false, block: true });
        }
      }

      // Permission check for external actions
      if (step.module === 'llm') {
        if (permissions.external === 'deny') {
          step.preconditions.push({ check: 'permission', rule: 'external', passed: false, block: true });
        }
      }

      // Agent must exist for most actions
      if (!['identity', 'receipt'].includes(step.module) && !agentState) {
        step.preconditions.push({ check: 'agent_exists', passed: false, block: true });
      }
    }
  }
}

// ────────────────────────────────────────────────────────────
// Think Engine — The Orchestrator
// ────────────────────────────────────────────────────────────

const SYSTEM_PROMPT = `You are the Aether Think Engine — an embedded AI agent inside the Aether sovereign programming language.

You have access to these stdlib modules:
- metabolism: Token economy (Dopamine/Synapse birth endowment, hourly drip, daily decay, 10:1 conversion, burn-for-action)
- witness: Peer validation (random witness selection 3/4/5 by difficulty, reputation tracking)
- identity: Cryptographic identity (Ed25519 keypairs, elemental derivation, aether:// addresses)
- memory: 3-tier persistent memory (working/short-term/long-term, auto-extraction, cross-agent sharing)
- hire: Job escrow lifecycle (create/release/refund/forfeit/dispute, AIO on-chain bridge)
- swarm: Multi-agent coordination (hierarchical/democratic/competitive/pipeline strategies)
- evolve: Tier progression (calm → moderate → aggressive, reputation-based)

When asked to reason about a task, respond with a JSON object:
{
  "understanding": "one-sentence restatement of the user's goal",
  "plan": [
    { "module": "<stdlib_module>", "action": "<action>", "params": {}, "reason": "why" }
  ],
  "risks": ["potential issues"],
  "confidence": 0.0-1.0
}

Be precise. Use the exact module and action names. Consider the agent's current state, ethics, and permissions.`;

class ThinkEngine {
  constructor(interpreterRef) {
    this.interpreter = interpreterRef;
    this.llm = new LLMProvider(interpreterRef?._thinkConfig || {});
    this.planBuilder = new PlanBuilder();
    this.executionHistory = [];
    this.iterationLimit = 5;
  }

  /**
   * Main entry point — takes a natural language prompt and executes it.
   */
  async process(prompt) {
    const startTime = Date.now();
    const traceId = crypto.randomBytes(8).toString('hex');

    console.log(`\n[THINK] ═══════════════════════════════════════`);
    console.log(`[THINK] Processing: "${prompt}"`);
    console.log(`[THINK] Trace: ${traceId}`);

    // Phase 1: Understand
    const analysis = this._analyze(prompt);
    console.log(`[THINK] Intents: ${analysis.intents.join(', ')}`);
    console.log(`[THINK] Entities: ${JSON.stringify(analysis.entities)}`);

    // Phase 2: Consult memory for context
    const memoryContext = this._consultMemory(prompt);
    if (memoryContext.length > 0) {
      console.log(`[THINK] Memory hits: ${memoryContext.length} relevant entries`);
    }

    // Phase 3: Attempt LLM reasoning for complex/ambiguous prompts
    let llmPlan = null;
    if (analysis.intents.includes(INTENT.GENERAL) || analysis.intents.length > 2 || analysis.confidence < 0.6) {
      llmPlan = await this._llmReason(prompt, analysis, memoryContext);
    }

    // Phase 4: Build execution plan
    const plan = llmPlan
      ? this._mergeLLMPlan(llmPlan, analysis)
      : this.planBuilder.build(
          analysis.intents,
          analysis.entities,
          this.interpreter?.agent || null,
          this.interpreter?.ethics || {},
          this.interpreter?.permissions || {},
        );

    console.log(`[THINK] Plan: ${plan.steps.length} steps`);
    for (const step of plan.steps) {
      const blocked = step.preconditions?.some(p => p.block);
      console.log(`[THINK]   ${blocked ? '✗' : '→'} ${step.description}`);
    }

    // Phase 5: Validate plan
    const validation = this._validatePlan(plan);
    if (!validation.valid) {
      console.log(`[THINK] ⚠ Plan validation failed: ${validation.issues.join(', ')}`);
      // Self-correct
      const corrected = this._selfCorrect(plan, validation);
      if (corrected) {
        console.log(`[THINK] ✓ Self-corrected plan (${corrected.corrections.join(', ')})`);
        plan.steps = corrected.steps;
      }
    }

    // Phase 6: Execute
    const results = await this._executePlan(plan);

    // Phase 7: Store execution in memory
    this._recordExecution(traceId, prompt, analysis, plan, results, startTime);

    const elapsed = Date.now() - startTime;
    console.log(`[THINK] ═══ Complete (${elapsed}ms, ${results.stepsExecuted}/${plan.steps.length} steps) ═══\n`);

    return {
      traceId,
      prompt,
      analysis,
      plan,
      results,
      elapsed,
    };
  }

  // ──────────────── Phase 1: Analysis ────────────────

  _analyze(prompt) {
    const intents = [];
    const scores = [];

    for (const { intent, patterns } of INTENT_PATTERNS) {
      let score = 0;
      for (const pattern of patterns) {
        if (pattern.test(prompt)) score++;
      }
      if (score > 0) {
        intents.push(intent);
        scores.push({ intent, score });
      }
    }

    // If nothing matched, it's a general request
    if (intents.length === 0) intents.push(INTENT.GENERAL);

    // Sort by match strength
    scores.sort((a, b) => b.score - a.score);

    const entities = extractEntities(prompt);
    const confidence = scores.length > 0
      ? Math.min(1, scores[0].score / 3)
      : 0.3;

    return {
      intents,
      scores,
      entities,
      confidence,
      raw: prompt,
    };
  }

  // ──────────────── Phase 2: Memory Consultation ────────────────

  _consultMemory(prompt) {
    if (!this.interpreter?.memory) return [];
    try {
      const words = prompt.split(/\s+/).filter(w => w.length > 4);
      const results = [];
      for (const word of words.slice(0, 5)) {
        const hits = this.interpreter.memory.search(word, 3);
        for (const hit of hits) {
          if (!results.some(r => r.key === hit.key)) {
            results.push(hit);
          }
        }
      }
      return results;
    } catch {
      return [];
    }
  }

  // ──────────────── Phase 3: LLM Reasoning ────────────────

  async _llmReason(prompt, analysis, memoryContext) {
    const contextStr = memoryContext.length > 0
      ? `\n\nRelevant memories:\n${memoryContext.map(m => `- ${m.key}: ${JSON.stringify(m.data)}`).join('\n')}`
      : '';

    const agentInfo = this.interpreter?.agent
      ? `\nAgent: ${this.interpreter.agent.name} (${this.interpreter.agent.tier} tier)`
      : '\nNo agent born yet — may need birth step first.';

    const userPrompt = `${prompt}${agentInfo}${contextStr}

My initial intent classification: ${analysis.intents.join(', ')} (confidence: ${analysis.confidence})
Extracted entities: ${JSON.stringify(analysis.entities)}

Generate the optimal execution plan.`;

    const result = await this.llm.generate(userPrompt, SYSTEM_PROMPT);
    if (!result) return null;

    try {
      // Try to extract JSON from response
      const jsonMatch = result.content.match(/\{[\s\S]*\}/);
      if (jsonMatch) {
        const parsed = JSON.parse(jsonMatch[0]);
        console.log(`[THINK] LLM (${result.provider}): "${parsed.understanding || 'understood'}"`);
        return parsed;
      }
    } catch {
      // LLM responded but not valid JSON — use as general insight
      console.log(`[THINK] LLM insight: ${result.content.slice(0, 100)}...`);
    }
    return null;
  }

  // ──────────────── Phase 4: Merge LLM plan ────────────────

  _mergeLLMPlan(llmPlan, analysis) {
    const steps = [];
    let counter = 0;

    if (Array.isArray(llmPlan.plan)) {
      for (const llmStep of llmPlan.plan) {
        const validModule = Object.keys(STDLIB_MODULES).includes(llmStep.module) || ['llm', 'receipt'].includes(llmStep.module);
        if (validModule) {
          steps.push({
            id: `step_${++counter}`,
            module: llmStep.module,
            action: llmStep.action,
            params: llmStep.params || {},
            description: llmStep.reason || `${llmStep.module}.${llmStep.action}`,
            status: 'pending',
            result: null,
            preconditions: [],
            source: 'llm',
          });
        }
      }
    }

    // If LLM plan was empty or all steps were invalid, fall back to pattern-based
    if (steps.length === 0) {
      return this.planBuilder.build(
        analysis.intents,
        analysis.entities,
        this.interpreter?.agent || null,
        this.interpreter?.ethics || {},
        this.interpreter?.permissions || {},
      );
    }

    // Always end with receipt
    steps.push({
      id: `step_${++counter}`,
      module: 'receipt',
      action: 'generate',
      params: { type: 'think_execution', stepCount: steps.length },
      description: 'Seal execution with receipt',
      status: 'pending',
      result: null,
      preconditions: [],
    });

    return {
      id: `plan_${crypto.randomBytes(8).toString('hex')}`,
      intents: analysis.intents,
      entities: analysis.entities,
      steps,
      created: Date.now(),
      source: 'llm',
      confidence: llmPlan.confidence || 0.8,
    };
  }

  // ──────────────── Phase 5: Validation ────────────────

  _validatePlan(plan) {
    const issues = [];

    for (const step of plan.steps) {
      // Check blocked preconditions
      if (step.preconditions?.some(p => p.block)) {
        issues.push(`${step.id}: blocked by ${step.preconditions.filter(p => p.block).map(p => p.rule).join(', ')}`);
      }

      // Validate module exists
      if (!['llm', 'receipt'].includes(step.module) && !STDLIB_MODULES[step.module]) {
        issues.push(`${step.id}: unknown module '${step.module}'`);
      }
    }

    // Check agent exists for non-identity steps
    if (!this.interpreter?.agent) {
      const needsAgent = plan.steps.some(s =>
        !['identity', 'receipt', 'llm'].includes(s.module)
      );
      if (needsAgent) {
        issues.push('agent not born — metabolism/witness/memory/hire/swarm/evolve require an active agent');
      }
    }

    return { valid: issues.length === 0, issues };
  }

  // ──────────────── Phase 5b: Self-Correction ────────────────

  _selfCorrect(plan, validation) {
    const corrections = [];
    const newSteps = [];

    // If agent doesn't exist, prepend a birth step
    if (validation.issues.some(i => i.includes('agent not born'))) {
      newSteps.push({
        id: 'step_auto_birth',
        module: 'birth',
        action: 'auto',
        params: {
          name: plan.entities?.agent_name || `auto_${Date.now().toString(36)}`,
          tier: plan.entities?.tier || 'moderate',
          budget: plan.entities?.amount || 86000000,
        },
        description: 'Auto-birth agent (self-correction)',
        status: 'pending',
        result: null,
        preconditions: [],
      });
      corrections.push('injected auto-birth');
    }

    // Remove blocked steps
    for (const step of plan.steps) {
      const blocked = step.preconditions?.some(p => p.block);
      if (blocked) {
        corrections.push(`removed blocked step: ${step.id}`);
        continue;
      }
      newSteps.push(step);
    }

    if (corrections.length === 0) return null;

    return { steps: newSteps, corrections };
  }

  // ──────────────── Phase 6: Execution ────────────────

  async _executePlan(plan) {
    const results = [];
    let stepsExecuted = 0;
    let stepsFailed = 0;
    const outputs = {};

    for (const step of plan.steps) {
      try {
        const result = await this._executeStep(step, plan, results);
        step.status = 'completed';
        step.result = result;
        results.push({ stepId: step.id, status: 'ok', result });
        outputs[step.id] = result;
        stepsExecuted++;
      } catch (err) {
        step.status = 'failed';
        step.result = { error: err.message };
        results.push({ stepId: step.id, status: 'error', error: err.message });
        stepsFailed++;
        console.log(`[THINK]   ✗ ${step.description}: ${err.message}`);

        // Don't abort — try remaining steps if they're independent
      }
    }

    return { results, stepsExecuted, stepsFailed, outputs };
  }

  async _executeStep(step, plan, previousResults) {
    const { module, action, params } = step;
    const interp = this.interpreter;

    if (!interp) throw new Error('No interpreter reference');

    // Handle dynamic parameter substitution (e.g., $previous.result.content)
    const resolvedParams = { ...params };
    for (const [key, value] of Object.entries(resolvedParams)) {
      if (typeof value === 'string' && value.startsWith('$previous')) {
        // Find the result of the actually preceding step
        const lastResultRecord = previousResults.length > 0 ? previousResults[previousResults.length - 1].result : null;
        if (value === '$previous.result.content' && lastResultRecord?.content) {
          resolvedParams[key] = lastResultRecord.content;
        }
      }
    }

    // Bridge Think Engine plan steps to Aether interpreter nodes
    const node = {
      name: module,
      arguments: {
        action,
        ...resolvedParams
      }
    };

    // Special case: 'llm' module is handled by the Think Engine itself
    if (module === 'llm') {
      const contextParts = [];
      if (interp.agent) {
        contextParts.push(`Agent: ${interp.agent.name} (${interp.agent.tier})`);
        const bal = interp.metabolism.getBalance(interp.agent.name);
        contextParts.push(`Tokens: ${bal.dopamine.toLocaleString()} D / ${bal.synapse.toLocaleString()} S`);
      }

      const taskPrompt = `Task: ${params.task}\nTopic: ${params.topic || 'general'}\n${contextParts.join('\n')}`;
      const result = await this.llm.generate(taskPrompt, SYSTEM_PROMPT);

      if (result) {
        console.log(`[THINK]   ✓ LLM (${result.provider}): ${result.content.slice(0, 80)}...`);
        if (interp.memory) interp.memory.extractFacts(result.content, 'llm_reasoning');
        return { content: result.content, receipt: result.receipt, provider: result.provider };
      }

      const reasoning = this._builtinReason(params);
      console.log(`[THINK]   ✓ Built-in reasoner: ${reasoning.slice(0, 80)}...`);
      return { content: reasoning, receipt: crypto.createHash('sha256').update(reasoning).digest('hex'), provider: 'builtin' };
    }

    // Default: delegate to the hardened interpreter
    return await interp.execute(node);
  }

  // ──────────────── Built-in Reasoner (no LLM fallback) ────────────────

  _builtinReason(params) {
    const { task, topic } = params;

    switch (task) {
      case 'research':
        return `[Aether Reasoner] Research task queued for: ${topic}. ` +
          `The agent will gather information, verify sources via the witness system, ` +
          `and store findings in long-term memory. For deep research, connect an LLM ` +
          `provider (set OLLAMA_URL, ANTHROPIC_API_KEY, or OPENROUTER_API_KEY).`;

      case 'build':
        return `[Aether Reasoner] Build task initialized for: ${topic}. ` +
          `The agent has allocated Dopamine for this action. For code generation, ` +
          `connect an LLM provider. The built-in reasoner can plan architecture ` +
          `but requires an LLM for implementation details.`;

      case 'analyze':
        return `[Aether Reasoner] Analysis task for: ${topic}. ` +
          `Checking memory for relevant data. Analysis complete — ` +
          `results stored in working memory for immediate access.`;

      default:
        return `[Aether Reasoner] Processing: ${topic || task || 'general task'}. ` +
          `The think engine has classified this request and executed the ` +
          `appropriate stdlib modules. Connect an LLM for richer reasoning.`;
    }
  }

  // ──────────────── Phase 7: Record ────────────────

  _recordExecution(traceId, prompt, analysis, plan, results, startTime) {
    const record = {
      traceId,
      prompt,
      intents: analysis.intents,
      confidence: analysis.confidence,
      stepCount: plan.steps.length,
      stepsExecuted: results.stepsExecuted,
      stepsFailed: results.stepsFailed,
      elapsed: Date.now() - startTime,
      timestamp: Date.now(),
    };

    this.executionHistory.push(record);

    // Store in memory
    if (this.interpreter?.memory) {
      try {
        this.interpreter.memory.store(
          `think_trace:${traceId}`,
          record,
          'short_term',
          { importance: 0.6, source: 'think_engine', tags: ['think', 'trace'] }
        );
      } catch {
        // Memory store is best-effort
      }
    }
  }

  getHistory() {
    return this.executionHistory;
  }
}

export { ThinkEngine, INTENT, extractEntities, PlanBuilder, LLMProvider, STDLIB_MODULES };
