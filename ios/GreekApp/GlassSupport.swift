import SwiftUI

// Liquid Glass solo en entradas y controles de seleccion.
// En iOS 17 y 18 se mantiene el diseno plano original.

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

private struct GreekSelectionModifier: ViewModifier {
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

extension View {
    func greekInputSurface(cornerRadius: CGFloat = 16) -> some View {
        modifier(GreekSurfaceModifier(cornerRadius: cornerRadius))
    }

    func greekSelectionSurface() -> some View {
        modifier(GreekSelectionModifier())
    }
}
