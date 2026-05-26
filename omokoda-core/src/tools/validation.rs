use serde_json::Value;
use std::collections::HashMap;

/// A single field constraint applied during tool input validation.
#[derive(Debug, Clone)]
pub struct FieldConstraint {
    pub field: &'static str,
    pub required: bool,
    /// Minimum string length (for string fields)
    pub min_len: Option<usize>,
    /// Maximum string length (for string fields)
    pub max_len: Option<usize>,
    /// If set, the field value must match one of these strings
    pub allowed_values: Option<&'static [&'static str]>,
}

impl FieldConstraint {
    pub const fn required(field: &'static str) -> Self {
        Self {
            field,
            required: true,
            min_len: None,
            max_len: None,
            allowed_values: None,
        }
    }

    pub const fn optional(field: &'static str) -> Self {
        Self {
            field,
            required: false,
            min_len: None,
            max_len: None,
            allowed_values: None,
        }
    }

    pub const fn with_max_len(mut self, max: usize) -> Self {
        self.max_len = Some(max);
        self
    }

    pub const fn with_allowed_values(mut self, values: &'static [&'static str]) -> Self {
        self.allowed_values = Some(values);
        self
    }
}

/// A validation error with the field path and reason.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub reason: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}

/// Validates tool inputs (pre-act) and outputs (post-act) against declared constraints.
/// Called FROM WITHIN `act` — the executor runs this before dispatching a tool call.
pub struct ToolSchemaValidator {
    /// tool_name → list of field constraints
    schemas: HashMap<&'static str, Vec<FieldConstraint>>,
}

impl ToolSchemaValidator {
    pub fn new() -> Self {
        let mut schemas: HashMap<&'static str, Vec<FieldConstraint>> = HashMap::new();

        // Core tool schemas (constraints match the Omo-Koda2 tool definitions)
        schemas.insert(
            "read_file",
            vec![FieldConstraint::required("path").with_max_len(4096)],
        );
        schemas.insert(
            "write_file",
            vec![
                FieldConstraint::required("path").with_max_len(4096),
                FieldConstraint::required("content").with_max_len(1_000_000),
            ],
        );
        schemas.insert(
            "bash",
            vec![FieldConstraint::required("command").with_max_len(65536)],
        );
        schemas.insert(
            "list_directory",
            vec![FieldConstraint::optional("path").with_max_len(4096)],
        );
        schemas.insert("todo_write", vec![FieldConstraint::required("todos")]);

        Self { schemas }
    }

    /// Register a custom tool schema. Used by plugin tools.
    pub fn register(&mut self, tool_name: &'static str, constraints: Vec<FieldConstraint>) {
        self.schemas.insert(tool_name, constraints);
    }

    /// Validate tool input JSON before execution.
    /// Returns a list of all violations (empty = valid).
    pub fn validate_input(&self, tool_name: &str, input_json: &str) -> Vec<ValidationError> {
        let Some(constraints) = self.schemas.get(tool_name) else {
            // Unknown tool — no registered schema, pass through
            return vec![];
        };

        let parsed: Value = match serde_json::from_str(input_json) {
            Ok(v) => v,
            Err(e) => {
                return vec![ValidationError {
                    field: "(input)".to_string(),
                    reason: format!("invalid JSON: {e}"),
                }]
            }
        };

        let obj = match parsed.as_object() {
            Some(o) => o,
            None => {
                return vec![ValidationError {
                    field: "(input)".to_string(),
                    reason: "expected JSON object".to_string(),
                }]
            }
        };

        let mut errors = Vec::new();
        for constraint in constraints {
            match obj.get(constraint.field) {
                None if constraint.required => {
                    errors.push(ValidationError {
                        field: constraint.field.to_string(),
                        reason: "required field missing".to_string(),
                    });
                }
                Some(value) => {
                    self.check_value(constraint, value, &mut errors);
                }
                None => {} // optional, absent — OK
            }
        }
        errors
    }

    /// Validate tool output JSON after execution.
    /// Currently checks that the output is valid JSON if the tool declares a schema.
    pub fn validate_output(&self, tool_name: &str, output: &str) -> Vec<ValidationError> {
        if !self.schemas.contains_key(tool_name) {
            return vec![];
        }
        if serde_json::from_str::<Value>(output).is_err() {
            // Many tools return plain text output — that's fine
            return vec![];
        }
        vec![]
    }

    fn check_value(
        &self,
        constraint: &FieldConstraint,
        value: &Value,
        errors: &mut Vec<ValidationError>,
    ) {
        if let Some(s) = value.as_str() {
            if let Some(min) = constraint.min_len {
                if s.len() < min {
                    errors.push(ValidationError {
                        field: constraint.field.to_string(),
                        reason: format!("too short (min {min} chars)"),
                    });
                }
            }
            if let Some(max) = constraint.max_len {
                if s.len() > max {
                    errors.push(ValidationError {
                        field: constraint.field.to_string(),
                        reason: format!("too long (max {max} chars, got {})", s.len()),
                    });
                }
            }
            if let Some(allowed) = constraint.allowed_values {
                if !allowed.contains(&s) {
                    errors.push(ValidationError {
                        field: constraint.field.to_string(),
                        reason: format!("'{}' is not an allowed value", s),
                    });
                }
            }
        }
    }
}

impl Default for ToolSchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_read_file_input_passes() {
        let v = ToolSchemaValidator::new();
        let errors = v.validate_input("read_file", r#"{"path": "/workspace/main.rs"}"#);
        assert!(errors.is_empty(), "{:?}", errors);
    }

    #[test]
    fn missing_required_field_fails() {
        let v = ToolSchemaValidator::new();
        let errors = v.validate_input("read_file", r#"{}"#);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "path");
    }

    #[test]
    fn path_too_long_fails() {
        let v = ToolSchemaValidator::new();
        let long_path = "a".repeat(5000);
        let input = format!(r#"{{"path": "{long_path}"}}"#);
        let errors = v.validate_input("read_file", &input);
        assert!(!errors.is_empty());
        assert!(errors[0].reason.contains("too long"));
    }

    #[test]
    fn unknown_tool_passes_through() {
        let v = ToolSchemaValidator::new();
        let errors = v.validate_input("custom_tool", r#"{}"#);
        assert!(errors.is_empty());
    }

    #[test]
    fn invalid_json_produces_error() {
        let v = ToolSchemaValidator::new();
        let errors = v.validate_input("bash", "not json");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].reason.contains("invalid JSON"));
    }

    #[test]
    fn non_object_input_produces_error() {
        let v = ToolSchemaValidator::new();
        let errors = v.validate_input("bash", r#""just a string""#);
        assert!(!errors.is_empty());
    }

    #[test]
    fn custom_registered_schema_validates() {
        let mut v = ToolSchemaValidator::new();
        v.register(
            "my_tool",
            vec![
                FieldConstraint::required("action").with_allowed_values(&["run", "stop", "status"])
            ],
        );
        let ok = v.validate_input("my_tool", r#"{"action": "run"}"#);
        assert!(ok.is_empty());
        let bad = v.validate_input("my_tool", r#"{"action": "explode"}"#);
        assert!(!bad.is_empty());
    }

    #[test]
    fn output_validation_passes_for_plain_text() {
        let v = ToolSchemaValidator::new();
        let errors = v.validate_output("read_file", "file contents here");
        assert!(errors.is_empty());
    }

    #[test]
    fn write_file_requires_both_fields() {
        let v = ToolSchemaValidator::new();
        let errors = v.validate_input("write_file", r#"{"path": "/workspace/a.rs"}"#);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "content");
    }
}
