import SwiftUI

struct VirtualDesktopView: View {
    let desktop: VirtualDesktop?
    var detailed: Bool = false

    var body: some View {
        GeometryReader { geometry in
            let scale = min(geometry.size.width / 900, geometry.size.height / 600)

            ZStack(alignment: .topLeading) {
                Color.greekBg

                if let desktop {
                    if desktop.windows.isEmpty {
                        Text("Escritorio vacío")
                            .font(.greek(9))
                            .foregroundStyle(Color.greekMuted)
                            .position(x: geometry.size.width / 2, y: geometry.size.height / 2)
                    } else {
                        ForEach(desktop.windows) { window in
                            DesktopWindowView(
                                window: window,
                                desktop: desktop,
                                detailed: detailed,
                                isActive: window.id == desktop.activeWindowId
                            )
                            .frame(
                                width: max(CGFloat(window.width) * scale, 46),
                                height: max(CGFloat(window.height) * scale, 30)
                            )
                            .offset(
                                x: min(max(0, CGFloat(window.x) * scale), max(0, geometry.size.width - 46)),
                                y: min(max(0, CGFloat(window.y) * scale), max(0, geometry.size.height - 30))
                            )
                        }
                    }
                } else {
                    VStack(spacing: 5) {
                        Image(systemName: "desktopcomputer")
                            .font(.system(size: 18, weight: .light))
                        Text("Sin conexión")
                            .font(.greek(9))
                    }
                    .foregroundStyle(Color.greekMuted)
                    .position(x: geometry.size.width / 2, y: geometry.size.height / 2)
                }
            }
            .frame(width: geometry.size.width, height: geometry.size.height)
            .clipped()
        }
        .background(Color.greekBg)
    }
}

private struct DesktopWindowView: View {
    let window: VirtualWindow
    let desktop: VirtualDesktop
    let detailed: Bool
    let isActive: Bool

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 3) {
                Circle().fill(Color(hex: 0xF87171)).frame(width: 3, height: 3)
                Circle().fill(Color(hex: 0xFBBF24)).frame(width: 3, height: 3)
                Circle().fill(Color(hex: 0x22C55E)).frame(width: 3, height: 3)

                if detailed {
                    Text(window.title)
                        .font(.greek(8, weight: .medium))
                        .foregroundStyle(Color.greekMuted)
                        .lineLimit(1)
                        .padding(.leading, 3)
                }

                Spacer(minLength: 0)
            }
            .padding(.horizontal, 4)
            .frame(height: detailed ? 16 : 9)
            .background(Color.greekActive)

            Rectangle()
                .fill(Color.greekBorder)
                .frame(height: 0.5)

            content
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        }
        .background(Color.greekSurface)
        .clipShape(RoundedRectangle(cornerRadius: detailed ? 5 : 3, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: detailed ? 5 : 3, style: .continuous)
                .stroke(isActive ? Color(hex: 0x3F3F46) : Color.greekBorderStrong, lineWidth: 0.8)
        )
    }

    @ViewBuilder
    private var content: some View {
        switch window.kind {
        case .browser:
            if detailed, let browser = desktop.browser {
                VStack(alignment: .leading, spacing: 2) {
                    Text(browser.url)
                        .font(.greekMono(6))
                        .foregroundStyle(Color.greekMuted)
                        .lineLimit(1)

                    ForEach(Array(browser.lines.prefix(4).indices), id: \.self) { index in
                        Text(browser.lines[index])
                            .font(.greek(7))
                            .foregroundStyle(Color.greekText.opacity(0.8))
                            .lineLimit(1)
                    }
                }
                .padding(4)
            } else {
                bars
            }

        case .files:
            if detailed, !desktop.files.isEmpty {
                VStack(alignment: .leading, spacing: 2) {
                    ForEach(desktop.files.prefix(4)) { file in
                        Text(file.name)
                            .font(.greekMono(6))
                            .foregroundStyle(Color.greekText.opacity(0.8))
                            .lineLimit(1)
                    }
                }
                .padding(4)
            } else {
                bars
            }

        case .terminal:
            if detailed {
                VStack(alignment: .leading, spacing: 1) {
                    ForEach(Array(desktop.terminal.suffix(5).indices), id: \.self) { index in
                        Text(desktop.terminal[index])
                            .font(.greekMono(6))
                            .foregroundStyle(Color.greekText.opacity(0.7))
                            .lineLimit(1)
                    }
                }
                .padding(4)
            } else {
                bars
            }
        }
    }

    private var bars: some View {
        VStack(alignment: .leading, spacing: 3) {
            RoundedRectangle(cornerRadius: 1).fill(Color.greekBorderStrong).frame(height: 2)
            RoundedRectangle(cornerRadius: 1).fill(Color.greekBorderStrong).frame(width: 24, height: 2)
            RoundedRectangle(cornerRadius: 1).fill(Color.greekBorderStrong).frame(width: 16, height: 2)
        }
        .padding(4)
    }
}
