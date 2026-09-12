/**
 * Swibe Think Loop — Agentic Iteration Engine
 *
 * Upgrades `think` from single-shot to iterative tool-call loops.
 * The agent can think, invoke tools, observe results, and think again
 * until it reaches a goal or exhausts its budget.
 *
 * Flow:
 *   think "goal" { loop: true, tools: [...], max_iterations: 10 }
 *     → LLM generates response
 *     → If response contains tool_call → execute tool → feed result back
 *     → Repeat until goal achieved or max_iterations reached
 *     → Seal receipt chain for the full trajectory
 */

import crypto from 'node:crypto';
import { LLMIntegration } from './llm-integration.js';

// ────────────────────────────────────────────────────────────
// Think Loop Configuration
// ────────────────────────────────────────────────────────────

const DEFAULT_LOOP_CONFIG = {
  loop: false,
  max_iterations: 10,
  tools: [],
  stream: false,
  budget_tokens: 100_000,
  budget_usd: null,
  on_tool_call: null,     // Callback: (toolName, args) => void
  on_iteration: null,     // Callback: (iteration, response) => void
  stop_phrases: [],       // Phrases that signal goal completion
  temperature: 0.7,
};

// ────────────────────────────────────────────────────────────
// Tool Registry for Think Loops
// ────────────────────────────────────────────────────────────

class ToolRegistry {
  constructor() {
    this.tools = new Map();
  }

  /**
   * Register a tool that the think loop can invoke.
   * @param {string} name
   * @param {string} description
   * @param {object} inputSchema — JSON Schema for the tool's parameters
   * @param {function} handler — Async function(args) => result
   */
  register(name, description, inputSchema, handler) {
    this.tools.set(name, { name, description, inputSchema, handler });
  }

  get(name) {
    return this.tools.get(name);
  }

  list() {
    return Array.from(this.tools.values()).map(t => ({
      name: t.name,
      description: t.description,
      inputSchema: t.inputSchema,
    }));
  }

  async invoke(name, args) {
    const tool = this.tools.get(name);
    if (!tool) throw new Error(`Unknown tool: ${name}`);
    return tool.handler(args);
  }
}

// ────────────────────────────────────────────────────────────
// Think Loop Executor
// ────────────────────────────────────────────────────────────

class ThinkLoop {
  /**
   * @param {object} config — Merged with DEFAULT_LOOP_CONFIG
   * @param {ToolRegistry} [toolRegistry]
   * @param {object} [permissionGate] — PermissionGate instance
   */
  constructor(config = {}, toolRegistry = null, permissionGate = null) {
    this.config = { ...DEFAULT_LOOP_CONFIG, ...config };
    this.toolRegistry = toolRegistry || new ToolRegistry();
    this.permissionGate = permissionGate;
    this.llm = new LLMIntegration();
    this.trajectory = [];
    this.tokensUsed = 0;
  }

  /**
   * Execute the think loop.
   * @param {string} prompt — The initial goal/prompt
   * @returns {Promise<{content: string, trajectory: object[], iterations: number, receipt: string}>}
   */
  async execute(prompt) {
    if (!this.config.loop) {
      // Single-shot mode — original think behavior
      const result = await this.llm.think(prompt);
      this.trajectory.push({
        iteration: 1,
        type: 'think',
        prompt,
        response: result.content,
        receipt: result.receipt,
      });
      return {
        content: result.content,
        trajectory: this.trajectory,
        iterations: 1,
        receipt: result.receipt,
      };
    }

    // Agentic loop mode
    const conversation = [{ role: 'user', content: prompt }];
    let lastContent = '';
    let iteration = 0;

    while (iteration < this.config.max_iterations) {
      iteration++;

      // Budget check
      if (this.tokensUsed >= this.config.budget_tokens) {
        console.log(`[THINK-LOOP] Budget exhausted at iteration ${iteration} (${this.tokensUsed} tokens)`);
        break;
      }

      // Generate response
      const conversationText = conversation
        .map(m => `${m.role}: ${m.content}`)
        .join('\n');

      const toolDescriptions = this.toolRegistry.list();
      const systemPrompt = toolDescriptions.length > 0
        ? `You have access to these tools:\n${toolDescriptions.map(t =>
            `- ${t.name}: ${t.description}`
          ).join('\n')}\n\nTo use a tool, respond with: [TOOL:tool_name:{"arg":"value"}]\nWhen you've achieved the goal, respond with: [DONE]`
        : '';

      const fullPrompt = systemPrompt
        ? `${systemPrompt}\n\n${conversationText}`
        : conversationText;

      const result = await this.llm.think(fullPrompt);
      const response = result.content || '';
      lastContent = response;

      // Rough token estimate
      this.tokensUsed += Math.ceil((fullPrompt.length + response.length) / 4);

      this.trajectory.push({
        iteration,
        type: 'think',
        prompt: conversation[conversation.length - 1]?.content || '',
        response,
        receipt: result.receipt,
        tokens_estimated: this.tokensUsed,
      });

      if (this.config.on_iteration) {
        this.config.on_iteration(iteration, response);
      }

      // Check for tool calls
      const toolCall = this._parseToolCall(response);
      if (toolCall) {
        // Permission check
        if (this.permissionGate) {
          const { granted, reason } = await this.permissionGate.check('call_tool', {
            tool: toolCall.name,
            args: toolCall.args,
          });
          if (!granted) {
            conversation.push(
              { role: 'assistant', content: response },
              { role: 'user', content: `Tool call denied: ${reason}. Try a different approach.` }
            );
            continue;
          }
        }

        if (this.config.on_tool_call) {
          this.config.on_tool_call(toolCall.name, toolCall.args);
        }

        try {
          const toolResult = await this.toolRegistry.invoke(toolCall.name, toolCall.args);
          const resultStr = typeof toolResult === 'string'
            ? toolResult
            : JSON.stringify(toolResult);

          this.trajectory.push({
            iteration,
            type: 'tool_call',
            tool: toolCall.name,
            args: toolCall.args,
            result: resultStr,
          });

          conversation.push(
            { role: 'assistant', content: response },
            { role: 'user', content: `Tool "${toolCall.name}" returned: ${resultStr}` }
          );
        } catch (e) {
          conversation.push(
            { role: 'assistant', content: response },
            { role: 'user', content: `Tool "${toolCall.name}" failed: ${e.message}` }
          );
        }
        continue;
      }

      // Check for completion
      if (response.includes('[DONE]')) {
        lastContent = response.replace('[DONE]', '').trim();
        break;
      }

      // Check stop phrases
      if (this.config.stop_phrases.some(p => response.toLowerCase().includes(p.toLowerCase()))) {
        break;
      }

      // If no tool call and no done signal, we're in a single-turn or stuck
      conversation.push({ role: 'assistant', content: response });
      break;
    }

    const trajectoryHash = crypto.createHash('sha256')
      .update(JSON.stringify(this.trajectory))
      .digest('hex');

    return {
      content: lastContent,
      trajectory: this.trajectory,
      iterations: iteration,
      tokens_used: this.tokensUsed,
      receipt: trajectoryHash,
    };
  }

  /**
   * Parse a tool call from the LLM response.
   * Format: [TOOL:tool_name:{"arg":"value"}]
   */
  _parseToolCall(response) {
    const match = response.match(/\[TOOL:(\w+):(.*?)\]/s);
    if (!match) return null;

    const name = match[1];
    let args = {};
    try {
      args = JSON.parse(match[2]);
    } catch {
      args = { raw: match[2] };
    }

    return { name, args };
  }
}

// ────────────────────────────────────────────────────────────
// Built-in Tools for Think Loops
// ────────────────────────────────────────────────────────────

function registerBuiltinTools(registry) {
  registry.register(
    'read_file',
    'Read the contents of a file',
    { type: 'object', properties: { path: { type: 'string' } }, required: ['path'] },
    async (args) => {
      const fs = await import('node:fs');
      return fs.readFileSync(args.path, 'utf-8');
    }
  );

  registry.register(
    'write_file',
    'Write content to a file',
    { type: 'object', properties: { path: { type: 'string' }, content: { type: 'string' } }, required: ['path', 'content'] },
    async (args) => {
      const fs = await import('node:fs');
      fs.writeFileSync(args.path, args.content);
      return `Wrote ${args.content.length} bytes to ${args.path}`;
    }
  );

  registry.register(
    'edit_file',
    'Replace a string in a file (partial edit, not full overwrite)',
    {
      type: 'object',
      properties: {
        path: { type: 'string' },
        old_string: { type: 'string' },
        new_string: { type: 'string' },
      },
      required: ['path', 'old_string', 'new_string'],
    },
    async (args) => {
      const fs = await import('node:fs');
      const content = fs.readFileSync(args.path, 'utf-8');
      if (!content.includes(args.old_string)) {
        return `Error: old_string not found in ${args.path}`;
      }
      const updated = content.replace(args.old_string, args.new_string);
      fs.writeFileSync(args.path, updated);
      return `Replaced in ${args.path}`;
    }
  );

  registry.register(
    'list_files',
    'List files in a directory',
    { type: 'object', properties: { path: { type: 'string' } }, required: ['path'] },
    async (args) => {
      const fs = await import('node:fs');
      return fs.readdirSync(args.path).join('\n');
    }
  );

  registry.register(
    'bash',
    'Execute a shell command',
    { type: 'object', properties: { command: { type: 'string' } }, required: ['command'] },
    async (args) => {
      const { execSync } = await import('node:child_process');
      return execSync(args.command, { encoding: 'utf-8', timeout: 30_000 });
    }
  );

  registry.register(
    'pilot',
    'Control the computer (click, type, scroll, etc.)',
    {
      type: 'object',
      properties: {
        action: { type: 'string', enum: ['click', 'type', 'scroll', 'keycombo', 'screenshot'] },
        target: { type: 'string' },
        text: { type: 'string' },
        coordinates: { type: 'array', items: { type: 'number' } },
      },
      required: ['action'],
    },
    async (args) => {
      const { PilotEngine } = await import('./pilot.js');
      const pilot = new PilotEngine();
      return pilot.execute(args);
    }
  );

  registry.register(
    'viewport',
    'Analyze the screen content (OCR, UI elements)',
    {
      type: 'object',
      properties: {
        screenshot: { type: 'string', description: 'Base64 screenshot data (optional, takes fresh one if omitted)' },
      },
    },
    async (args) => {
      const { ViewportEngine } = await import('./viewport.js');
      const { PilotEngine } = await import('./pilot.js');
      let data = args.screenshot;
      if (!data) {
        const pilot = new PilotEngine();
        const res = await pilot.execute({ action: 'screenshot' });
        data = res.data || 'mock_screenshot_data';
      }
      const viewport = new ViewportEngine();
      return viewport.analyze(data);
    }
  );
}

export { ThinkLoop, ToolRegistry, registerBuiltinTools, DEFAULT_LOOP_CONFIG };
