use crate::models::{BrowserState, ToolChip};
use crate::persist;
use crate::state::{Core, ProviderChoice};
use crate::tools;
use anyhow::{bail, Context, Result};
use chrono::Local;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

const SYSTEM_PROMPT: &str = "Eres Greek, un agente personal que corre en el equipo del usuario. Responde en español neutro, claro y breve, con un máximo de 160 palabras. No inventes datos, cifras, fuentes ni acciones. Si no sabes algo, dilo. Tus herramientas reales son: clima con Open-Meteo, archivos dentro de un sandbox y una terminal limitada. No afirmes haber usado una herramienta si el contexto no lo confirma. No uses guiones largos.";

fn short_model(model: &str) -> String {
    let tail = model.rsplit(['/', ':']).next().unwrap_or(model);
    tail.chars().take(28).collect()
}

fn is_greeting(lower: &str) -> bool {
    [
        "hola",
        "buenas",
        "quién eres",
        "quien eres",
        "qué puedes",
        "que puedes",
        "ayuda",
        "help",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn local_chat_reply() -> String {
    "Estoy en modo local, sin un modelo de lenguaje conectado. Puedo consultar el clima real con Open-Meteo, crear archivos en el escritorio virtual y ejecutar comandos seguros en la terminal aislada. Para preguntas abiertas, configura OPENCODE_ZEN_API_KEY y OPENCODE_ZEN_MODEL, o usa Ollama y OpenRouter en el archivo .env.".to_string()
}

fn greeting_reply() -> String {
    "Hola. Soy Greek. Puedo consultar el clima real, guardar archivos en el escritorio virtual y ejecutar comandos seguros en la terminal aislada. ¿Qué necesitas?".to_string()
}

fn messages_for(core: &Core, chat_id: &str) -> Vec<Value> {
    let mut messages = vec![json!({ "role": "system", "content": SYSTEM_PROMPT })];
    for (role, text) in core.history(chat_id, 12) {
        let text: String = text.chars().take(4000).collect();
        messages.push(json!({ "role": role, "content": text }));
    }
    messages
}

async fn call_openrouter(
    client: &reqwest::Client,
    key: &str,
    model: &str,
    messages: Vec<Value>,
) -> Result<String> {
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(key)
        .header("HTTP-Referer", "https://greek.local")
        .header("X-Title", "Greek")
        .json(&json!({
            "model": model,
            "messages": messages,
            "max_tokens": 700,
            "temperature": 0.4
        }))
        .timeout(Duration::from_secs(45))
        .send()
        .await
        .context("No se pudo contactar OpenRouter.")?
        .error_for_status()
        .context("OpenRouter rechazó la solicitud.")?;

    let body: Value = response
        .json()
        .await
        .context("OpenRouter devolvió una respuesta inválida.")?;
    let text = body
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if text.is_empty() {
        bail!("OpenRouter devolvió una respuesta vacía.");
    }
    Ok(text)
}

async fn call_ollama(
    client: &reqwest::Client,
    model: &str,
    messages: Vec<Value>,
) -> Result<String> {
    let response = client
        .post("http://127.0.0.1:11434/api/chat")
        .json(&json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "options": { "temperature": 0.4 }
        }))
        .timeout(Duration::from_secs(90))
        .send()
        .await
        .context("No se pudo contactar Ollama.")?
        .error_for_status()
        .context("Ollama rechazó la solicitud.")?;

    let body: Value = response
        .json()
        .await
        .context("Ollama devolvió una respuesta inválida.")?;
    let text = body
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if text.is_empty() {
        bail!("Ollama devolvió una respuesta vacía.");
    }
    Ok(text)
}

async fn call_opencode_zen(
    client: &reqwest::Client,
    key: &str,
    model: &str,
    base: &str,
    messages: Vec<Value>,
) -> Result<String> {
    let url = format!("{}/chat/completions", base.trim_end_matches('/'));
    let response = client
        .post(url)
        .bearer_auth(key)
        .json(&json!({
            "model": model,
            "messages": messages,
            "max_tokens": 700,
            "temperature": 0.4
        }))
        .timeout(Duration::from_secs(45))
        .send()
        .await
        .context("No se pudo contactar OpenCode Zen.")?
        .error_for_status()
        .context("OpenCode Zen rechazó la solicitud.")?;

    let body: Value = response
        .json()
        .await
        .context("OpenCode Zen devolvió una respuesta inválida.")?;
    let text = body
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    if text.is_empty() {
        bail!("OpenCode Zen devolvió una respuesta vacía.");
    }
    Ok(text)
}

async fn ask_provider(
    core: &Core,
    choice: ProviderChoice,
    chat_id: &str,
) -> Result<(String, String)> {
    let messages = messages_for(core, chat_id);
    match choice {
        ProviderChoice::Local => bail!("No hay proveedor remoto configurado."),
        ProviderChoice::OpenRouter { key, model } => {
            let text = call_openrouter(&core.http, &key, &model, messages).await?;
            Ok((text, format!("openrouter · {}", short_model(&model))))
        }
        ProviderChoice::Ollama { model } => {
            let text = call_ollama(&core.http, &model, messages).await?;
            Ok((text, format!("ollama · {}", short_model(&model))))
        }
        ProviderChoice::OpenCodeZen { key, model, base } => {
            let text = call_opencode_zen(&core.http, &key, &model, &base, messages).await?;
            Ok((text, format!("opencode zen · {}", short_model(&model))))
        }
    }
}

pub async fn respond(core: Arc<Core>, chat_id: String, user_text: String) {
    tokio::time::sleep(Duration::from_millis(700)).await;

    let lower = user_text.to_lowercase();
    let requested = core.chat_model(&chat_id);
    let (choice, base_reason) = core.resolve_model(&requested);

    let is_terminal = lower.contains("terminal")
        || lower.contains("ejecuta")
        || lower.contains("ejecutar")
        || lower.contains("comando");
    let is_weather = lower.contains("clima")
        || lower.contains("tiempo")
        || lower.contains("temperatura");
    let is_file = tools::wants_file(&user_text)
        && (lower.contains("archivo")
            || lower.contains("crea")
            || lower.contains("crear")
            || lower.contains("escribe")
            || lower.contains("guarda"));
    let is_browser = lower.contains("navegador") || lower.contains("navega");

    if is_terminal {
        let command = tools::extract_terminal_command(&user_text);
        let output = tools::run_terminal(&core.sandbox, &command);
        let first = command.split_whitespace().next().unwrap_or("help").to_string();

        let mut lines = vec![format!("$ {command}")];
        lines.extend(output.lines().map(str::to_string));
        core.log_terminal(lines);

        let chips = vec![ToolChip {
            label: "Terminal".into(),
            detail: Some(first),
        }];
        let text = format!("Ejecuté `{command}` en el entorno aislado:\n\n{output}");
        core.finish_agent(
            &chat_id,
            text,
            chips,
            format!("{base_reason} · terminal"),
        );
        return;
    }

    if is_weather {
        let city = tools::extract_city(&user_text).unwrap_or_else(|| "Bogotá".to_string());
        match tools::weather(&core.http, &city).await {
            Ok(report) => {
                let mut chips = vec![ToolChip {
                    label: "Clima".into(),
                    detail: Some(report.city.clone()),
                }];

                core.set_browser(BrowserState {
                    title: "Open-Meteo".into(),
                    url: report.source_url(),
                    status: 200,
                    lines: report.browser_lines(),
                });
                core.log_terminal(vec![format!(
                    "clima {}: {} °C, {}",
                    report.city,
                    tools::es_number(report.temp_c, 1),
                    report.description
                )]);

                let mut saved_file: Option<String> = None;
                let mut save_error: Option<String> = None;
                if tools::wants_file(&user_text) {
                    let name = tools::extract_file_name(&user_text).unwrap_or_else(|| {
                        format!("clima-{}.md", persist::slugify(&report.city))
                    });
                    match core.write_file(&name, &report.markdown()) {
                        Ok(file) => {
                            chips.push(ToolChip {
                                label: "Archivos".into(),
                                detail: Some(file.name.clone()),
                            });
                            saved_file = Some(file.name);
                        }
                        Err(error) => save_error = Some(error.to_string()),
                    }
                }

                let mut text = format!(
                    "En {} hay {} °C y {}. Sensación de {} °C, humedad {} % y viento de {} km/h.",
                    report.city,
                    tools::es_number(report.temp_c, 1),
                    report.description.to_lowercase(),
                    tools::es_number(report.apparent_c, 1),
                    report.humidity,
                    tools::es_number(report.wind_kmh, 1)
                );

                if let Some(name) = saved_file {
                    text.push_str(&format!(
                        "\n\nGuardé el resumen en `{name}` dentro del escritorio virtual."
                    ));
                } else if let Some(error) = save_error {
                    text.push_str(&format!("\n\nNo pude guardar el archivo: {error}"));
                }

                core.finish_agent(&chat_id, text, chips, format!("{base_reason} · clima"));
            }
            Err(error) => {
                core.log_terminal(vec![format!("clima: error ({error})")]);
                let text = format!(
                    "No pude consultar Open-Meteo para {city}. No tengo datos y no voy a inventarlos. Revisa la conexión y vuelve a intentarlo."
                );
                core.finish_agent(
                    &chat_id,
                    text,
                    Vec::new(),
                    format!("{base_reason} · clima"),
                );
            }
        }
        return;
    }

    if is_file {
        let name = tools::extract_file_name(&user_text).unwrap_or_else(|| "nota.md".to_string());
        let content = tools::extract_quoted(&user_text).unwrap_or_else(|| {
            format!(
                "# Nota\n\nCreada por Greek el {}.\n\nSolicitud: {}\n",
                Local::now().format("%Y-%m-%d %H:%M"),
                user_text
            )
        });

        match core.write_file(&name, &content) {
            Ok(file) => {
                let chips = vec![ToolChip {
                    label: "Archivos".into(),
                    detail: Some(file.name.clone()),
                }];
                let text = format!(
                    "Listo. Creé `{}` en el escritorio virtual. Puedes verlo en la ventana Archivos.",
                    file.name
                );
                core.finish_agent(&chat_id, text, chips, format!("{base_reason} · archivos"));
            }
            Err(error) => {
                let text = format!("No pude crear el archivo en el sandbox: {error}");
                core.finish_agent(
                    &chat_id,
                    text,
                    Vec::new(),
                    format!("{base_reason} · archivos"),
                );
            }
        }
        return;
    }

    if is_browser {
        core.open_app(crate::models::WindowKind::Browser);
        let has_page = core.snapshot().desktop.browser.is_some();
        let text = if has_page {
            "Abrí el navegador del escritorio virtual. Muestra los últimos datos consultados.".to_string()
        } else {
            "Abrí el navegador del escritorio virtual. Puedo cargar el clima de una ciudad si me la dices.".to_string()
        };
        let chips = vec![ToolChip {
            label: "Navegador".into(),
            detail: Some("abierto".into()),
        }];
        core.finish_agent(&chat_id, text, chips, format!("{base_reason} · navegador"));
        return;
    }

    if is_greeting(&lower) {
        core.finish_agent(
            &chat_id,
            greeting_reply(),
            Vec::new(),
            format!("{base_reason} · chat"),
        );
        return;
    }

    if matches!(choice, ProviderChoice::Local) {
        core.finish_agent(&chat_id, local_chat_reply(), Vec::new(), base_reason);
        return;
    }

    match ask_provider(&core, choice, &chat_id).await {
        Ok((text, reason)) => {
            core.finish_agent(&chat_id, text, Vec::new(), reason);
        }
        Err(_) => {
            let text = format!(
                "{}\n\nNo pude contactar el proveedor remoto, así que respondo en modo local.",
                local_chat_reply()
            );
            core.finish_agent(
                &chat_id,
                text,
                Vec::new(),
                format!("{base_reason} · respaldo"),
            );
        }
    }
}
