import SwiftUI

struct ChatHeader: View {
    let title: String
    let modelLabel: String
    let models: [ModelOption]
    let selectedModel: String
    let onNewChat: () -> Void
    let onOpenMenu: () -> Void
    let onPair: () -> Void
    let onModelSelect: (String) -> Void

    var body: some View {
        ZStack {
            Text(title)
                .font(.greek(15, weight: .semibold))
                .foregroundStyle(Color.greekText)
                .lineLimit(1)
                .padding(.horizontal, 92)

            HStack(spacing: 8) {
                Menu {
                    Button {
                        onNewChat()
                    } label: {
                        Label("Nuevo chat", systemImage: "square.and.pencil")
                    }

                    Button {
                        onOpenMenu()
                    } label: {
                        Label("Chats y modelo", systemImage: "line.3.horizontal")
                    }

                    Button {
                        onPair()
                    } label: {
                        Label("Conectar al PC", systemImage: "wifi")
                    }
                } label: {
                    Image(systemName: "line.3.horizontal")
                        .font(.system(size: 16, weight: .medium))
                        .foregroundStyle(Color.greekText)
                        .frame(width: 44, height: 44)
                        .contentShape(Rectangle())
                }
                .accessibilityLabel("Menú")
                .greekGlassButton()

                Spacer(minLength: 0)

                Menu {
                    ForEach(models) { option in
                        Button {
                            onModelSelect(option.id)
                        } label: {
                            if option.id == selectedModel {
                                Label(option.label, systemImage: "checkmark")
                            } else {
                                Text(option.label)
                            }
                        }
                        .disabled(!option.available)
                    }
                } label: {
                    HStack(spacing: 5) {
                        Text(modelLabel)
                            .font(.greek(12, weight: .medium))
                            .lineLimit(1)
                        Image(systemName: "chevron.down")
                            .font(.system(size: 9, weight: .semibold))
                    }
                    .foregroundStyle(Color.greekText)
                    .padding(.horizontal, 11)
                    .frame(minHeight: 44)
                    .greekCapsuleSurface()
                }
                .accessibilityLabel("Modelo: \(modelLabel)")
            }
        }
        .padding(.horizontal, 8)
        .frame(height: 56)
        .background(Color.greekBg)
        .overlay(alignment: .bottom) {
            Rectangle()
                .fill(Color.greekBorder)
                .frame(height: 1)
        }
    }
}
