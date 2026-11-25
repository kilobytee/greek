// swift-tools-version: 6.0
// Empaquetado para xtool (compilación de iOS desde Linux/WSL).
// El proyecto de Xcode sigue siendo la vía principal: `xcodegen generate`.

import PackageDescription

let package = Package(
	name: "Greek",
	platforms: [
		.iOS(.v17)
	],
	products: [
		// xtool espera exactamente un producto de librería que represente la app.
		.library(
			name: "GreekApp",
			targets: ["GreekApp"]
		)
	],
	targets: [
		.target(
			name: "GreekApp",
			path: "GreekApp",
			exclude: [
				"Info.plist",
				"Assets.xcassets"
			]
		)
	],
	swiftLanguageModes: [.v5]
)
