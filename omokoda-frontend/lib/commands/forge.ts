/**
 * CommandForge — Pattern 73.
 *
 * Parses markdown files into slash command definitions and resolves
 * invocations by substituting `{{arg}}` placeholders in templates.
 */

import type { CommandDefinition, CommandArgument, CommandInvocation } from './types';

export class CommandForge {
  private commands: Map<string, CommandDefinition> = new Map();

  /** Register a command definition. */
  register(cmd: CommandDefinition): void {
    this.commands.set(cmd.name, { ...cmd });
  }

  /** Parse a markdown string into a CommandDefinition. */
  static parse(markdown: string, source?: string): CommandDefinition | null {
    const trimmed = markdown.trim();

    if (!trimmed.startsWith('---')) return null;

    const rest = trimmed.slice(3);
    const endIdx = rest.indexOf('\n---');
    if (endIdx === -1) return null;

    const frontmatterStr = rest.slice(0, endIdx);
    const template = rest.slice(endIdx + 4).trim();

    // Parse simple key: value frontmatter
    const frontmatter: Record<string, string> = {};
    for (const line of frontmatterStr.split('\n')) {
      const colonIdx = line.indexOf(':');
      if (colonIdx !== -1) {
        const key = line.slice(0, colonIdx).trim();
        const value = line.slice(colonIdx + 1).trim();
        frontmatter[key] = value;
      }
    }

    const name = frontmatter['name'];
    if (!name) return null;

    // Parse arguments from frontmatter "arguments" key (comma-separated names)
    const argNames = frontmatter['arguments']
      ? frontmatter['arguments'].split(',').map((a) => a.trim())
      : [];

    const args: CommandArgument[] = argNames.map((argName) => ({
      name: argName,
      description: frontmatter[`arg.${argName}.description`] ?? argName,
      required: frontmatter[`arg.${argName}.required`] !== 'false',
      type: (frontmatter[`arg.${argName}.type`] as CommandArgument['type']) ?? 'string',
      default: frontmatter[`arg.${argName}.default`],
    }));

    return {
      name,
      description: frontmatter['description'] ?? '',
      arguments: args,
      template,
      frontmatter,
      source,
    };
  }

  /** Resolve a command invocation, substituting `{{arg}}` placeholders. */
  invoke(name: string, args: Record<string, string> = {}): CommandInvocation | null {
    const cmd = this.commands.get(name);
    if (!cmd) return null;

    const resolvedPrompt = this.resolveTemplate(cmd.template, args);
    return { command: cmd, args, resolvedPrompt };
  }

  /** Substitute `{{argName}}` placeholders with values from args. */
  resolveTemplate(template: string, args: Record<string, string>): string {
    return template.replace(/\{\{(\w+)\}\}/g, (_, key: string) => args[key] ?? `{{${key}}}`);
  }

  get(name: string): CommandDefinition | undefined {
    return this.commands.get(name);
  }

  list(): CommandDefinition[] {
    return Array.from(this.commands.values()).sort((a, b) => a.name.localeCompare(b.name));
  }

  /** Search commands by name or description. */
  search(query: string): CommandDefinition[] {
    const q = query.toLowerCase();
    return this.list().filter(
      (c) => c.name.toLowerCase().includes(q) || c.description.toLowerCase().includes(q),
    );
  }

  unregister(name: string): boolean {
    return this.commands.delete(name);
  }

  count(): number {
    return this.commands.size;
  }
}
