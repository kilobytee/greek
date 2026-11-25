import Foundation

enum Role: String, Decodable {
    case user
    case assistant
}

enum WindowKind: String, Decodable {
    case browser
    case files
    case terminal
}

enum ConnectionStatus: Equatable {
    case connecting
    case online
    case offline

    var label: String {
        switch self {
        case .connecting: return "conectando"
        case .online: return "en línea"
        case .offline: return "sin conexión"
        }
    }
}

struct ToolChip: Decodable, Identifiable {
    let label: String
    let detail: String?

    var id: String { "\(label)-\(detail ?? "")" }
}

struct ChatMessage: Decodable, Identifiable {
    let id: String
    let role: Role
    let text: String
    let createdAt: Int
    let tools: [ToolChip]
}

struct Chat: Decodable, Identifiable {
    let id: String
    let title: String
    let createdAt: Int
    let updatedAt: Int
    let model: String
    let routeReason: String
    let tools: [String]
    let messages: [ChatMessage]
}

struct DesktopFile: Decodable, Identifiable {
    let name: String
    let size: Int
    let updatedAt: Int
    let content: String

    var id: String { name }
}

struct BrowserState: Decodable {
    let title: String
    let url: String
    let status: Int
    let lines: [String]
}

struct VirtualWindow: Decodable, Identifiable {
    let id: String
    let kind: WindowKind
    let title: String
    let x: Double
    let y: Double
    let width: Double
    let height: Double
}

struct VirtualDesktop: Decodable {
    let windows: [VirtualWindow]
    let activeWindowId: String?
    let files: [DesktopFile]
    let selectedFile: String?
    let browser: BrowserState?
    let terminal: [String]
}

struct ModelOption: Decodable, Identifiable {
    let id: String
    let label: String
    let provider: String
    let available: Bool
    let detail: String
}

struct ServerInfo: Decodable {
    let online: Bool
    let bind: String
    let lanUrl: String
    let version: String
}

struct Snapshot: Decodable {
    let schemaVersion: Int
    let chats: [Chat]
    let activeChatId: String
    let thinkingChatId: String?
    let desktop: VirtualDesktop
    let models: [ModelOption]
    let server: ServerInfo
}

struct PairingConfig: Equatable {
    let urlString: String
    let token: String
}
