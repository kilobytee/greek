import SwiftUI

struct RootView: View {
    @EnvironmentObject private var model: GreekAppModel
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    var body: some View {
        ZStack {
            Color.greekBg.ignoresSafeArea()

            VStack(spacing: 0) {
                ChatHeader(
                    title: headerTitle,
                    modelLabel: model.activeModelLabel,
                    models: model.availableModels,
                    selectedModel: model.activeChat?.model ?? "auto",
                    onNewChat: model.newChat,
                    onOpenMenu: { model.isMenuPresented = true },
                    onPair: { model.isPairingPresented = true },
                    onModelSelect: model.setModel
                )

                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(spacing: 18) {
                            StatusCard(
                                status: model.status,
                                desktop: model.snapshot?.desktop,
                                isPaired: model.isPaired,
                                onShowScreen: model.openScreen,
                                onConnect: { model.isPairingPresented = true },
                                onRetry: model.retry
                            )

                            if let chat = model.activeChat, !chat.messages.isEmpty {
                                ForEach(chat.messages) { message in
                                    MessageBubble(message: message)
                                }
                            } else if model.status == .online {
                                EmptyChatView { suggestion in
                                    model.draft = suggestion
                                }
                            }

                            if model.isThinking {
                                TypingIndicator()
                            }

                            Color.clear
                                .frame(height: 1)
                                .id("bottom")
                        }
                        .padding(.horizontal, 16)
                        .padding(.top, 12)
                        .padding(.bottom, 4)
                    }
                    .scrollDismissesKeyboard(.interactively)
                    .onChange(of: model.activeChat?.messages.count) { _, _ in
                        scrollToBottom(proxy)
                    }
                    .onChange(of: model.isThinking) { _, _ in
                        scrollToBottom(proxy)
                    }
                    .onChange(of: model.snapshot?.activeChatId) { _, _ in
                        scrollToBottom(proxy)
                    }
                }
            }
        }
        .safeAreaInset(edge: .bottom) {
            ComposerBar(text: $model.draft, canSend: model.canSend, onSend: model.send)
                .background(Color.greekBg)
        }
        .sheet(isPresented: $model.isMenuPresented) {
            MenuSheet()
                .environmentObject(model)
        }
        .sheet(isPresented: $model.isPairingPresented) {
            PairingSheet()
                .environmentObject(model)
        }
        .fullScreenCover(isPresented: $model.isScreenPresented) {
            ScreenViewer(
                desktop: model.snapshot?.desktop,
                server: model.snapshot?.server,
                onClose: { model.isScreenPresented = false }
            )
        }
        .alert("Greek", isPresented: errorBinding) {
            Button("Entendido", role: .cancel) {
                model.errorMessage = nil
            }
        } message: {
            Text(model.errorMessage ?? "")
        }
        .onChange(of: scenePhase) { _, phase in
            if phase == .active, model.status == .offline, model.isPaired {
                model.retry()
            }
        }
    }

    private var headerTitle: String {
        guard let title = model.activeChat?.title, !title.isEmpty else { return "Greek" }
        return title
    }

    private var errorBinding: Binding<Bool> {
        Binding(
            get: { model.errorMessage != nil },
            set: { isPresented in
                if !isPresented {
                    model.errorMessage = nil
                }
            }
        )
    }

    private func scrollToBottom(_ proxy: ScrollViewProxy) {
        let animation: Animation? = reduceMotion ? nil : .easeOut(duration: 0.2)
        withAnimation(animation) {
            proxy.scrollTo("bottom", anchor: .bottom)
        }
    }
}

private struct EmptyChatView: View {
    let onSuggestion: (String) -> Void

    private let suggestions = [
        "Busca el clima de Bogotá",
        "Crea una nota llamada ideas",
        "Ejecuta date"
    ]

    var body: some View {
        VStack(spacing: 14) {
            Text("¿Qué necesitas?")
                .font(.greek(16, weight: .medium))
                .foregroundStyle(Color.greekText)

            VStack(spacing: 8) {
                ForEach(suggestions, id: \.self) { suggestion in
                    Button {
                        onSuggestion(suggestion)
                    } label: {
                        Text(suggestion)
                            .font(.greek(13))
                            .foregroundStyle(Color.greekText)
                            .frame(maxWidth: .infinity, minHeight: 44)
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
        }
        .padding(.top, 8)
    }
}
