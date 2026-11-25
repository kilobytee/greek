import SwiftUI

// Liquid Glass con degradacion al diseno plano en iOS 17 y 18.

private struct GreekSurfaceModifier: ViewModifier {
    let cornerRadius: CGFloat

    func body(content: Content) -> some View {
        if #available(iOS 26, *) {
            content.glassEffect(in: .rect(cornerRadius: cornerRadius))
        } else {
            content
                .background(
                    RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
                        .fill(Color.greekSurface)
                )
                .overlay(
                    RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
                        .stroke(Color.greekBorder, lineWidth: 1)
                )
        }
    }
}

private struct GreekCapsuleModifier: ViewModifier {
    func body(content: Content) -> some View {
        if #available(iOS 26, *) {
            content.glassEffect()
        } else {
            content
                .background(Capsule().fill(Color.greekSurface))
                .overlay(Capsule().stroke(Color.greekBorderStrong, lineWidth: 1))
        }
    }
}

private struct GreekSendSurfaceModifier: ViewModifier {
    func body(content: Content) -> some View {
        if #available(iOS 26, *) {
            content.glassEffect(in: .circle)
        } else {
            content.background(Circle().fill(Color.greekText))
        }
    }
}

private struct GreekButtonModifier: ViewModifier {
    let prominent: Bool

    func body(content: Content) -> some View {
        if #available(iOS 26, *) {
            if prominent {
                content.buttonStyle(GlassProminentButtonStyle())
            } else {
                content.buttonStyle(GlassButtonStyle())
            }
        } else {
            content
        }
    }
}

extension View {
    func greekSurface(cornerRadius: CGFloat = 16) -> some View {
        modifier(GreekSurfaceModifier(cornerRadius: cornerRadius))
    }

    func greekCapsuleSurface() -> some View {
        modifier(GreekCapsuleModifier())
    }

    func greekSendSurface() -> some View {
        modifier(GreekSendSurfaceModifier())
    }

    func greekGlassButton(prominent: Bool = false) -> some View {
        modifier(GreekButtonModifier(prominent: prominent))
    }
}
