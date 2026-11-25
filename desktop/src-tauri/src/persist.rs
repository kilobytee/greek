use crate::models::*;
use anyhow::{bail, Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;
use uuid::Uuid;

const SERVICE: &str = "com.greek.desktop";
const ACCOUNT: &str = "core-token";

#[cfg(target_os = "windows")]
fn platform_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|value| PathBuf::from(value).join("Greek"))
}

#[cfg(target_os = "macos")]
fn platform_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(|value| PathBuf::from(value).join("Library/Application Support/Greek"))
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn platform_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(|value| PathBuf::from(value).join("greek"))
        .or_else(|| std::env::var_os("HOME").map(|value| PathBuf::from(value).join(".local/share/greek")))
}

pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("GREEK_DATA_DIR") {
        return PathBuf::from(dir);
    }
    platform_dir().unwrap_or_else(|| std::env::temp_dir().join("greek"))
}

pub fn sandbox_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("sandbox")
}

fn new_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

#[cfg(unix)]
fn restrict_token_file(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_token_file(_path: &Path) {}

pub fn load_or_create_token(data_dir: &Path) -> String {
    if let Ok(entry) = keyring::Entry::new(SERVICE, ACCOUNT) {
        if let Ok(existing) = entry.get_password() {
            if !existing.trim().is_empty() {
                return existing;
            }
        }
        let token = new_token();
        if entry.set_password(&token).is_ok() {
            return token;
        }
        warn!("No se pudo escribir el token en el llavero del sistema, se usara un archivo local.");
    }

    let path = data_dir.join("token");
    if let Ok(existing) = fs::read_to_string(&path) {
        let token = existing.trim().to_string();
        if !token.is_empty() {
            return token;
        }
    }

    let token = new_token();
    if fs::create_dir_all(data_dir).is_ok() {
        if fs::write(&path, &token).is_ok() {
            restrict_token_file(&path);
        }
    }
    token
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistedState {
    #[serde(default)]
    pub chats: Vec<Chat>,
    #[serde(default)]
    pub active_chat_id: String,
    #[serde(default)]
    pub desktop: VirtualDesktop,
}

pub fn seed_chat() -> Chat {
    let now = Utc::now().timestamp_millis();
    Chat {
        id: Uuid::new_v4().to_string(),
        title: "Clima en Bogotá".into(),
        created_at: now,
        updated_at: now,
        model: "auto".into(),
        route_reason: "local · reglas".into(),
        tools: Vec::new(),
        messages: vec![ChatMessage {
            id: Uuid::new_v4().to_string(),
            role: Role::Assistant,
            text: "Hola. Soy Greek. Consulto el clima real con Open-Meteo, guardo archivos en el escritorio virtual y ejecuto comandos seguros en la terminal aislada. Prueba: «Busca el clima de Bogotá y guárdalo en un archivo».".into(),
            created_at: now,
            tools: Vec::new(),
        }],
    }
}

pub fn seed_desktop() -> VirtualDesktop {
    VirtualDesktop {
        windows: vec![VirtualWindow {
            id: "win-files".into(),
            kind: WindowKind::Files,
            title: "Archivos".into(),
            x: 400.0,
            y: 190.0,
            width: 440.0,
            height: 330.0,
        }],
        active_window_id: Some("win-files".into()),
        files: Vec::new(),
        selected_file: None,
        browser: None,
        terminal: Vec::new(),
    }
}

pub fn load_state(data_dir: &Path) -> Option<PersistedState> {
    let path = data_dir.join("state.json");
    let raw = fs::read_to_string(&path).ok()?;
    match serde_json::from_str::<PersistedState>(&raw) {
        Ok(state) => Some(state),
        Err(error) => {
            warn!("No se pudo leer state.json ({}). Se empieza con datos nuevos.", error);
            None
        }
    }
}

pub fn save_state(data_dir: &Path, state: &PersistedState) {
    let _ = fs::create_dir_all(data_dir);
    let path = data_dir.join("state.json");
    let temp = data_dir.join("state.json.tmp");
    let Ok(raw) = serde_json::to_string_pretty(state) else {
        warn!("No se pudo serializar el estado.");
        return;
    };
    if fs::write(&temp, raw).is_err() {
        warn!("No se pudo escribir el estado temporal.");
        return;
    }
    if fs::rename(&temp, &path).is_err() {
        let _ = fs::remove_file(&path);
        if let Err(error) = fs::rename(&temp, &path) {
            warn!("No se pudo guardar el estado: {}", error);
            let _ = fs::remove_file(&temp);
        }
    }
}

fn fold_char(ch: char) -> Option<char> {
    let lower = ch.to_ascii_lowercase();
    let folded = match lower {
        'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        _ => lower,
    };
    if folded.is_ascii_alphanumeric() {
        Some(folded)
    } else if folded == ' ' || folded == '-' || folded == '_' || folded == '.' {
        Some('-')
    } else {
        None
    }
}

pub fn slugify(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if let Some(mapped) = fold_char(ch) {
            out.push(mapped);
        }
    }
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    let out = out.trim_matches('-').to_string();
    if out.is_empty() {
        "ciudad".to_string()
    } else {
        out
    }
}

pub fn sanitize_file_name(raw: &str) -> String {
    let candidate = raw
        .trim()
        .trim_matches(|ch| ch == '"' || ch == '\'' || ch == '`')
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(raw);

    let (stem, extension) = match candidate.rsplit_once('.') {
        Some((stem, ext))
            if !stem.is_empty()
                && !ext.is_empty()
                && ext.len() <= 5
                && ext.chars().all(|ch| ch.is_ascii_alphanumeric()) =>
        {
            (stem, ext.to_ascii_lowercase())
        }
        _ => (candidate, String::new()),
    };

    let mut base = String::new();
    for ch in stem.chars() {
        if let Some(mapped) = fold_char(ch) {
            base.push(mapped);
        }
    }
    while base.contains("--") {
        base = base.replace("--", "-");
    }
    let mut base = base.trim_matches('-').to_string();
    if base.is_empty() {
        base = "nota".to_string();
    }
    if base.chars().count() > 48 {
        base = base.chars().take(48).collect::<String>();
        base = base.trim_end_matches('-').to_string();
    }

    const RESERVED: [&str; 22] = [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7",
        "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ];
    if RESERVED.contains(&base.as_str()) {
        base = format!("nota-{base}");
    }

    let extension = match extension.as_str() {
        "md" | "txt" | "json" | "log" => extension,
        _ => "md".to_string(),
    };

    format!("{base}.{extension}")
}

pub fn refresh_files(sandbox: &Path) -> Vec<DesktopFile> {
    let _ = fs::create_dir_all(sandbox);
    let Ok(entries) = fs::read_dir(sandbox) else {
        return Vec::new();
    };

    let mut files: Vec<DesktopFile> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name()?.to_string_lossy().to_string();
            let extension = path.extension()?.to_string_lossy().to_ascii_lowercase();
            if !matches!(extension.as_str(), "md" | "txt" | "json" | "log") {
                return None;
            }
            let metadata = entry.metadata().ok()?;
            let content = fs::read_to_string(&path).unwrap_or_default();
            let content: String = content.chars().take(4000).collect();
            let updated_at = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_millis() as i64)
                .unwrap_or_else(|| Utc::now().timestamp_millis());
            Some(DesktopFile {
                name,
                size: metadata.len(),
                updated_at,
                content,
            })
        })
        .collect();

    files.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    files.truncate(20);
    files
}

pub fn write_sandbox_file(sandbox: &Path, name: &str, content: &str) -> Result<DesktopFile> {
    let name = sanitize_file_name(name);
    let path = sandbox.join(&name);
    if path.parent() != Some(sandbox) {
        bail!("La ruta queda fuera del sandbox.");
    }
    fs::create_dir_all(sandbox).context("No se pudo crear el sandbox.")?;
    let content: String = content.chars().take(20_000).collect();
    fs::write(&path, content.as_bytes()).context("No se pudo escribir el archivo.")?;
    let metadata = fs::metadata(&path).context("No se pudo leer el archivo escrito.")?;
    let stored = fs::read_to_string(&path).unwrap_or_default();
    Ok(DesktopFile {
        name,
        size: metadata.len(),
        updated_at: Utc::now().timestamp_millis(),
        content: stored.chars().take(4000).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_paths_and_accents() {
        assert_eq!(sanitize_file_name("../../secreto.md"), "secreto.md");
        assert_eq!(sanitize_file_name("Clima de Bogotá.md"), "clima-de-bogota.md");
        assert_eq!(sanitize_file_name("CON"), "nota-con.md");
        assert_eq!(sanitize_file_name(""), "nota.md");
        assert_eq!(sanitize_file_name("notas.ejecutable"), "notas-ejecutable.md");
    }

    #[test]
    fn slugifies_cities() {
        assert_eq!(slugify("Bogotá"), "bogota");
        assert_eq!(slugify("Ciudad de México"), "ciudad-de-mexico");
    }
}
