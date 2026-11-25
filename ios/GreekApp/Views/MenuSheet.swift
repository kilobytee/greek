import SwiftUI

struct MenuSheet: View {
    @EnvironmentObject private var model: GreekAppModel
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                Section("Chats") {
                    if let chats = model.snapshot?.chats, !chats.isEmpty {
                        ForEach(chats) { chat in
                            Button {
                                model.selectChat(chat.id)
                                dismiss()
                            } label: {
                                HStack {
                                    Text(chat.title)
                                        .font(.greek(14))
                                        .foregroundStyle(Color.greekText)
                                        .lineLimit(1)

                                    Spacer()

                                    if chat.id == model.snapshot?.activeChatId {
                                        Image(systemName: "checkmark")
                                            .font(.system(size: 12, weight: .semibold))
                                            .foregroundStyle(Color.greekMuted)
                                    }
                                }
                                .frame(minHeight: 44)
                            }
                        }
                    } else {
                        Text("Sin chats todavía")
                            .font(.greek(13))
                            .foregroundStyle(Color.greekMuted)
                    }
                }

                Section("Modelo") {
                    if let models = model.snapshot?.models, !models.isEmpty {
                        ForEach(models) { option in
                            Button {
                                model.setModel(option.id)
                                dismiss()
                            } label: {
                                HStack {
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(option.label)
                                            .font(.greek(14))
                                            .foregroundStyle(option.available ? Color.greekText : Color.greekMuted)
                                        Text(option.detail)
                                            .font(.greek(11))
                                            .foregroundStyle(Color.greekMuted)
                                            .lineLimit(1)
                                    }

                                    Spacer()

                                    if option.id == model.activeChat?.model {
                                        Image(systemName: "checkmark")
                                            .font(.system(size: 12, weight: .semibold))
                                            .foregroundStyle(Color.greekMuted)
                                    }
                                }
                                .frame(minHeight: 44)
                            }
                            .disabled(!option.available)
                        }
                    } else {
                        Text("Conecta el PC para ver los modelos")
                            .font(.greek(13))
                            .foregroundStyle(Color.greekMuted)
                    }
                }

                Section {
                    Button {
                        model.newChat()
                        dismiss()
                    } label: {
                        Label("Nuevo chat", systemImage: "square.and.pencil")
                            .font(.greek(14))
                            .foregroundStyle(Color.greekText)
                            .frame(minHeight: 44)
                    }

                    NavigationLink {
                        PairingForm()
                            .environmentObject(model)
                    } label: {
                        Label("Conectar al PC", systemImage: "wifi")
                            .font(.greek(14))
                            .foregroundStyle(Color.greekText)
                            .frame(minHeight: 44)
                    }
                }

                Section("Conexión") {
                    HStack(spacing: 7) {
                        Circle()
                            .fill(statusColor)
                            .frame(width: 7, height: 7)
                        Text(model.status.label)
                            .font(.greek(13))
                            .foregroundStyle(Color.greekText)
                    }
                    .frame(minHeight: 36)

                    Button("Reintentar conexión") {
                        model.retry()
                    }
                    .font(.greek(13))
                    .foregroundStyle(Color.greekText)
                    .frame(minHeight: 44)
                }
            }
            .listStyle(.plain)
            .scrollContentBackground(.hidden)
            .background(Color.greekBg)
            .navigationTitle("Menú")
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
        .greekGlassButton()
        .presentationDetents([.medium, .large])
        .presentationBackground(Color.greekBg)
        .preferredColorScheme(.dark)
    }

    private var statusColor: Color {
        switch model.status {
        case .online: return .greekOnline
        case .connecting: return .greekWarning
        case .offline: return .greekMuted
        }
    }
}
