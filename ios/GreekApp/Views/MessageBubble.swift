import SwiftUI

struct MessageBubble: View {
    let message: ChatMessage

    var body: some View {
        if message.role == .user {
            userBubble
        } else {
            assistantRow
        }
    }

    private var userBubble: some View {
        HStack {
            Spacer(minLength: 44)
            Text(message.text)
                .font(.greek(15))
                .foregroundStyle(Color.greekText)
                .lineSpacing(4)
                .textSelection(.enabled)
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .background(
                    RoundedRectangle(cornerRadius: 16, style: .continuous)
                        .fill(Color.greekActive)
                )
        }
    }

    private var assistantRow: some View {
        HStack(alignment: .top, spacing: 10) {
            ZStack {
                Circle().fill(Color.greekText)
                Text("G")
                    .font(.greek(11, weight: .bold))
                    .foregroundStyle(Color.greekBg)
            }
            .frame(width: 24, height: 24)
            .accessibilityHidden(true)

            VStack(alignment: .leading, spacing: 8) {
                inlineText(message.text)
                    .lineSpacing(4)
                    .textSelection(.enabled)

                if !message.tools.isEmpty {
                    ToolChipRow(chips: message.tools)
                }
            }

            Spacer(minLength: 0)
        }
    }

    private func inlineText(_ raw: String) -> Text {
        let parts = raw.components(separatedBy: "`")
        var result = Text("")

        for (index, part) in parts.enumerated() {
            if index.isMultiple(of: 2) {
                result = result + Text(part)
                    .font(.greek(15))
                    .foregroundColor(Color.greekText)
            } else {
                result = result + Text(part)
                    .font(.greekMono(14))
                    .foregroundColor(Color.greekText.opacity(0.82))
            }
        }

        return result
    }
}

struct ToolChipRow: View {
    let chips: [ToolChip]

    var body: some View {
        HStack(spacing: 6) {
            ForEach(chips) { chip in
                HStack(spacing: 5) {
                    Image(systemName: icon(for: chip.label))
                        .font(.system(size: 10, weight: .medium))
                    Text(chip.detail.map { "\(chip.label) · \($0)" } ?? chip.label)
                        .font(.greek(11, weight: .medium))
                        .lineLimit(1)
                }
                .foregroundStyle(Color.greekMuted)
                .padding(.horizontal, 8)
                .frame(height: 24)
                .greekCapsuleSurface()
            }
        }
    }

    private func icon(for label: String) -> String {
        switch label {
        case "Clima": return "cloud.sun"
        case "Archivos": return "folder"
        case "Terminal": return "terminal"
        case "Navegador": return "globe"
        default: return "wrench.and.screwdriver"
        }
    }
}

struct TypingIndicator: View {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var animating = false

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            ZStack {
                Circle().fill(Color.greekText)
                Text("G")
                    .font(.greek(11, weight: .bold))
                    .foregroundStyle(Color.greekBg)
            }
            .frame(width: 24, height: 24)
            .accessibilityHidden(true)

            HStack(spacing: 4) {
                ForEach(0..<3, id: \.self) { index in
                    Circle()
                        .fill(Color.greekMuted)
                        .frame(width: 4, height: 4)
                        .opacity(animating ? 1 : 0.35)
                        .animation(
                            reduceMotion
                                ? nil
                                : .easeInOut(duration: 0.7)
                                    .repeatForever(autoreverses: true)
                                    .delay(Double(index) * 0.15),
                            value: animating
                        )
                }
            }
            .frame(height: 24)

            Spacer(minLength: 0)
        }
        .onAppear {
            animating = true
        }
        .accessibilityLabel("Greek está escribiendo")
    }
}
