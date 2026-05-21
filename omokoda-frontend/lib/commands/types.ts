/**
 * CommandForge types — Pattern 73.
 * Slash commands defined as markdown files with YAML frontmatter,
 * mirroring Claw's `commit-commands/commands/*.md` pattern.
 */

export interface CommandArgument {
  name: string;
  description: string;
  required: boolean;
  type: 'string' | 'number' | 'boolean' | 'file';
  default?: string;
}

/** A slash command definition parsed from a markdown file. */
export interface CommandDefinition {
  /** Command name without leading slash (e.g. "pr-review") */
  name: string;
  description: string;
  /** Arguments the command accepts */
  arguments: CommandArgument[];
  /** The markdown body — used as the command's prompt template.
   *  Supports `{{arg}}` placeholder substitution. */
  template: string;
  /** Raw YAML frontmatter */
  frontmatter: Record<string, string>;
  /** Source path or identifier */
  source?: string;
}

export interface CommandInvocation {
  command: CommandDefinition;
  /** Resolved argument values */
  args: Record<string, string>;
  /** The final prompt after template substitution */
  resolvedPrompt: string;
}
