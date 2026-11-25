export type Role = 'user' | 'assistant';
export type WindowKind = 'browser' | 'files' | 'terminal';
export type ConnectionStatus = 'connecting' | 'online' | 'offline';

export interface ToolChip {
	label: string;
	detail?: string | null;
}

export interface ChatMessage {
	id: string;
	role: Role;
	text: string;
	createdAt: number;
	tools: ToolChip[];
}

export interface Chat {
	id: string;
	title: string;
	createdAt: number;
	updatedAt: number;
	model: string;
	routeReason: string;
	tools: string[];
	messages: ChatMessage[];
}

export interface DesktopFile {
	name: string;
	size: number;
	updatedAt: number;
	content: string;
}

export interface BrowserState {
	title: string;
	url: string;
	status: number;
	lines: string[];
}

export interface VirtualWindow {
	id: string;
	kind: WindowKind;
	title: string;
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface VirtualDesktop {
	windows: VirtualWindow[];
	activeWindowId: string | null;
	files: DesktopFile[];
	selectedFile: string | null;
	browser: BrowserState | null;
	terminal: string[];
}

export interface ModelOption {
	id: string;
	label: string;
	provider: string;
	available: boolean;
	detail: string;
}

export interface ServerInfo {
	online: boolean;
	bind: string;
	lanUrl: string;
	version: string;
}

export interface Snapshot {
	schemaVersion: number;
	chats: Chat[];
	activeChatId: string;
	thinkingChatId: string | null;
	desktop: VirtualDesktop;
	models: ModelOption[];
	server: ServerInfo;
}

export interface ConnectionInfo {
	url: string;
	lanUrl: string;
	token: string;
	port: number;
	bind: string;
}

export type ClientCommand =
	| { type: 'newChat' }
	| { type: 'selectChat'; chatId: string }
	| { type: 'sendMessage'; text: string; model?: string | null }
	| { type: 'setModel'; model: string }
	| { type: 'desktopOpen'; app: 'browser' | 'files' }
	| { type: 'desktopClose'; windowId: string }
	| { type: 'desktopFocus'; windowId: string }
	| { type: 'desktopSelectFile'; file: string };

export type ServerEvent =
	| { type: 'snapshot'; snapshot: Snapshot }
	| { type: 'error'; message: string }
	| { type: 'pong' };
