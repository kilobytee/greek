use crate::models::*;
use crate::persist::{self, PersistedState};
use anyhow::Result;
use chrono::Utc;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::broadcast;
use tracing::warn;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Providers {
    pub openrouter_key: Option<String>,
    pub openrouter_model: Option<String>,
    pub ollama_model: Option<String>,
    pub ollama_available: bool,
    pub opencode_key: Option<String>,
    pub opencode_model: Option<String>,
    pub opencode_base: String,
}

impl Default for Providers {
    fn default() -> Self {
        Self {
            openrouter_key: None,
            openrouter_model: None,
            ollama_model: None,
            ollama_available: false,
            opencode_key: None,
            opencode_model: None,
            opencode_base: "https://opencode.ai/zen/v1".to_string(),
        }
    }
}

impl Providers {
    pub fn openrouter(&self) -> Option<(String, String)> {
        match (&self.openrouter_key, &self.openrouter_model) {
            (Some(key), Some(model)) if !key.is_empty() && !model.is_empty() => {
                Some((key.clone(), model.clone()))
            }
            _ => None,
        }
    }

    pub fn opencode_zen(&self) -> Option<(String, String, String)> {
        match (&self.opencode_key, &self.opencode_model) {
            (Some(key), Some(model)) if !key.is_empty() && !model.is_empty() => {
                Some((key.clone(), model.clone(), self.opencode_base.clone()))
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProviderChoice {
    Local,
    Ollama { model: String },
    OpenRouter { key: String, model: String },
    OpenCodeZen { key: String, model: String, base: String },
}

pub struct Inner {
    pub chats: Vec<Chat>,
    pub active_chat_id: String,
    pub thinking: Option<String>,
    pub desktop: VirtualDesktop,
    pub models: Vec<ModelOption>,
    pub server: ServerInfo,
}

pub struct Core {
    pub info: ConnectionInfo,
    pub token: String,
    pub inner: Mutex<Inner>,
    pub tx: broadcast::Sender<ServerEvent>,
    pub sandbox: PathBuf,
    pub data_dir: PathBuf,
    pub http: reqwest::Client,
    pub providers: Providers,
}

fn lock(inner: &Mutex<Inner>) -> MutexGuard<'_, Inner> {
    inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn title_from(text: &str) -> String {
    let one_line = text.replace(['\n', '\r'], " ");
    let compact = one_line.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.is_empty() {
        return "Nuevo chat".to_string();
    }
    let mut title: String = compact.chars().take(48).collect();
    if compact.chars().count() > 48 {
        title = title.trim_end().to_string();
        title.push('…');
    }
    title
}

fn build_models(providers: &Providers) -> Vec<ModelOption> {
    let ollama_detail = if providers.ollama_available {
        providers
            .ollama_model
            .clone()
            .unwrap_or_else(|| "modelo detectado".to_string())
    } else {
        "no detectado en el puerto 11434".to_string()
    };

    let openrouter_available = providers.openrouter().is_some();
    let openrouter_detail = if openrouter_available {
        providers
            .openrouter_model
            .clone()
            .unwrap_or_else(|| "configurado".to_string())
    } else if providers.openrouter_key.is_some() {
        "define GREEK_OPENROUTER_MODEL".to_string()
    } else {
        "define OPENROUTER_API_KEY".to_string()
    };

    let zen_available = providers.opencode_zen().is_some();
    let zen_detail = if zen_available {
        providers
            .opencode_model
            .clone()
            .unwrap_or_else(|| "configurado".to_string())
    } else if providers.opencode_key.is_some() {
        "define OPENCODE_ZEN_MODEL".to_string()
    } else {
        "define OPENCODE_ZEN_API_KEY".to_string()
    };

    vec![
        ModelOption {
            id: "auto".into(),
            label: "Auto".into(),
            provider: "Greek".into(),
            available: true,
            detail: "elige el proveedor disponible".into(),
        },
        ModelOption {
            id: "greek-local".into(),
            label: "Greek Local".into(),
            provider: "En este equipo".into(),
            available: true,
            detail: "reglas y herramientas".into(),
        },
        ModelOption {
            id: "ollama".into(),
            label: "Ollama".into(),
            provider: "Ollama".into(),
            available: providers.ollama_available,
            detail: ollama_detail,
        },
        ModelOption {
            id: "openrouter".into(),
            label: "OpenRouter".into(),
            provider: "OpenRouter".into(),
            available: openrouter_available,
            detail: openrouter_detail,
        },
        ModelOption {
            id: "opencode-zen".into(),
            label: "OpenCode Zen".into(),
            provider: "OpenCode".into(),
            available: zen_available,
            detail: zen_detail,
        },
    ]
}

impl Core {
    pub fn new(
        token: String,
        info: ConnectionInfo,
        data_dir: PathBuf,
        providers: Providers,
        http: reqwest::Client,
    ) -> Arc<Self> {
        let sandbox = persist::sandbox_dir(&data_dir);
        let _ = std::fs::create_dir_all(&sandbox);

        let loaded = persist::load_state(&data_dir);
        let (mut chats, mut desktop, mut active_chat_id) = match loaded {
            Some(state) => (state.chats, state.desktop, state.active_chat_id),
            None => {
                let chat = persist::seed_chat();
                let id = chat.id.clone();
                (vec![chat], persist::seed_desktop(), id)
            }
        };

        if chats.is_empty() {
            let chat = persist::seed_chat();
            active_chat_id = chat.id.clone();
            chats.push(chat);
        }
        if !chats.iter().any(|chat| chat.id == active_chat_id) {
            active_chat_id = chats[0].id.clone();
        }

        desktop.files = persist::refresh_files(&sandbox);
        let selected_is_valid = desktop
            .selected_file
            .as_ref()
            .map(|name| desktop.files.iter().any(|file| &file.name == name))
            .unwrap_or(false);
        if !selected_is_valid {
            desktop.selected_file = desktop.files.first().map(|file| file.name.clone());
        }
        let active_window_is_valid = desktop
            .active_window_id
            .as_ref()
            .map(|id| desktop.windows.iter().any(|window| &window.id == id))
            .unwrap_or(false);
        if !active_window_is_valid {
            desktop.active_window_id = desktop.windows.last().map(|window| window.id.clone());
        }

        let server = ServerInfo {
            online: !info.bind.starts_with("127.")
                && !info.bind.starts_with("localhost")
                && !info.bind.starts_with("[::1]"),
            bind: info.bind.clone(),
            lan_url: info.lan_url.clone(),
            version: crate::VERSION.to_string(),
        };

        let models = build_models(&providers);
        let (tx, _) = broadcast::channel(64);

        let core = Arc::new(Self {
            info,
            token,
            inner: Mutex::new(Inner {
                chats,
                active_chat_id,
                thinking: None,
                desktop,
                models,
                server,
            }),
            tx,
            sandbox,
            data_dir,
            http,
            providers,
        });

        core.persist();
        core
    }

    pub fn snapshot(&self) -> Snapshot {
        let inner = lock(&self.inner);
        Snapshot {
            schema_version: SCHEMA_VERSION,
            chats: inner.chats.clone(),
            active_chat_id: inner.active_chat_id.clone(),
            thinking_chat_id: inner.thinking.clone(),
            desktop: inner.desktop.clone(),
            models: inner.models.clone(),
            server: inner.server.clone(),
        }
    }

    pub fn broadcast(&self) {
        let _ = self.tx.send(ServerEvent::Snapshot {
            snapshot: self.snapshot(),
        });
    }

    pub fn persist(&self) {
        let (chats, active_chat_id, desktop) = {
            let inner = lock(&self.inner);
            (
                inner.chats.clone(),
                inner.active_chat_id.clone(),
                inner.desktop.clone(),
            )
        };
        persist::save_state(
            &self.data_dir,
            &PersistedState {
                chats,
                active_chat_id,
                desktop,
            },
        );
    }

    pub async fn handle_command(self: &Arc<Self>, command: ClientCommand) -> Option<String> {
        match command {
            ClientCommand::Ping => None,
            ClientCommand::NewChat => {
                let now = Utc::now().timestamp_millis();
                let chat = Chat {
                    id: Uuid::new_v4().to_string(),
                    title: "Nuevo chat".into(),
                    created_at: now,
                    updated_at: now,
                    model: "auto".into(),
                    route_reason: "esperando mensaje".into(),
                    tools: Vec::new(),
                    messages: Vec::new(),
                };
                {
                    let mut inner = lock(&self.inner);
                    inner.active_chat_id = chat.id.clone();
                    inner.chats.insert(0, chat);
                }
                self.persist();
                self.broadcast();
                None
            }
            ClientCommand::SelectChat { chat_id } => {
                {
                    let mut inner = lock(&self.inner);
                    if !inner.chats.iter().any(|chat| chat.id == chat_id) {
                        return Some("Ese chat ya no existe.".into());
                    }
                    inner.active_chat_id = chat_id;
                }
                self.persist();
                self.broadcast();
                None
            }
            ClientCommand::SendMessage { text, model } => {
                let text = text.trim().to_string();
                if text.is_empty() {
                    return Some("El mensaje está vacío.".into());
                }
                if text.chars().count() > 8000 {
                    return Some("El mensaje supera los 8000 caracteres.".into());
                }

                let now = Utc::now().timestamp_millis();
                let chat_id = {
                    let mut inner = lock(&self.inner);
                    let active = inner.active_chat_id.clone();
                    let Some(index) = inner.chats.iter().position(|chat| chat.id == active) else {
                        return Some("No hay un chat activo.".into());
                    };
                    if inner.thinking.as_deref() == Some(active.as_str()) {
                        return Some("Greek todavía está respondiendo en este chat.".into());
                    }

                    let chat = &mut inner.chats[index];
                    if let Some(model) = model {
                        let model = model.trim();
                        if !model.is_empty() {
                            chat.model = model.to_string();
                        }
                    }
                    let is_first = chat.messages.is_empty();
                    chat.messages.push(ChatMessage {
                        id: Uuid::new_v4().to_string(),
                        role: Role::User,
                        text: text.clone(),
                        created_at: now,
                        tools: Vec::new(),
                    });
                    if is_first || chat.title == "Nuevo chat" {
                        chat.title = title_from(&text);
                    }
                    chat.updated_at = now;
                    chat.route_reason = "pensando".into();

                    let moved = inner.chats.remove(index);
                    inner.chats.insert(0, moved);
                    inner.thinking = Some(active.clone());
                    active
                };

                self.persist();
                self.broadcast();

                let core = Arc::clone(self);
                tokio::spawn(async move {
                    crate::agent::respond(core, chat_id, text).await;
                });
                None
            }
            ClientCommand::SetModel { model } => {
                {
                    let mut inner = lock(&self.inner);
                    let Some(option) = inner.models.iter().find(|option| option.id == model) else {
                        return Some("Ese modelo no existe.".into());
                    };
                    if !option.available {
                        return Some("Ese modelo no está disponible en este equipo.".into());
                    }
                    let active = inner.active_chat_id.clone();
                    let Some(chat) = inner.chats.iter_mut().find(|chat| chat.id == active) else {
                        return Some("No hay un chat activo.".into());
                    };
                    chat.model = model;
                    chat.route_reason = "modelo elegido".into();
                }
                self.persist();
                self.broadcast();
                None
            }
            ClientCommand::DesktopOpen { app } => {
                let kind = match app.as_str() {
                    "browser" => WindowKind::Browser,
                    "files" => WindowKind::Files,
                    _ => return Some("Aplicación desconocida.".into()),
                };
                self.open_app(kind);
                self.persist();
                self.broadcast();
                None
            }
            ClientCommand::DesktopClose { window_id } => {
                {
                    let mut inner = lock(&self.inner);
                    inner.desktop.windows.retain(|window| window.id != window_id);
                    if inner.desktop.active_window_id.as_deref() == Some(window_id.as_str()) {
                        inner.desktop.active_window_id =
                            inner.desktop.windows.last().map(|window| window.id.clone());
                    }
                }
                self.persist();
                self.broadcast();
                None
            }
            ClientCommand::DesktopFocus { window_id } => {
                {
                    let mut inner = lock(&self.inner);
                    let Some(index) = inner
                        .desktop
                        .windows
                        .iter()
                        .position(|window| window.id == window_id)
                    else {
                        return Some("Esa ventana ya no existe.".into());
                    };
                    let window = inner.desktop.windows.remove(index);
                    inner.desktop.windows.push(window);
                    inner.desktop.active_window_id = Some(window_id);
                }
                self.broadcast();
                None
            }
            ClientCommand::DesktopSelectFile { file } => {
                {
                    let mut inner = lock(&self.inner);
                    if !inner.desktop.files.iter().any(|entry| entry.name == file) {
                        return Some("Ese archivo no existe en el sandbox.".into());
                    }
                    inner.desktop.selected_file = Some(file);
                }
                self.persist();
                self.broadcast();
                None
            }
        }
    }

    pub fn open_app(&self, kind: WindowKind) {
        let mut inner = lock(&self.inner);
        let id = match kind {
            WindowKind::Browser => "win-browser",
            WindowKind::Files => "win-files",
            WindowKind::Terminal => "win-terminal",
        };

        if let Some(index) = inner.desktop.windows.iter().position(|window| window.id == id) {
            let window = inner.desktop.windows.remove(index);
            inner.desktop.windows.push(window);
        } else {
            let (title, x, y, width, height) = match kind {
                WindowKind::Browser => ("Navegador", 60.0, 50.0, 520.0, 340.0),
                WindowKind::Files => ("Archivos", 400.0, 190.0, 440.0, 330.0),
                WindowKind::Terminal => ("Terminal", 130.0, 380.0, 560.0, 190.0),
            };
            inner.desktop.windows.push(VirtualWindow {
                id: id.to_string(),
                kind,
                title: title.to_string(),
                x,
                y,
                width,
                height,
            });
        }

        inner.desktop.active_window_id = Some(id.to_string());
        if kind == WindowKind::Files && inner.desktop.selected_file.is_none() {
            inner.desktop.selected_file =
                inner.desktop.files.first().map(|file| file.name.clone());
        }
    }

    pub fn write_file(&self, name: &str, content: &str) -> Result<DesktopFile> {
        let file = persist::write_sandbox_file(&self.sandbox, name, content)?;
        {
            let mut inner = lock(&self.inner);
            inner.desktop.files.retain(|entry| entry.name != file.name);
            inner.desktop.files.insert(0, file.clone());
            inner.desktop.files.truncate(20);
            inner.desktop.selected_file = Some(file.name.clone());
        }
        self.open_app(WindowKind::Files);
        Ok(file)
    }

    pub fn set_browser(&self, browser: BrowserState) {
        {
            let mut inner = lock(&self.inner);
            inner.desktop.browser = Some(browser);
        }
        self.open_app(WindowKind::Browser);
    }

    pub fn log_terminal(&self, lines: Vec<String>) {
        {
            let mut inner = lock(&self.inner);
            for line in lines {
                inner.desktop.terminal.push(line);
            }
            let excess = inner.desktop.terminal.len().saturating_sub(200);
            if excess > 0 {
                inner.desktop.terminal.drain(0..excess);
            }
        }
        self.open_app(WindowKind::Terminal);
    }

    pub fn finish_agent(
        &self,
        chat_id: &str,
        text: String,
        chips: Vec<ToolChip>,
        route_reason: String,
    ) {
        let now = Utc::now().timestamp_millis();
        {
            let mut inner = lock(&self.inner);
            if let Some(chat) = inner.chats.iter_mut().find(|chat| chat.id == chat_id) {
                let mut labels: Vec<String> = Vec::new();
                for chip in &chips {
                    if !labels.contains(&chip.label) {
                        labels.push(chip.label.clone());
                    }
                }
                chat.messages.push(ChatMessage {
                    id: Uuid::new_v4().to_string(),
                    role: Role::Assistant,
                    text,
                    created_at: now,
                    tools: chips,
                });
                chat.updated_at = now;
                chat.route_reason = route_reason;
                chat.tools = labels;
            } else {
                warn!("El chat {} desapareció antes de la respuesta.", chat_id);
            }
            if inner.thinking.as_deref() == Some(chat_id) {
                inner.thinking = None;
            }
        }
        self.persist();
        self.broadcast();
    }

    pub fn chat_model(&self, chat_id: &str) -> String {
        let inner = lock(&self.inner);
        inner
            .chats
            .iter()
            .find(|chat| chat.id == chat_id)
            .map(|chat| chat.model.clone())
            .unwrap_or_else(|| "auto".to_string())
    }

    pub fn history(&self, chat_id: &str, limit: usize) -> Vec<(String, String)> {
        let inner = lock(&self.inner);
        let Some(chat) = inner.chats.iter().find(|chat| chat.id == chat_id) else {
            return Vec::new();
        };
        let start = chat.messages.len().saturating_sub(limit);
        chat.messages[start..]
            .iter()
            .map(|message| {
                let role = match message.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                };
                (role.to_string(), message.text.clone())
            })
            .collect()
    }

    pub fn resolve_model(&self, requested: &str) -> (ProviderChoice, String) {
        let openrouter = self.providers.openrouter();
        let zen = self.providers.opencode_zen();
        let ollama = if self.providers.ollama_available {
            self.providers.ollama_model.clone()
        } else {
            None
        };

        match requested {
            "greek-local" => (ProviderChoice::Local, "local · reglas".into()),
            "openrouter" => match openrouter {
                Some((key, model)) => (ProviderChoice::OpenRouter { key, model }, "openrouter".into()),
                None => (ProviderChoice::Local, "local · respaldo".into()),
            },
            "opencode-zen" => match zen {
                Some((key, model, base)) => (
                    ProviderChoice::OpenCodeZen { key, model, base },
                    "opencode zen".into(),
                ),
                None => (ProviderChoice::Local, "local · respaldo".into()),
            },
            "ollama" => match ollama {
                Some(model) => (ProviderChoice::Ollama { model }, "ollama".into()),
                None => (ProviderChoice::Local, "local · respaldo".into()),
            },
            _ => {
                if let Some((key, model)) = openrouter {
                    (ProviderChoice::OpenRouter { key, model }, "auto → openrouter".into())
                } else if let Some((key, model, base)) = zen {
                    (
                        ProviderChoice::OpenCodeZen { key, model, base },
                        "auto → opencode zen".into(),
                    )
                } else if let Some(model) = ollama {
                    (ProviderChoice::Ollama { model }, "auto → ollama".into())
                } else {
                    (ProviderChoice::Local, "auto → local".into())
                }
            }
        }
    }
}
