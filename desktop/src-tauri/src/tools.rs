use crate::persist::sanitize_file_name;
use anyhow::{Context, Result};
use chrono::Local;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

const WEATHER_TIMEOUT: Duration = Duration::from_secs(9);

#[derive(Clone, Debug)]
pub struct WeatherReport {
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub temp_c: f64,
    pub apparent_c: f64,
    pub humidity: u64,
    pub wind_kmh: f64,
    pub code: u64,
    pub description: &'static str,
    pub local_time: String,
}

pub fn es_number(value: f64, decimals: usize) -> String {
    format!("{value:.decimals$}").replace('.', ",")
}

pub fn weather_description(code: u64) -> &'static str {
    match code {
        0 => "Despejado",
        1 => "Mayormente despejado",
        2 => "Parcialmente nublado",
        3 => "Nublado",
        45 | 48 => "Niebla",
        51 | 53 | 55 => "Llovizna",
        56 | 57 => "Llovizna helada",
        61 | 63 | 65 => "Lluvia",
        66 | 67 => "Lluvia helada",
        71 | 73 | 75 => "Nieve",
        77 => "Granos de nieve",
        80 | 81 | 82 => "Chubascos",
        85 | 86 => "Chubascos de nieve",
        95 => "Tormenta",
        96 | 99 => "Tormenta con granizo",
        _ => "Condición desconocida",
    }
}

impl WeatherReport {
    pub fn browser_lines(&self) -> Vec<String> {
        vec![
            format!(
                "{}{}",
                self.city,
                if self.country.is_empty() {
                    String::new()
                } else {
                    format!(", {}", self.country)
                }
            ),
            format!("{} °C · {}", es_number(self.temp_c, 1), self.description),
            format!(
                "Sensación {} °C · Humedad {} %",
                es_number(self.apparent_c, 1),
                self.humidity
            ),
            format!("Viento {} km/h", es_number(self.wind_kmh, 1)),
            format!("Datos de Open-Meteo · hora local {}", self.local_time),
        ]
    }

    pub fn markdown(&self) -> String {
        format!(
            "# Clima en {city}\n\n- Temperatura: {temp} °C\n- Sensación: {apparent} °C\n- Condición: {description}\n- Humedad: {humidity} %\n- Viento: {wind} km/h\n- Hora local: {time}\n\nDatos de Open-Meteo (open-meteo.com). Generado por Greek.\n",
            city = self.city,
            temp = es_number(self.temp_c, 1),
            apparent = es_number(self.apparent_c, 1),
            description = self.description,
            humidity = self.humidity,
            wind = es_number(self.wind_kmh, 1),
            time = self.local_time
        )
    }

    pub fn source_url(&self) -> String {
        format!(
            "https://api.open-meteo.com/v1/forecast?latitude={:.4}&longitude={:.4}&current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m",
            self.latitude, self.longitude
        )
    }
}

pub async fn weather(client: &reqwest::Client, city: &str) -> Result<WeatherReport> {
    let geo_response = client
        .get("https://geocoding-api.open-meteo.com/v1/search")
        .query(&[
            ("name", city),
            ("count", "1"),
            ("language", "es"),
            ("format", "json"),
        ])
        .timeout(WEATHER_TIMEOUT)
        .send()
        .await
        .context("No se pudo contactar el geocodificador de Open-Meteo.")?
        .error_for_status()
        .context("Open-Meteo rechazó la búsqueda de la ciudad.")?;

    let geo: Value = geo_response
        .json()
        .await
        .context("La respuesta del geocodificador no es JSON válido.")?;

    let result = geo
        .get("results")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .with_context(|| format!("No encontré la ciudad «{city}»."))?;

    let name = result
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(city)
        .to_string();
    let country = result
        .get("country")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let latitude = result
        .get("latitude")
        .and_then(Value::as_f64)
        .context("La ciudad no trae latitud.")?;
    let longitude = result
        .get("longitude")
        .and_then(Value::as_f64)
        .context("La ciudad no trae longitud.")?;

    let forecast_response = client
        .get("https://api.open-meteo.com/v1/forecast")
        .query(&[
            ("latitude", latitude.to_string()),
            ("longitude", longitude.to_string()),
            (
                "current",
                "temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m".to_string(),
            ),
            ("timezone", "auto".to_string()),
        ])
        .timeout(WEATHER_TIMEOUT)
        .send()
        .await
        .context("No se pudo contactar el pronóstico de Open-Meteo.")?
        .error_for_status()
        .context("Open-Meteo rechazó la consulta del pronóstico.")?;

    let forecast: Value = forecast_response
        .json()
        .await
        .context("La respuesta del pronóstico no es JSON válido.")?;

    let current = forecast
        .get("current")
        .context("Open-Meteo no devolvió datos actuales.")?;

    let temp_c = current
        .get("temperature_2m")
        .and_then(Value::as_f64)
        .context("Falta la temperatura actual.")?;
    let apparent_c = current
        .get("apparent_temperature")
        .and_then(Value::as_f64)
        .context("Falta la sensación térmica.")?;
    let humidity = current
        .get("relative_humidity_2m")
        .and_then(Value::as_f64)
        .context("Falta la humedad.")? as u64;
    let wind_kmh = current
        .get("wind_speed_10m")
        .and_then(Value::as_f64)
        .context("Falta el viento.")?;
    let code = current
        .get("weather_code")
        .and_then(Value::as_u64)
        .context("Falta el código de clima.")?;
    let local_time = current
        .get("time")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    Ok(WeatherReport {
        city: name,
        country,
        latitude,
        longitude,
        temp_c,
        apparent_c,
        humidity,
        wind_kmh,
        code,
        description: weather_description(code),
        local_time,
    })
}

fn cut_city(value: &str) -> Option<String> {
    let lower = value.to_lowercase();
    let cutters = [
        " y ",
        " para ",
        " por favor",
        " en un archivo",
        " en el archivo",
        " guárdalo",
        " guardalo",
        " guarda",
        " hoy",
        " ahora",
        "?",
        "¿",
        "!",
        ".",
        ";",
        ",",
    ];
    let mut end = value.len();
    for cutter in cutters {
        if let Some(position) = lower.find(cutter) {
            if position < end {
                end = position;
            }
        }
    }
    let city = value[..end].trim().trim_matches(|ch: char| {
        ch == '"' || ch == '\'' || ch == '«' || ch == '»' || ch == '?' || ch == '¿' || ch == '!'
    });
    if city.is_empty() {
        None
    } else {
        Some(city.chars().take(60).collect())
    }
}

pub fn extract_city(message: &str) -> Option<String> {
    let lower = message.to_lowercase();
    let keywords = ["clima", "tiempo", "temperatura"];
    let (keyword, position) = keywords
        .iter()
        .filter_map(|keyword| lower.find(keyword).map(|position| (*keyword, position)))
        .min_by_key(|(_, position)| *position)?;

    let after = &message[position + keyword.len()..];
    let after_lower = after.to_lowercase();

    let mut best: Option<(usize, usize)> = None;
    for separator in [" de ", " en ", " para "] {
        if let Some(index) = after_lower.find(separator) {
            if best
                .map(|(current, _)| index < current)
                .unwrap_or(true)
            {
                best = Some((index, separator.len()));
            }
        }
    }

    if let Some((index, length)) = best {
        let candidate = &after[index + length..];
        if let Some(city) = cut_city(candidate) {
            return Some(city);
        }
    }

    cut_city(after.trim_start_matches([' ', ':', ',', '¿', '?']))
}

pub fn wants_file(message: &str) -> bool {
    let lower = message.to_lowercase();
    ["archivo", "guarda", "guardar", "escribe", "crea", "crear"]
        .iter()
        .any(|needle| lower.contains(needle))
}

pub fn extract_file_name(message: &str) -> Option<String> {
    let tokens: Vec<&str> = message.split_whitespace().collect();
    fn punctuation(value: &str) -> &str {
        value.trim_matches(|ch: char| {
            matches!(ch, '"' | '\'' | '`' | '«' | '»' | ',' | '.' | ';' | ':' | '!' | '?' | '(' | ')')
        })
    }

    for token in &tokens {
        let cleaned = punctuation(token);
        if let Some((_, extension)) = cleaned.rsplit_once('.') {
            let extension = extension.to_ascii_lowercase();
            if matches!(extension.as_str(), "md" | "txt" | "json" | "log") {
                return Some(cleaned.to_string());
            }
        }
    }

    let triggers = ["llamado", "llamada", "nombre", "archivo"];
    let stopwords = [
        "en", "un", "una", "el", "la", "los", "las", "de", "del", "con", "y", "para", "por",
        "que", "este", "esta", "nuevo", "nueva", "guárdalo", "guardalo", "llamado", "llamada",
        "nombre", "archivo",
    ];

    for (index, token) in tokens.iter().enumerate() {
        let lower = token.to_lowercase();
        if triggers.contains(&lower.as_str()) {
            if let Some(next) = tokens.get(index + 1) {
                let cleaned = punctuation(next);
                let lower_next = cleaned.to_lowercase();
                if !cleaned.is_empty()
                    && cleaned.len() <= 60
                    && !stopwords.contains(&lower_next.as_str())
                    && cleaned.chars().any(|ch| ch.is_alphanumeric())
                {
                    return Some(format!("{cleaned}.md"));
                }
            }
        }
    }

    None
}

pub fn extract_quoted(message: &str) -> Option<String> {
    let pairs = [('"', '"'), ('“', '”'), ('«', '»'), ('`', '`')];
    let mut best: Option<String> = None;
    for (open, close) in pairs {
        let mut rest = message;
        while let Some(start) = rest.find(open) {
            let after = &rest[start + open.len_utf8()..];
            if let Some(end) = after.find(close) {
                let candidate = after[..end].trim();
                if !candidate.is_empty()
                    && best
                        .as_ref()
                        .map(|current| candidate.len() > current.len())
                        .unwrap_or(true)
                {
                    best = Some(candidate.chars().take(4000).collect());
                }
                rest = &after[end + close.len_utf8()..];
            } else {
                break;
            }
        }
    }

    if best.is_some() {
        return best;
    }

    let lower = message.to_lowercase();
    for marker in ["contenido:", "texto:"] {
        if let Some(index) = lower.find(marker) {
            let candidate = message[index + marker.len()..].trim();
            if !candidate.is_empty() {
                return Some(candidate.chars().take(4000).collect());
            }
        }
    }

    None
}

pub fn extract_terminal_command(message: &str) -> String {
    let lower = message.to_lowercase();
    let keywords = ["ejecuta", "ejecutar", "comando", "terminal"];
    let mut found: Option<(usize, &str)> = None;

    for keyword in keywords {
        if let Some(position) = lower.find(keyword) {
            if found
                .map(|(current, _)| position < current)
                .unwrap_or(true)
            {
                found = Some((position, keyword));
            }
        }
    }

    let command = if let Some((position, keyword)) = found {
        let after = &message[position + keyword.len()..];
        after
            .trim()
            .trim_start_matches([':', '¿', '?', ','])
            .trim()
            .trim_start_matches("el comando")
            .trim()
            .trim_start_matches("esto:")
            .trim()
            .to_string()
    } else {
        String::new()
    };

    let command = command.trim_matches(|ch| ch == '"' || ch == '\'' || ch == '`');
    if command.is_empty() {
        "help".to_string()
    } else {
        command.chars().take(200).collect()
    }
}

pub fn run_terminal(sandbox: &Path, command_line: &str) -> String {
    let trimmed = command_line.trim();
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let command = parts.next().unwrap_or("").to_lowercase();
    let rest = parts.next().unwrap_or("").trim();

    match command.as_str() {
        "" | "help" => [
            "Comandos disponibles en el sandbox:",
            "  date            fecha y hora local",
            "  echo <texto>    imprime el texto",
            "  whoami          usuario del sistema",
            "  pwd             ruta del sandbox",
            "  ls              lista los archivos",
            "  cat <archivo>   muestra un archivo",
        ]
        .join("\n"),
        "date" | "time" => Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "echo" => rest.to_string(),
        "whoami" => std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .unwrap_or_else(|_| "usuario".to_string()),
        "pwd" => sandbox.display().to_string(),
        "ls" | "dir" => {
            let mut names: Vec<String> = std::fs::read_dir(sandbox)
                .map(|entries| {
                    entries
                        .flatten()
                        .filter(|entry| entry.path().is_file())
                        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            names.sort();
            if names.is_empty() {
                "(sin archivos)".to_string()
            } else {
                names.join("\n")
            }
        }
        "cat" | "type" => {
            let name = sanitize_file_name(rest);
            if rest.is_empty() {
                return "Uso: cat <archivo>".to_string();
            }
            if rest.contains("..") || rest.contains('/') || rest.contains('\\') {
                return "Solo se pueden leer archivos del sandbox.".to_string();
            }
            match std::fs::read_to_string(sandbox.join(&name)) {
                Ok(content) => content.chars().take(4000).collect(),
                Err(_) => format!("No existe el archivo {name} en el sandbox."),
            }
        }
        _ => "Comando no permitido en el entorno aislado. Prueba con: date, echo, whoami, pwd, ls, cat, help."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn extracts_cities_from_spanish() {
        assert_eq!(
            extract_city("Busca el clima de Bogotá y guárdalo en un archivo").as_deref(),
            Some("Bogotá")
        );
        assert_eq!(
            extract_city("¿Cómo está el clima en Ciudad de México?").as_deref(),
            Some("Ciudad de México")
        );
        assert_eq!(extract_city("clima Medellín").as_deref(), Some("Medellín"));
        assert_eq!(extract_city("hola"), None);
    }

    #[test]
    fn extracts_file_names() {
        assert_eq!(
            extract_file_name("crea un archivo notas.md con \"hola\"").as_deref(),
            Some("notas.md")
        );
        assert_eq!(
            extract_file_name("crea un archivo llamado ideas").as_deref(),
            Some("ideas.md")
        );
        assert_eq!(extract_file_name("guárdalo en un archivo"), None);
    }

    #[test]
    fn extracts_quoted_content() {
        assert_eq!(
            extract_quoted("crea un archivo con \"hola mundo\"").as_deref(),
            Some("hola mundo")
        );
    }

    #[test]
    fn blocks_unsafe_terminal_commands() {
        let sandbox = PathBuf::from(".");
        let output = run_terminal(&sandbox, "rm -rf /");
        assert!(output.contains("no permitido"));
        assert!(run_terminal(&sandbox, "date").contains('-'));
        assert_eq!(run_terminal(&sandbox, "echo hola"), "hola");
    }

    #[test]
    fn maps_weather_codes() {
        assert_eq!(weather_description(0), "Despejado");
        assert_eq!(weather_description(3), "Nublado");
        assert_eq!(weather_description(95), "Tormenta");
    }

    #[test]
    fn formats_spanish_numbers() {
        assert_eq!(es_number(18.44, 1), "18,4");
    }
}
