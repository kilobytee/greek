import SwiftUI
import UIKit

struct PairingForm: View {
    @EnvironmentObject private var model: GreekAppModel
    @Environment(\.dismiss) private var dismiss
    @State private var error: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 18) {
            Text("En el PC, abre Greek y pulsa «Conectar iPhone». Copia la dirección y el token. Ambos equipos deben estar en la misma red Wi-Fi.")
                .font(.greek(13))
                .foregroundStyle(Color.greekMuted)
                .lineSpacing(3)

            field(
                title: "Dirección",
                placeholder: "http://192.168.1.10:4319",
                text: $model.pairingURL,
                keyboard: .URL,
                monospaced: false
            )

            field(
                title: "Token",
                placeholder: "Pega el token",
                text: $model.pairingToken,
                keyboard: .default,
                monospaced: true
            )

            if let error {
                Text(error)
                    .font(.greek(12))
                    .foregroundStyle(Color.greekDanger)
            }

            Button(action: connect) {
                Text("Conectar")
                    .font(.greek(14, weight: .semibold))
                    .foregroundStyle(Color.greekBg)
                    .frame(maxWidth: .infinity, minHeight: 48)
                    .background(
                        RoundedRectangle(cornerRadius: 12, style: .continuous)
                            .fill(Color.greekText)
                    )
            }
            .disabled(!canConnect)
            .opacity(canConnect ? 1 : 0.4)
            .accessibilityLabel("Conectar con el PC")

            if model.isPaired {
                Button(role: .destructive, action: forget) {
                    Text("Olvidar este PC")
                        .font(.greek(13, weight: .medium))
                        .foregroundStyle(Color.greekDanger)
                        .frame(maxWidth: .infinity, minHeight: 44)
                }
            }

            Text("El token se guarda en el Llavero del iPhone. No se escribe en el código ni se envía a terceros.")
                .font(.greek(11))
                .foregroundStyle(Color.greekMuted)
                .lineSpacing(2)
        }
        .padding(20)
    }

    private var canConnect: Bool {
        !model.pairingURL.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            && !model.pairingToken.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    private func connect() {
        if let message = model.connectFromPairing() {
            error = message
        } else {
            error = nil
            dismiss()
        }
    }

    private func forget() {
        model.forgetPairing()
        error = nil
        dismiss()
    }

    private func field(
        title: String,
        placeholder: String,
        text: Binding<String>,
        keyboard: UIKeyboardType,
        monospaced: Bool
    ) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.greek(12))
                .foregroundStyle(Color.greekMuted)

            TextField(placeholder, text: text)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .keyboardType(keyboard)
                .font(monospaced ? .greekMono(13) : .greek(14))
                .foregroundStyle(Color.greekText)
                .tint(Color.greekText)
                .padding(.horizontal, 12)
                .frame(height: 46)
                .background(
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .fill(Color.greekSurface)
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .stroke(Color.greekBorderStrong, lineWidth: 1)
                )
        }
    }
}

struct PairingSheet: View {
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                PairingForm()
            }
            .background(Color.greekBg)
            .navigationTitle("Conectar al PC")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cerrar") {
                        dismiss()
                    }
                    .tint(Color.greekText)
                }
            }
        }
        .presentationDetents([.large])
        .presentationBackground(Color.greekBg)
        .preferredColorScheme(.dark)
    }
}
