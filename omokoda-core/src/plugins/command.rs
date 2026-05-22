use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDef {
    pub name: String,
    pub description: String,
    pub allowed_tools: Vec<String>,
    pub argument_hint: Option<String>,
    pub body: String,
}

impl CommandDef {
    pub fn render(&self, args: &str) -> String {
        self.body.replace("$ARGUMENTS", args)
    }

    pub fn from_markdown(content: &str) -> Result<Self, String> {
        let (frontmatter, body) = split_frontmatter(content)?;

        #[derive(Deserialize)]
        struct Header {
            name: Option<String>,
            description: Option<String>,
            allowed_tools: Option<Vec<String>>,
            argument_hint: Option<String>,
        }

        let header: Header = serde_yaml::from_str(&frontmatter)
            .map_err(|e| format!("YAML parse error: {e}"))?;

        let name = header.name.ok_or("missing required field: name")?;

        Ok(CommandDef {
            name,
            description: header.description.unwrap_or_default(),
            allowed_tools: header.allowed_tools.unwrap_or_default(),
            argument_hint: header.argument_hint,
            body: body.trim_start_matches('\n').to_string(),
        })
    }
}

fn split_frontmatter(content: &str) -> Result<(String, String), String> {
    let stripped = content.trim_start();
    if !stripped.starts_with("---") {
        return Ok((String::new(), content.to_string()));
    }
    let after_open = &stripped[3..];
    let close = after_open
        .find("\n---")
        .ok_or("unclosed frontmatter fence")?;
    let frontmatter = after_open[..close].trim().to_string();
    let body = after_open[close + 4..].to_string();
    Ok((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_frontmatter() {
        let md = r#"---
name: deploy
description: Deploy the current workspace
allowed_tools:
  - bash
  - read_file
argument_hint: "[environment]"
---
Run deployment for $ARGUMENTS.
"#;
        let cmd = CommandDef::from_markdown(md).unwrap();
        assert_eq!(cmd.name, "deploy");
        assert_eq!(cmd.description, "Deploy the current workspace");
        assert_eq!(cmd.allowed_tools, vec!["bash", "read_file"]);
        assert_eq!(cmd.argument_hint.as_deref(), Some("[environment]"));
    }

    #[test]
    fn render_substitutes_arguments() {
        let cmd = CommandDef {
            name: "greet".to_string(),
            description: String::new(),
            allowed_tools: vec![],
            argument_hint: None,
            body: "Hello, $ARGUMENTS!".to_string(),
        };
        assert_eq!(cmd.render("Omo"), "Hello, Omo!");
    }

    #[test]
    fn missing_name_errors() {
        let md = r#"---
description: No name here
---
body
"#;
        let result = CommandDef::from_markdown(md);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name"));
    }
}
