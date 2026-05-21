use crate::tools::tool_definitions::ToolCall;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TokenEvent {
    Token(String),
    ToolUseStart { id: String, name: String },
    ToolUseInput { id: String, input: String },
    ToolUseEnd { id: String },
    Done,
}

pub trait StreamingProvider {
    fn stream(&self, prompt: &str) -> Box<dyn Iterator<Item = TokenEvent>>;
}

/// A token-level streaming event — emitted one per SSE chunk from the LLM.
/// Adapts Claw-code's SSE streaming pattern to Omo-Koda2's agentic think loop.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    /// Partial text delta from the model
    TextDelta { text: String },
    /// The model is calling a tool
    ToolUse { call: ToolCall },
    /// All tokens received; final usage snapshot
    Done {
        input_tokens: u32,
        output_tokens: u32,
    },
}

/// Sender half of a streaming session — `LlmProvider::stream_with_tools` drives this.
pub type StreamSender = mpsc::Sender<StreamEvent>;
/// Receiver half — callers iterate with `.recv().await`.
pub type StreamReceiver = mpsc::Receiver<StreamEvent>;

/// Create a bounded channel for streaming events (default capacity 128).
pub fn stream_channel() -> (StreamSender, StreamReceiver) {
    mpsc::channel(128)
}

/// Parse an SSE chunk (one or more `data: ...` lines) into `TokenEvent`s.
pub fn parse_sse(chunk: &str) -> Vec<TokenEvent> {
    let mut events = Vec::new();
    for line in chunk.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            let data = data.trim();
            if data == "[DONE]" {
                events.push(TokenEvent::Done);
                continue;
            }

            if let Ok(event) = serde_json::from_str::<TokenEvent>(data) {
                events.push(event);
            } else if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                // Handle various provider formats
                if let Some(text) = json["choices"][0]["delta"]["content"].as_str() {
                    events.push(TokenEvent::Token(text.to_string()));
                } else if let Some(text) = json["completion"].as_str() {
                    events.push(TokenEvent::Token(text.to_string()));
                } else if let Some(text) = json["message"]["content"].as_str() {
                    events.push(TokenEvent::Token(text.to_string()));
                } else if let Some(text) = json["response"].as_str() {
                    events.push(TokenEvent::Token(text.to_string()));
                }
            }
        }
    }
    events
}

/// Collect all `StreamEvent`s from a receiver into a single assembled string,
/// returning the text and accumulated token counts.
/// Useful for tests that want the full response without caring about streaming.
pub async fn collect_stream(mut rx: StreamReceiver) -> (String, u32, u32) {
    let mut text = String::new();
    let mut input_tokens = 0u32;
    let mut output_tokens = 0u32;

    while let Some(event) = rx.recv().await {
        match event {
            StreamEvent::TextDelta { text: delta } => text.push_str(&delta),
            StreamEvent::Done { input_tokens: i, output_tokens: o } => {
                input_tokens = i;
                output_tokens = o;
            }
            StreamEvent::ToolUse { .. } => {}
        }
    }
    (text, input_tokens, output_tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sse_done_sentinel() {
        let events = parse_sse("data: [DONE]\n");
        assert_eq!(events, vec![TokenEvent::Done]);
    }

    #[test]
    fn parse_sse_openai_delta() {
        let chunk = r#"data: {"choices":[{"delta":{"content":"hello"}}]}"#;
        let events = parse_sse(chunk);
        assert_eq!(events, vec![TokenEvent::Token("hello".to_string())]);
    }

    #[test]
    fn parse_sse_ollama_response() {
        let chunk = r#"data: {"response":"world"}"#;
        let events = parse_sse(chunk);
        assert_eq!(events, vec![TokenEvent::Token("world".to_string())]);
    }

    #[test]
    fn stream_channel_creates_bounded_channel() {
        let (tx, _rx) = stream_channel();
        // channel capacity 128: first 128 sends should succeed synchronously
        for i in 0..128u32 {
            tx.try_send(StreamEvent::TextDelta {
                text: format!("chunk-{}", i),
            })
            .unwrap();
        }
        // 129th should be full
        assert!(tx
            .try_send(StreamEvent::Done {
                input_tokens: 0,
                output_tokens: 0,
            })
            .is_err());
    }

    #[tokio::test]
    async fn collect_stream_assembles_text() {
        let (tx, rx) = stream_channel();
        tx.send(StreamEvent::TextDelta { text: "foo".to_string() }).await.unwrap();
        tx.send(StreamEvent::TextDelta { text: "bar".to_string() }).await.unwrap();
        tx.send(StreamEvent::Done { input_tokens: 10, output_tokens: 6 }).await.unwrap();
        drop(tx);

        let (text, inp, out) = collect_stream(rx).await;
        assert_eq!(text, "foobar");
        assert_eq!(inp, 10);
        assert_eq!(out, 6);
    }

    #[tokio::test]
    async fn collect_stream_ignores_tool_use_events() {
        let (tx, rx) = stream_channel();
        tx.send(StreamEvent::ToolUse {
            call: ToolCall {
                id: "c1".to_string(),
                name: "bash".to_string(),
                input: "{}".to_string(),
            },
        })
        .await
        .unwrap();
        tx.send(StreamEvent::TextDelta { text: "done".to_string() }).await.unwrap();
        drop(tx);

        let (text, _, _) = collect_stream(rx).await;
        assert_eq!(text, "done");
    }
}
