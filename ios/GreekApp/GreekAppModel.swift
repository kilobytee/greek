import Foundation

@MainActor
final class GreekAppModel: ObservableObject {
    @Published private(set) var snapshot: Snapshot?
    @Published private(set) var status: ConnectionStatus = .offline
    @Published var draft = ""
    @Published var errorMessage: String?
    @Published var isPairingPresented = false
    @Published var isMenuPresented = false
    @Published var isScreenPresented = false
    @Published var pairingURL: String
    @Published var pairingToken: String

    private let client = GreekClient()
    private let defaults = UserDefaults.standard
    private static let urlKey = "greek.server.url"
    private var pendingText: String?

    init() {
        pairingURL = UserDefaults.standard.string(forKey: Self.urlKey) ?? ""
        pairingToken = KeychainStore.loadToken() ?? ""

        client.onStatus = { [weak self] status in
            self?.status = status
        }
        client.onSnapshot = { [weak self] snapshot in
            guard let self else { return }
            self.snapshot = snapshot
            if let pending = self.pendingText,
               let chat = snapshot.chats.first(where: { $0.id == snapshot.activeChatId }),
               chat.messages.contains(where: { $0.role == .user && $0.text == pending }) {
                self.draft = ""
                self.pendingText = nil
            }
        }
        client.onServerError = { [weak self] message in
            guard let self else { return }
            if let pending = self.pendingText {
                self.draft = pending
                self.pendingText = nil
            }
            self.errorMessage = message
        }
        client.onAuthError = { [weak self] message in
            self?.errorMessage = message
        }

        if !pairingURL.isEmpty && !pairingToken.isEmpty {
            client.connect(PairingConfig(urlString: pairingURL, token: pairingToken))
        }
    }

    var activeChat: Chat? {
        guard let snapshot else { return nil }
        return snapshot.chats.first(where: { $0.id == snapshot.activeChatId }) ?? snapshot.chats.first
    }

    var isThinking: Bool {
        guard let snapshot, let thinking = snapshot.thinkingChatId else { return false }
        return thinking == snapshot.activeChatId
    }

    var canSend: Bool {
        status == .online && !draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    var isPaired: Bool {
        !pairingURL.isEmpty && !pairingToken.isEmpty
    }

    var activeModelLabel: String {
        guard let chat = activeChat else { return "Auto" }
        return snapshot?.models.first(where: { $0.id == chat.model })?.label ?? "Auto"
    }

    var availableModels: [ModelOption] {
        snapshot?.models.filter { $0.available } ?? []
    }

    @discardableResult
    func connectFromPairing() -> String? {
        let url = pairingURL.trimmingCharacters(in: .whitespacesAndNewlines)
        let token = pairingToken.trimmingCharacters(in: .whitespacesAndNewlines)

        guard let components = URLComponents(string: url),
              let host = components.host,
              !host.isEmpty
        else {
            return "Escribe una dirección válida, por ejemplo http://192.168.1.10:4319."
        }

        guard !token.isEmpty else {
            return "Pega el token que muestra el escritorio."
        }

        guard KeychainStore.saveToken(token) else {
            return "No se pudo guardar el token en el Llavero."
        }

        pairingURL = url
        pairingToken = token
        defaults.set(url, forKey: Self.urlKey)
        client.connect(PairingConfig(urlString: url, token: token))
        return nil
    }

    func retry() {
        client.retry()
    }

    func disconnect() {
        client.disconnect()
        snapshot = nil
        errorMessage = nil
        pendingText = nil
    }

    func forgetPairing() {
        client.disconnect()
        snapshot = nil
        errorMessage = nil
        pendingText = nil
        defaults.removeObject(forKey: Self.urlKey)
        KeychainStore.deleteToken()
        pairingURL = ""
        pairingToken = ""
    }

    func send() {
        let text = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }
        guard status == .online else {
            errorMessage = "El PC no está conectado."
            return
        }

        pendingText = text
        client.send(["type": "sendMessage", "text": text])
    }

    func newChat() {
        guard requireOnline() else { return }
        client.send(["type": "newChat"])
    }

    func selectChat(_ id: String) {
        guard requireOnline() else { return }
        client.send(["type": "selectChat", "chatId": id])
    }

    func setModel(_ id: String) {
        guard requireOnline() else { return }
        client.send(["type": "setModel", "model": id])
    }

    func openScreen() {
        isScreenPresented = true
    }

    private func requireOnline() -> Bool {
        guard status == .online else {
            errorMessage = "El PC no está conectado."
            return false
        }
        return true
    }
}
