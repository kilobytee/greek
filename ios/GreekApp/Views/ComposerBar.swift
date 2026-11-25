import SwiftUI

struct ComposerBar: View {
    @Binding var text: String
    let canSend: Bool
    let onSend: () -> Void

    var body: some View {
        HStack(alignment: .bottom, spacing: 6) {
            TextField("Escribe un mensaje...", text: $text, axis: .vertical)
                .textFieldStyle(.plain)
                .font(.greek(15))
                .foregroundStyle(Color.greekText)
                .tint(Color.greekText)
                .lineLimit(1...5)
                .padding(.leading, 4)
                .padding(.vertical, 10)
                .submitLabel(.send)
                .onSubmit {
                    if canSend { onSend() }
                }

            Button(action: onSend) {
                Image(systemName: "arrow.up")
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundStyle(Color.greekBg)
                    .frame(width: 36, height: 36)
                    .greekSendSurface()
                    .frame(width: 44, height: 44)
                    .contentShape(Circle())
            }
            .disabled(!canSend)
            .opacity(canSend ? 1 : 0.35)
            .accessibilityLabel("Enviar mensaje")
            .greekGlassButton(prominent: true)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 4)
        .greekSurface(cornerRadius: 24)
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
    }
}
