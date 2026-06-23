use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_input::Input;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Normal,
    SlashMenu,
    Help,
}

pub struct App {
    pub state: AppState,
    pub should_quit: bool,
    pub input: Input,
    pub chat_messages: Vec<ChatMessage>,
    pub chat_scroll: usize,
    pub slash_filter: String,
    pub slash_selected: usize,
    pub left_panel_visible: bool,
    pub right_panel_visible: bool,
    pub agent_name: String,
    pub tier: u8,
    pub session_id: String,
    pub token_used: u32,
    pub token_max: u32,
    pub tick_count: u64,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Agent,
    System,
    #[allow(dead_code)]
    Tool,
}

pub const SLASH_COMMANDS: &[(&str, &str)] = &[
    ("/clear", "Clear chat view"),
    ("/compact", "Force context compaction"),
    ("/config", "Show or set config"),
    ("/export", "Export session to file"),
    ("/help", "Show all commands"),
    ("/history", "Browse session history"),
    ("/memory", "Show memory entries"),
    ("/model", "Switch model or provider"),
    ("/private", "Toggle privacy mode"),
    ("/publish", "Publish to garden"),
    ("/receipts", "Browse receipts"),
    ("/sandbox", "Toggle sandbox mode"),
    ("/seal", "Encrypt private memory"),
    ("/sessions", "Open session browser"),
    ("/skills", "Browse skills"),
    ("/status", "Show agent status"),
    ("/think", "Toggle extended thinking"),
    ("/tools", "Tool catalog"),
    ("/transfer", "Transfer ownership"),
    ("/unlock", "Decrypt private memory"),
];

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Normal,
            should_quit: false,
            input: Input::default(),
            chat_messages: vec![ChatMessage {
                role: MessageRole::System,
                content: "Omo-Koda TUI ready. Type a message or / for commands. Press ? for help."
                    .to_string(),
            }],
            chat_scroll: 0,
            slash_filter: String::new(),
            slash_selected: 0,
            left_panel_visible: true,
            right_panel_visible: true,
            agent_name: "Agent".to_string(),
            tier: 1,
            session_id: "sess-0001".to_string(),
            token_used: 0,
            token_max: 2000,
            tick_count: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.state {
            AppState::SlashMenu => self.handle_slash_menu_key(key),
            AppState::Help => self.handle_help_key(key),
            AppState::Normal => self.handle_normal_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.should_quit = true;
                return true;
            }
            (KeyModifiers::NONE, KeyCode::Char('q')) => {
                if self.input.value().is_empty() {
                    self.should_quit = true;
                    return true;
                }
                self.input.handle(tui_input::InputRequest::InsertChar('q'));
            }
            (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                self.chat_messages.clear();
            }
            (KeyModifiers::NONE, KeyCode::F(1)) => {
                self.left_panel_visible = !self.left_panel_visible;
            }
            (KeyModifiers::NONE, KeyCode::F(2)) => {
                self.right_panel_visible = !self.right_panel_visible;
            }
            (KeyModifiers::NONE, KeyCode::Char('?')) => {
                if self.input.value().is_empty() {
                    self.state = AppState::Help;
                } else {
                    self.input.handle(tui_input::InputRequest::InsertChar('?'));
                }
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                let text = self.input.value().to_string();
                if !text.is_empty() {
                    self.submit_message(text);
                    self.input = Input::default();
                }
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.input.handle(tui_input::InputRequest::DeletePrevChar);
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                self.chat_scroll = self.chat_scroll.saturating_sub(1);
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                self.chat_scroll += 1;
            }
            _ => {
                if let KeyCode::Char(c) = key.code {
                    self.input.handle(tui_input::InputRequest::InsertChar(c));
                    let val = self.input.value().to_string();
                    if val == "/" || (val.starts_with('/') && self.state != AppState::SlashMenu) {
                        self.state = AppState::SlashMenu;
                        self.slash_filter = if val.len() > 1 {
                            val[1..].to_string()
                        } else {
                            String::new()
                        };
                        self.slash_selected = 0;
                    } else if val.starts_with('/') && self.state == AppState::SlashMenu {
                        self.slash_filter = val[1..].to_string();
                        self.slash_selected = 0;
                    }
                }
            }
        }
        false
    }

    fn handle_slash_menu_key(&mut self, key: KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc) => {
                self.state = AppState::Normal;
                self.input = Input::default();
                self.slash_filter.clear();
            }
            (KeyModifiers::NONE, KeyCode::Up) => {
                self.slash_selected = self.slash_selected.saturating_sub(1);
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                let count = self.filtered_commands().len();
                if count > 0 && self.slash_selected + 1 < count {
                    self.slash_selected += 1;
                }
            }
            (KeyModifiers::NONE, KeyCode::Tab) => {
                let count = self.filtered_commands().len();
                if count > 0 {
                    self.slash_selected = (self.slash_selected + 1) % count;
                }
            }
            (KeyModifiers::NONE, KeyCode::Enter) => {
                let cmds = self.filtered_commands();
                if let Some(&(cmd, _)) = cmds.get(self.slash_selected) {
                    let text = cmd.to_string();
                    self.submit_message(text);
                    self.input = Input::default();
                    self.slash_filter.clear();
                    self.state = AppState::Normal;
                }
            }
            (KeyModifiers::NONE, KeyCode::Backspace) => {
                self.input.handle(tui_input::InputRequest::DeletePrevChar);
                let val = self.input.value().to_string();
                if val.is_empty() || !val.starts_with('/') {
                    self.state = AppState::Normal;
                    self.slash_filter.clear();
                } else {
                    self.slash_filter = val[1..].to_string();
                    self.slash_selected = 0;
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.should_quit = true;
                return true;
            }
            _ => {
                if let KeyCode::Char(c) = key.code {
                    self.input.handle(tui_input::InputRequest::InsertChar(c));
                    let val = self.input.value().to_string();
                    if let Some(stripped) = val.strip_prefix('/') {
                        self.slash_filter = stripped.to_string();
                        self.slash_selected = 0;
                    }
                }
            }
        }
        false
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> bool {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc)
            | (KeyModifiers::NONE, KeyCode::Char('q'))
            | (KeyModifiers::NONE, KeyCode::Char('?')) => {
                self.state = AppState::Normal;
            }
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                self.should_quit = true;
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn filtered_commands(&self) -> Vec<&(&'static str, &'static str)> {
        if self.slash_filter.is_empty() {
            SLASH_COMMANDS.iter().collect()
        } else {
            let filter = self.slash_filter.to_lowercase();
            SLASH_COMMANDS
                .iter()
                .filter(|(cmd, desc)| {
                    cmd.contains(filter.as_str()) || desc.to_lowercase().contains(filter.as_str())
                })
                .collect()
        }
    }

    fn submit_message(&mut self, text: String) {
        self.token_used = (self.token_used + text.len() as u32 / 4).min(self.token_max);
        self.chat_messages.push(ChatMessage {
            role: MessageRole::User,
            content: text.clone(),
        });
        self.chat_messages.push(ChatMessage {
            role: MessageRole::Agent,
            content: format!("[stub] Received: {text}"),
        });
        self.chat_scroll = self.chat_messages.len().saturating_sub(1);
    }

    pub fn on_tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn token_pct(&self) -> u16 {
        if self.token_max == 0 {
            return 0;
        }
        ((self.token_used as f64 / self.token_max as f64) * 100.0) as u16
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
