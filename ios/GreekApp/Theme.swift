import SwiftUI

extension Color {
    init(hex: UInt32) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255,
            opacity: 1
        )
    }

    static let greekBg = Color(hex: 0x0B0B0C)
    static let greekSurface = Color(hex: 0x0F0F11)
    static let greekActive = Color(hex: 0x161618)
    static let greekText = Color(hex: 0xE4E4E7)
    static let greekMuted = Color(hex: 0x71717A)
    static let greekBorder = Color(hex: 0x1A1A1E)
    static let greekBorderStrong = Color(hex: 0x27272A)
    static let greekOnline = Color(hex: 0x22C55E)
    static let greekWarning = Color(hex: 0xFBBF24)
    static let greekDanger = Color(hex: 0xF87171)
}

extension Font {
    static func greek(_ size: CGFloat, weight: Font.Weight = .regular) -> Font {
        .system(size: size, weight: weight)
    }

    static func greekMono(_ size: CGFloat, weight: Font.Weight = .regular) -> Font {
        .system(size: size, weight: weight, design: .monospaced)
    }
}
