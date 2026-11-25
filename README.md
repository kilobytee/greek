# Greek

Agente personal con un escritorio virtual aislado. La app de escritorio corre en tu equipo, el núcleo en Rust hace el trabajo real y el iPhone se conecta a ese núcleo por la red local. Todo el stack es gratuito: modo local por reglas, Ollama o OpenRouter.

El «escritorio virtual» es un lienzo de estado que mantiene el núcleo, con ventanas de Navegador, Archivos y Terminal. No es una máquina virtual y no captura la pantalla real del PC.

## Arquitectura

```
iPhone (SwiftUI) ───── WebSocket ────┐
                                     ├── Núcleo Rust (axum, 127.0.0.1:4319)
Desktop (Tauri 2 + Svelte 5) ────────┘        ├── Open-Meteo (clima real)
                                              ├── Ollama / OpenRouter (opcional)
                                              ├── Sandbox de archivos
                                              └── Terminal con lista blanca
```

- **Escritorio**: Tauri 2, SvelteKit 3, Svelte 5 y TypeScript. Diseño oscuro de tres columnas.
- **Núcleo**: Rust con axum, tokio, reqwest y keyring. Estado persistido en JSON, token en el llavero del sistema.
- **iPhone**: SwiftUI, WebSocket nativo, token en el Llavero, objetivos táctiles de 44 pt.
- **Proveedores**: local por reglas, Ollama en `127.0.0.1:11434` u OpenRouter con clave gratuita. Sin claves, la app sigue funcionando con herramientas reales.

## Estructura

```
greek/
├── desktop/
│   ├── src/                 Interfaz SvelteKit
│   ├── src-tauri/           Núcleo Rust y configuración Tauri
│   ├── static/
│   └── package.json
├── ios/
│   ├── GreekApp/            Fuentes SwiftUI
│   ├── project.yml          Proyecto XcodeGen
│   └── README.md
├── .env.example
└── README.md
```

## Requisitos

- Windows 10/11 con WebView2 y toolchain MSVC, o macOS.
- Node.js 20 o posterior (probado con 24.21.0).
- Rust 1.77 o posterior (probado con 1.99.1).
- Para iOS: macOS con Xcode 16 o posterior y XcodeGen.

## Escritorio

```bash
cd desktop
npm install
npm run tauri dev
```

La primera compilación de Rust tarda varios minutos. La ventana abre en 1280x800.

Modo núcleo sin ventana, útil para probar la interfaz en el navegador:

```bash
cd desktop
npm run server
```

En otra terminal:

```bash
cd desktop
npm run dev
# abre http://localhost:1420
```

Compilar el instalador:

```bash
cd desktop
npm run tauri build
```

Los artefactos quedan en `desktop/src-tauri/target/release/bundle`.

### Configuración opcional

Copia `.env.example` a `desktop/.env` y completa lo que uses:

- `OPENROUTER_API_KEY` y `GREEK_OPENROUTER_MODEL` para OpenRouter.
- `GREEK_OLLAMA_MODEL` para fijar un modelo de Ollama. Si no se define, Greek usa el primero que detecte.
- `GREEK_BIND=0.0.0.0:4319` para aceptar conexiones del iPhone en la misma Wi-Fi.
- `GREEK_DATA_DIR` para cambiar la carpeta de datos.

Sin ninguna clave, el modo local responde con reglas y ejecuta clima, archivos y terminal.

### Datos y secretos

- Windows: `%LOCALAPPDATA%\Greek\state.json` y `%LOCALAPPDATA%\Greek\sandbox`.
- macOS: `~/Library/Application Support/Greek/`.
- Token: Credential Manager de Windows o Llavero de macOS, servicio `com.greek.desktop`, cuenta `core-token`. Si el llavero falla, se usa `token` con permisos restringidos.
- El token también se muestra en el diálogo **Conectar iPhone** para emparejar el móvil.

## iPhone

```bash
cd ios
xcodegen generate
open Greek.xcodeproj
```

En Xcode elige el esquema `Greek` y ejecuta en un simulador o en tu iPhone. Para un dispositivo físico configura tu equipo de firma. Instrucciones completas en `ios/README.md`.

El iPhone necesita que el núcleo escuche en la red local:

```powershell
# PowerShell, desde desktop/
$env:GREEK_BIND="0.0.0.0:4319"
npm run server
```

Luego abre el menú de la app, elige **Conectar al PC** y pega la dirección y el token que muestra el escritorio.

## Protocolo

- `GET /api/health`: estado y versión.
- `GET /api/dev/bootstrap`: URL, IP local y token. Solo acepta conexiones desde loopback.
- `WS /ws?token=...`: eventos `snapshot`, `error` y `pong`; comandos `newChat`, `selectChat`, `sendMessage`, `setModel`, `desktopOpen`, `desktopClose`, `desktopFocus`, `desktopSelectFile` y `ping`.
- Un token inválido cierra el WebSocket con el código `4401`.

## Seguridad

- El servidor escucha en `127.0.0.1:4319` por defecto. Exponerlo en la LAN es una decisión explícita con `GREEK_BIND`.
- El token se compara en tiempo constante y viaja como parámetro del WebSocket.
- Los orígenes del WebSocket están limitados a la app Tauri y al servidor de desarrollo.
- El bootstrap de desarrollo solo responde a loopback.
- Los archivos se escriben en un sandbox con nombres saneados y extensión limitada.
- La terminal no abre un shell: solo permite `help`, `date`, `echo`, `whoami`, `pwd`, `ls` y `cat`.
- El iPhone usa una excepción de ATS para hablar por HTTP con la red local. Si añades TLS, quítala.

## Verificación

| Comprobación | Resultado |
|---|---|
| `npm run check` | 0 errores, 0 advertencias |
| `npm run build` | Correcto, sitio estático en `desktop/build` |
| `cargo check` | Correcto |
| `cargo test` | 8 pruebas correctas |
| `cargo build` | Correcto |
| Prueba real del núcleo | Clima de Bogotá con Open-Meteo, archivo `clima-bogota.md`, terminal `date`, token inválido cerrado con 4401 |
| App Tauri | Compilada y abierta en Windows |
| App iOS | Compilada en GitHub Actions con Xcode 16 en `macos-15`, para simulador iOS, run `45679739998` |

Comandos de la tabla:

```bash
cd desktop
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --manifest-path src-tauri/Cargo.toml
```

## Integración continua

El repositorio está en `https://github.com/kilobytee/greek`. El workflow `.github/workflows/ios.yml` compila la app iOS en cada cambio dentro de `ios/`:

1. Instala XcodeGen con Homebrew.
2. Genera `Greek.xcodeproj` desde `project.yml`.
3. Ejecuta `xcodebuild` para simulador iOS sin firma.
4. Sube `xcodebuild.log` como artefacto si la compilación falla.

Los resultados están en `https://github.com/kilobytee/greek/actions`. Los runners estándar son gratis en repositorios públicos.

## Límites conocidos

- Sin un modelo conectado, las respuestas abiertas usan reglas locales y no un LLM general.
- Los modelos gratuitos de OpenRouter cambian con el tiempo; define el identificador exacto vigente.
- Ollama debe estar en marcha antes de abrir Greek para que aparezca como disponible.
- La app iOS se compila en CI para simulador, pero aún no se ha ejecutado en un iPhone físico. Para instalarla hace falta una cuenta de Apple y confiar en el dispositivo.
- **Ver pantalla** muestra el escritorio virtual, no la pantalla real del equipo.
