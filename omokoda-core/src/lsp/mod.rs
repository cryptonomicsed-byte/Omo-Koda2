/// Code-intelligence layer: diagnostics and symbol data for think context enrichment.
/// A lightweight client-side representation of LSP state that formats as a `think` prompt section,
/// keeping the actual LSP transport decoupled so any language server can feed into the pipeline.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity level mirroring LSP DiagnosticSeverity (1=Error, 2=Warning, 3=Info, 4=Hint).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Info = 3,
    Hint = 4,
}

impl DiagnosticSeverity {
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warning => "WARN",
            Self::Info => "INFO",
            Self::Hint => "HINT",
        }
    }
}

/// A single LSP diagnostic message attached to a file range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

impl Diagnostic {
    /// Format as a single human-readable line suitable for prompt injection.
    #[must_use]
    pub fn render_line(&self) -> String {
        let source_tag = self
            .source
            .as_deref()
            .map(|s| format!("[{}] ", s))
            .unwrap_or_default();
        format!(
            "{}:{}: {} {}{}",
            self.file,
            self.line,
            self.severity.label(),
            source_tag,
            self.message
        )
    }
}

/// A symbol definition or reference returned by go-to-definition / find-references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolLocation {
    pub name: String,
    pub kind: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// A hover / documentation snippet from the language server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverInfo {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub content: String,
}

/// Collected LSP context for one work session — diagnostics, symbols, hover info.
/// Build this from real LSP JSON-RPC responses or from test fixtures; call
/// `render_prompt_section()` to inject into the think phase system prompt.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LspContext {
    pub diagnostics: Vec<Diagnostic>,
    pub definitions: Vec<SymbolLocation>,
    pub references: Vec<SymbolLocation>,
    pub hover: Vec<HoverInfo>,
    /// Per-language-server health: server_name → is_running
    pub server_health: HashMap<String, bool>,
}

impl LspContext {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic to the context.
    pub fn push_diagnostic(&mut self, d: Diagnostic) {
        self.diagnostics.push(d);
    }

    /// Errors only — the most urgent subset for prompt injection.
    #[must_use]
    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Error)
            .collect()
    }

    /// Warnings only.
    #[must_use]
    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == DiagnosticSeverity::Warning)
            .collect()
    }

    /// Returns true if all registered servers are healthy.
    #[must_use]
    pub fn all_servers_healthy(&self) -> bool {
        self.server_health.values().all(|ok| *ok)
    }

    /// Render the full LSP context as a structured prompt section to be injected
    /// into the system prompt before the `think` phase.
    ///
    /// Format is intentionally terse: LLMs parse structured lists better than prose.
    #[must_use]
    pub fn render_prompt_section(&self) -> String {
        if self.diagnostics.is_empty() && self.definitions.is_empty() {
            return String::new();
        }

        let mut out = String::from("## Language Server Context\n");

        let errors = self.errors();
        if !errors.is_empty() {
            out.push_str("\n### Errors\n");
            for d in &errors {
                out.push_str(&format!("- {}\n", d.render_line()));
            }
        }

        let warnings = self.warnings();
        if !warnings.is_empty() {
            out.push_str("\n### Warnings\n");
            for d in warnings.iter().take(5) {
                out.push_str(&format!("- {}\n", d.render_line()));
            }
            if warnings.len() > 5 {
                out.push_str(&format!("  (and {} more…)\n", warnings.len() - 5));
            }
        }

        if !self.definitions.is_empty() {
            out.push_str("\n### Definitions\n");
            for sym in self.definitions.iter().take(10) {
                out.push_str(&format!(
                    "- {} `{}` at {}:{}\n",
                    sym.kind, sym.name, sym.file, sym.line
                ));
            }
        }

        if !self.server_health.is_empty() {
            let unhealthy: Vec<&str> = self
                .server_health
                .iter()
                .filter(|(_, ok)| !**ok)
                .map(|(name, _)| name.as_str())
                .collect();
            if !unhealthy.is_empty() {
                out.push_str(&format!(
                    "\n> ⚠ Language servers not responding: {}\n",
                    unhealthy.join(", ")
                ));
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_error(file: &str, line: u32) -> Diagnostic {
        Diagnostic {
            file: file.to_string(),
            line,
            column: 1,
            severity: DiagnosticSeverity::Error,
            message: "type mismatch".to_string(),
            source: Some("rust-analyzer".to_string()),
            code: Some("E0308".to_string()),
        }
    }

    fn make_warning(file: &str, line: u32) -> Diagnostic {
        Diagnostic {
            file: file.to_string(),
            line,
            column: 5,
            severity: DiagnosticSeverity::Warning,
            message: "unused variable".to_string(),
            source: None,
            code: None,
        }
    }

    #[test]
    fn empty_context_renders_empty_section() {
        let ctx = LspContext::new();
        assert!(ctx.render_prompt_section().is_empty());
    }

    #[test]
    fn errors_appear_in_prompt_section() {
        let mut ctx = LspContext::new();
        ctx.push_diagnostic(make_error("src/main.rs", 42));
        let section = ctx.render_prompt_section();
        assert!(section.contains("Errors"));
        assert!(section.contains("src/main.rs:42"));
        assert!(section.contains("[rust-analyzer]"));
    }

    #[test]
    fn warnings_capped_at_five_in_prompt() {
        let mut ctx = LspContext::new();
        for i in 0..8 {
            ctx.push_diagnostic(make_warning("lib.rs", i));
        }
        let section = ctx.render_prompt_section();
        assert!(section.contains("3 more"));
    }

    #[test]
    fn severity_ordering_error_above_warning() {
        assert!(DiagnosticSeverity::Error < DiagnosticSeverity::Warning);
    }

    #[test]
    fn unhealthy_server_appears_in_section() {
        let mut ctx = LspContext::new();
        ctx.push_diagnostic(make_error("x.rs", 1));
        ctx.server_health.insert("tsserver".to_string(), false);
        let section = ctx.render_prompt_section();
        assert!(section.contains("tsserver"));
    }

    #[test]
    fn render_line_formats_correctly() {
        let d = make_error("foo.rs", 10);
        let line = d.render_line();
        assert!(line.starts_with("foo.rs:10:"));
        assert!(line.contains("ERROR"));
        assert!(line.contains("[rust-analyzer]"));
    }
}
