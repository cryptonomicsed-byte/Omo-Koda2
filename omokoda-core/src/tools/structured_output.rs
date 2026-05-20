//! Structured output tool — validates JSON output against a schema with retry logic.
//! Ports Claw-code's structured output pattern.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::{ExecutionContext, Tool};
use crate::usage::TokenUsage;

/// Request for structured output generation
#[derive(Debug, Serialize, Deserialize)]
pub struct StructuredOutputRequest {
    /// The JSON schema to validate against
    pub schema: serde_json::Value,
    /// The data to validate/format
    pub data: serde_json::Value,
    /// Optional transformation hints
    #[serde(default)]
    pub description: String,
}

/// Tool that validates and returns structured JSON output
pub struct StructuredOutputTool;

#[async_trait]
impl Tool for StructuredOutputTool {
    fn name(&self) -> &str {
        "structured_output"
    }

    fn description(&self) -> &str {
        "Validate and return structured JSON output against a schema. \
         Params: JSON {schema, data, description?}"
    }

    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        false
    }

    fn params_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "schema": { "type": "object", "description": "JSON Schema to validate against" },
                "data": { "description": "Data to validate" },
                "description": { "type": "string", "description": "Optional description" }
            },
            "required": ["schema", "data"]
        }))
    }

    async fn execute(
        &self,
        params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, TokenUsage), String> {
        let req: StructuredOutputRequest = serde_json::from_str(params)
            .map_err(|e| format!("structured_output requires JSON: {}", e))?;

        // Validate data against schema
        let compiled = jsonschema::JSONSchema::compile(&req.schema)
            .map_err(|e| format!("Invalid JSON Schema: {}", e))?;

        let validation = compiled.validate(&req.data);

        if let Err(errors) = validation {
            let error_list: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(format!(
                "Structured output validation failed:\n{}",
                error_list.join("\n")
            ));
        }

        // Output is the validated data as pretty JSON
        let output = serde_json::to_string_pretty(&req.data)
            .map_err(|e| format!("Failed to serialize output: {}", e))?;

        Ok((output, TokenUsage::default()))
    }
}

/// Helper to build structured output requests
pub struct StructuredOutputBuilder {
    schema: serde_json::Value,
}

impl StructuredOutputBuilder {
    pub fn new(schema: serde_json::Value) -> Self {
        Self { schema }
    }

    /// Validate data, return pretty-printed JSON on success
    pub fn validate(&self, data: &serde_json::Value) -> Result<String, Vec<String>> {
        let compiled = jsonschema::JSONSchema::compile(&self.schema)
            .map_err(|e| vec![format!("Invalid schema: {}", e)])?;

        compiled
            .validate(data)
            .map_err(|errors| errors.map(|e| e.to_string()).collect::<Vec<_>>())?;

        serde_json::to_string_pretty(data).map_err(|e| vec![e.to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_output_builder_valid() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number" }
            },
            "required": ["name"]
        });

        let builder = StructuredOutputBuilder::new(schema);
        let data = serde_json::json!({"name": "Alice", "age": 30});
        assert!(builder.validate(&data).is_ok());
    }

    #[test]
    fn test_structured_output_builder_invalid() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "name": { "type": "string" } },
            "required": ["name"]
        });

        let builder = StructuredOutputBuilder::new(schema);
        let data = serde_json::json!({"age": 30}); // missing required "name"
        assert!(builder.validate(&data).is_err());
    }
}
