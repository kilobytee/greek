import SwiftUI

struct StatusCard: View {
    let status: ConnectionStatus
    let desktop: VirtualDesktop?
    let isPaired: Bool
    let onShowScreen: () -> Void
    let onConnect: () -> Void
    let onRetry: () -> Void

    private var statusColor: Color {
        switch status {
        case .online: return .greekOnline
        case .connecting: return .greekWarning
        case .offline: return .greekMuted
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 6) {
                Circle()
                    .fill(statusColor)
                    .frame(width: 7, height: 7)

                Text("Tu PC")
                    .font(.greek(13, weight: .semibold))
                    .foregroundStyle(Color.greekText)

                Text("· \(status.label)")
                    .font(.greek(12))
                    .foregroundStyle(Color.greekMuted)

                Spacer(minLength: 8)

                if status == .offline {
                    Button(action: isPaired ? onRetry : onConnect) {
                        Text(isPaired ? "Reconectar" : "Conectar")
                            .font(.greek(12, weight: .medium))
                            .foregroundStyle(Color.greekText)
                            .padding(.horizontal, 10)
                            .frame(minHeight: 44)
                            .background(Capsule().fill(Color.greekActive))
                    }
                    .accessibilityLabel(isPaired ? "Reconectar con el PC" : "Conectar con el PC")
                    .greekGlassButton()
                }
            }

            ZStack {
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(Color.greekBg)

                if let desktop {
                    VirtualDesktopView(desktop: desktop)
                        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                } else {
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .strokeBorder(
                            Color.greekBorderStrong,
                            style: StrokeStyle(lineWidth: 1, dash: [4, 4])
                        )
                    VStack(spacing: 5) {
                        Image(systemName: "desktopcomputer")
                            .font(.system(size: 18, weight: .light))
                        Text(status == .connecting ? "Conectando" : "Sin conexión")
                            .font(.greek(9))
                    }
                    .foregroundStyle(Color.greekMuted)
                }
            }
            .frame(height: 132)
            .overlay(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .stroke(Color.greekBorderStrong, lineWidth: 1)
            )

            Button(action: onShowScreen) {
                HStack(spacing: 6) {
                    Image(systemName: "rectangle.on.rectangle")
                        .font(.system(size: 12, weight: .medium))
                    Text("Ver pantalla")
                        .font(.greek(13, weight: .medium))
                }
                .foregroundStyle(desktop == nil ? Color.greekMuted : Color.greekText)
                .frame(maxWidth: .infinity, minHeight: 44)
                .greekSurface(cornerRadius: 9)
            }
            .disabled(desktop == nil)
            .accessibilityLabel("Ver la pantalla del escritorio virtual")
            .greekGlassButton()
        }
        .padding(14)
        .greekSurface(cornerRadius: 16)
    }
}
