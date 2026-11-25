import SwiftUI

struct ScreenViewer: View {
    let desktop: VirtualDesktop?
    let server: ServerInfo?
    let onClose: () -> Void

    var body: some View {
        ZStack {
            Color.greekBg.ignoresSafeArea()

            VStack(spacing: 0) {
                HStack(alignment: .center) {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("Escritorio virtual")
                            .font(.greek(16, weight: .semibold))
                            .foregroundStyle(Color.greekText)

                        Text(subtitle)
                            .font(.greek(11))
                            .foregroundStyle(Color.greekMuted)
                            .lineLimit(1)
                    }

                    Spacer(minLength: 12)

                    Button(action: onClose) {
                        Image(systemName: "xmark")
                            .font(.system(size: 13, weight: .semibold))
                            .foregroundStyle(Color.greekText)
                            .frame(width: 44, height: 44)
                            .background(Circle().fill(Color.greekSurface))
                            .overlay(Circle().stroke(Color.greekBorderStrong, lineWidth: 1))
                    }
                    .accessibilityLabel("Cerrar")
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 10)

                Spacer(minLength: 8)

                VirtualDesktopView(desktop: desktop, detailed: true)
                    .aspectRatio(1.5, contentMode: .fit)
                    .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
                    .overlay(
                        RoundedRectangle(cornerRadius: 14, style: .continuous)
                            .stroke(Color.greekBorderStrong, lineWidth: 1)
                    )
                    .padding(.horizontal, 16)

                Spacer(minLength: 8)

                Text("Vista del escritorio virtual de Greek. No es la pantalla real del PC.")
                    .font(.greek(11))
                    .foregroundStyle(Color.greekMuted)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, 24)
                    .padding(.bottom, 20)
            }
        }
    }

    private var subtitle: String {
        guard let server else { return "Sin conexión" }
        return "\(server.bind) · Greek \(server.version)"
    }
}
