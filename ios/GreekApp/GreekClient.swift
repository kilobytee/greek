import Foundation

@MainActor
final class GreekClient {
    var onSnapshot: ((Snapshot) -> Void)?
    var onServerError: ((String) -> Void)?
    var onStatus: ((ConnectionStatus) -> Void)?

    private let session: URLSession
    private var task: URLSessionWebSocketTask?
    private var receiveTask: Task<Void, Never>?
    private var reconnectTask: Task<Void, Never>?
    private var config: PairingConfig?
    private var attempts = 0
    private var stopped = true

    init() {
        let configuration = URLSessionConfiguration.default
        configuration.waitsForConnectivity = false
        configuration.timeoutIntervalForRequest = 15
        session = URLSession(configuration: configuration)
    }

    func connect(_ config: PairingConfig) {
        stopKeepingConfig()
        stopped = false
        self.config = config

        guard let url = websocketURL(from: config) else {
            onStatus?(.offline)
            onServerError?("La dirección no es válida. Usa el formato http://192.168.1.10:4319.")
            return
        }

        onStatus?(.connecting)
        let task = session.webSocketTask(with: url)
        self.task = task
        task.resume()
        startReceiving()
    }

    func retry() {
        guard let config else { return }
        attempts = 0
        connect(config)
    }

    func disconnect() {
        stopped = true
        receiveTask?.cancel()
        receiveTask = nil
        reconnectTask?.cancel()
        reconnectTask = nil
        task?.cancel(with: .goingAway, reason: nil)
        task = nil
        config = nil
        attempts = 0
        onStatus?(.offline)
    }

    func send(_ payload: [String: Any]) {
        guard let task else { return }
        guard let data = try? JSONSerialization.data(withJSONObject: payload),
              let text = String(data: data, encoding: .utf8)
        else {
            return
        }

        Task {
            do {
                try await task.send(.string(text))
            } catch {
                self.onServerError?("No se pudo enviar el mensaje al PC.")
            }
        }
    }

    private func stopKeepingConfig() {
        stopped = true
        receiveTask?.cancel()
        receiveTask = nil
        reconnectTask?.cancel()
        reconnectTask = nil
        task?.cancel(with: .goingAway, reason: nil)
        task = nil
    }

    private func websocketURL(from config: PairingConfig) -> URL? {
        let trimmed = config.urlString.trimmingCharacters(in: .whitespacesAndNewlines)
        guard var components = URLComponents(string: trimmed),
              let host = components.host,
              !host.isEmpty
        else {
            return nil
        }

        components.scheme = components.scheme == "https" ? "wss" : "ws"
        components.path = "/ws"
        components.queryItems = [URLQueryItem(name: "token", value: config.token)]
        return components.url
    }

    private func startReceiving() {
        receiveTask?.cancel()
        receiveTask = Task { [weak self] in
            guard let self else { return }
            while !Task.isCancelled {
                guard let task = self.task else { return }
                do {
                    let message = try await task.receive()
                    switch message {
                    case .string(let text):
                        self.handle(text)
                    case .data(let data):
                        if let text = String(data: data, encoding: .utf8) {
                            self.handle(text)
                        }
                    @unknown default:
                        break
                    }
                } catch {
                    if Task.isCancelled { return }
                    self.fail()
                    return
                }
            }
        }
    }

    private func handle(_ text: String) {
        guard let data = text.data(using: .utf8),
              let envelope = try? JSONDecoder().decode(Envelope.self, from: data)
        else {
            return
        }

        switch envelope.type {
        case "snapshot":
            if let snapshot = envelope.snapshot {
                attempts = 0
                onStatus?(.online)
                onSnapshot?(snapshot)
            }
        case "error":
            onServerError?(envelope.message ?? "El núcleo devolvió un error.")
        default:
            break
        }
    }

    private func fail() {
        guard !stopped else { return }
        task?.cancel(with: .goingAway, reason: nil)
        task = nil
        onStatus?(.offline)

        guard let config else { return }
        attempts += 1
        guard attempts <= 6 else {
            onServerError?("No se pudo reconectar. Revisa que el PC esté encendido y en la misma red Wi-Fi.")
            return
        }

        reconnectTask?.cancel()
        reconnectTask = Task { [weak self] in
            guard let self else { return }
            let seconds = min(2.0 * Double(self.attempts), 10.0)
            try? await Task.sleep(nanoseconds: UInt64(seconds * 1_000_000_000))
            guard !Task.isCancelled else { return }
            self.connect(config)
        }
    }

    private struct Envelope: Decodable {
        let type: String
        let snapshot: Snapshot?
        let message: String?
    }
}
