import SwiftUI

struct ChatHeader: View {
    let title: String
    let modelLabel: String
    let onMenu: () -> Void
    let onModel: () -> Void

    var body: some View {
        ZStack {
            Text(title)
                .font(.greek(15, weight: .semibold))
                .foregroundStyle(Color.greekText)
                .lineLimit(1)
                .padding(.horizontal, 92)

            HStack(spacing: 8) {
                Button(action: onMenu) {
                    Image(systemName: "line.3.horizontal")
                        .font(.system(size: 16, weight: .medium))
                        .foregroundStyle(Color.greekText)
                        .frame(width: 44, height: 44)
                        .contentShape(Rectangle())
                }
                .accessibilityLabel("Menú")

                Spacer(minLength: 0)

                Button(action: onModel) {
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
                    .background(Capsule().fill(Color.greekSurface))
                    .overlay(Capsule().stroke(Color.greekBorderStrong, lineWidth: 1))
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
