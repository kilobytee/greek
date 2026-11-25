use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowKind {
    Browser,
    Files,
    Terminal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolChip {
    pub label: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub role: Role,
    pub text: String,
    pub created_at: i64,
    #[serde(default)]
    pub tools: Vec<ToolChip>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chat {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub model: String,
    pub route_reason: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopFile {
    pub name: String,
    pub size: u64,
    pub updated_at: i64,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BrowserState {
    pub title: String,
    pub url: String,
    pub status: u16,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualWindow {
    pub id: String,
    pub kind: WindowKind,
    pub title: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VirtualDesktop {
    #[serde(default)]
    pub windows: Vec<VirtualWindow>,
    pub active_window_id: Option<String>,
    #[serde(default)]
    pub files: Vec<DesktopFile>,
    pub selected_file: Option<String>,
    pub browser: Option<BrowserState>,
    #[serde(default)]
    pub terminal: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelOption {
    pub id: String,
    pub label: String,
    pub provider: String,
    pub available: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub online: bool,
    pub bind: String,
    pub lan_url: String,
    pub version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub schema_version: u32,
    pub chats: Vec<Chat>,
    pub active_chat_id: String,
    pub thinking_chat_id: Option<String>,
    pub desktop: VirtualDesktop,
    pub models: Vec<ModelOption>,
    pub server: ServerInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInfo {
    pub url: String,
    pub lan_url: String,
    pub token: String,
    pub port: u16,
    pub bind: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ClientCommand {
    NewChat,
    SelectChat {
        chat_id: String,
    },
    SendMessage {
        text: String,
        #[serde(default)]
        model: Option<String>,
    },
    SetModel {
        model: String,
    },
    DesktopOpen {
        app: String,
    },
    DesktopClose {
        window_id: String,
    },
    DesktopFocus {
        window_id: String,
    },
    DesktopSelectFile {
        file: String,
    },
    Ping,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerEvent {
    Snapshot { snapshot: Snapshot },
    Error { message: String },
    Pong,
}
