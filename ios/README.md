# Greek para iPhone

Cliente SwiftUI que se conecta al núcleo de Greek en el PC por WebSocket dentro de la red Wi-Fi local. No usa servicios de terceros ni envía datos a internet: solo habla con la dirección que indiques.

## Requisitos

- macOS con Xcode 16 o posterior.
- XcodeGen: `brew install xcodegen`.
- Un iPhone con iOS 17 o posterior, o el simulador de Xcode.
- El escritorio de Greek ejecutándose con el núcleo accesible en la red local.

## Generar y compilar

```bash
cd ios
xcodegen generate
open Greek.xcodeproj
```

En Xcode, elige el esquema `Greek`, selecciona tu iPhone o un simulador y pulsa Run. Para instalar en un iPhone físico, configura tu equipo en Signing & Capabilities.

Compilación por línea de comandos para el simulador:

```bash
xcodebuild -project Greek.xcodeproj -scheme Greek \
  -destination 'platform=iOS Simulator,name=iPhone 16' build
```

Para un dispositivo físico hace falta firma. Ajusta el equipo y el identificador si lo necesitas:

```bash
xcodebuild -project Greek.xcodeproj -scheme Greek \
  -destination 'generic/platform=iOS' \
  DEVELOPMENT_TEAM=TU_EQUIPO build
```

## Conectar con el PC

1. En el PC, abre Greek y pulsa **Conectar iPhone** en la cabecera del chat.
2. Si el núcleo solo escucha en loopback, reinícialo con `GREEK_BIND=0.0.0.0:4319` antes de abrir la app.
3. En el iPhone, abre el menú y elige **Conectar al PC**.
4. Pega la dirección (por ejemplo `http://192.168.1.10:4319`) y el token.
5. Pulsa **Conectar**. La tarjeta **Tu PC** debe mostrar el punto verde y el escritorio virtual.

La dirección se guarda en `UserDefaults` y el token en el Llavero del iPhone (`kSecClassGenericPassword`, servicio `com.greek.ios`). El token no se escribe en el código ni se muestra en la interfaz después de guardarlo.

## Estructura

- `GreekApp.swift`: punto de entrada.
- `Theme.swift`: paleta y tipografía del diseño.
- `Models.swift`: espejo del protocolo JSON del núcleo.
- `GreekClient.swift`: WebSocket, reconexión y comandos.
- `GreekAppModel.swift`: estado de la app, persistencia y acciones.
- `RootView.swift`: cabecera, chat, tarjeta del PC y compositor.
- `Views/`: componentes de chat, escritorio virtual, menú y emparejamiento.
- `Info.plist`: ATS local, red local y orientación vertical.
- `Assets.xcassets`: icono de la app con el templo griego.

## Notas y límites

- La excepción de ATS (`NSAllowsArbitraryLoads`) existe para poder hablar por HTTP con el PC en la red local. Si más adelante usas TLS, quítala.
- **Ver pantalla** muestra el escritorio virtual que mantiene el núcleo, no una captura de la pantalla real del PC.
- Este código se escribió en Windows, donde no hay toolchain de Swift. La compilación se verifica en GitHub Actions con Xcode en macOS, para simulador iOS. Todavía no se ha ejecutado en un iPhone físico.
- La interfaz usa controles nativos (`Menu`, estilos de botón del sistema) y Liquid Glass en iOS 26 mediante `GlassSupport.swift`. En iOS 17 y 18 conserva el diseño plano original.
- La app no incluye claves de IA. El proveedor se elige en el escritorio y el iPhone solo muestra el modelo activo.
