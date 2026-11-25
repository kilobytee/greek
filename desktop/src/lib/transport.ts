import type {
	ClientCommand,
	ConnectionInfo,
	ConnectionStatus,
	ServerEvent,
	Snapshot
} from './types';

interface TransportHandlers {
	onSnapshot: (snapshot: Snapshot) => void;
	onStatus: (status: ConnectionStatus) => void;
	onError: (message: string) => void;
	onConnectionInfo?: (info: ConnectionInfo) => void;
}

export interface GreekTransport {
	send: (command: ClientCommand) => void;
	close: () => void;
	retry: () => void;
}

const DEV_BASE = (import.meta.env.VITE_GREEK_URL as string | undefined) ?? 'http://127.0.0.1:4319';

function isTauri(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function resolveConnection(): Promise<ConnectionInfo> {
	if (isTauri()) {
		const { invoke } = await import('@tauri-apps/api/core');
		return invoke<ConnectionInfo>('get_connection_info');
	}

	const response = await fetch(`${DEV_BASE}/api/dev/bootstrap`, { cache: 'no-store' });
	if (!response.ok) {
		throw new Error(
			`El núcleo no responde en ${DEV_BASE}. Inicia "npm run server" o abre la app de escritorio.`
		);
	}
	return (await response.json()) as ConnectionInfo;
}

function websocketUrl(info: ConnectionInfo): string {
	const base = info.url.replace(/\/$/, '').replace(/^http/, 'ws');
	const url = new URL(`${base}/ws`);
	url.searchParams.set('token', info.token);
	return url.toString();
}

export function startConnection(handlers: TransportHandlers): GreekTransport {
	let socket: WebSocket | null = null;
	let closed = false;
	let connecting = false;
	let generation = 0;
	let retryTimer: ReturnType<typeof setTimeout> | null = null;
	let heartbeat: ReturnType<typeof setInterval> | null = null;
	let attempts = 0;

	const clearTimers = () => {
		if (retryTimer) clearTimeout(retryTimer);
		if (heartbeat) clearInterval(heartbeat);
		retryTimer = null;
		heartbeat = null;
	};

	const scheduleRetry = (message?: string) => {
		if (closed) return;
		handlers.onStatus('offline');
		if (message) handlers.onError(message);
		const delay = Math.min(10_000, 1_500 * 2 ** Math.min(attempts, 3));
		attempts += 1;
		retryTimer = setTimeout(connect, delay);
	};

	const connect = async () => {
		if (closed || connecting) return;
		connecting = true;
		const token = ++generation;
		clearTimers();
		handlers.onStatus('connecting');

		let info: ConnectionInfo;
		try {
			info = await resolveConnection();
		} catch (error) {
			connecting = false;
			if (closed || token !== generation) return;
			scheduleRetry(error instanceof Error ? error.message : 'No se pudo conectar con el núcleo.');
			return;
		}

		connecting = false;
		if (closed || token !== generation) return;
		handlers.onConnectionInfo?.(info);

		let next: WebSocket;
		try {
			next = new WebSocket(websocketUrl(info));
		} catch {
			if (closed || token !== generation) return;
			scheduleRetry('La dirección del núcleo no es válida.');
			return;
		}

		if (socket && socket !== next) socket.close(1000, 'reemplazo');
		socket = next;
		const current = next;

		current.addEventListener('open', () => {
			if (closed || token !== generation || current !== socket) {
				current.close();
				return;
			}
			attempts = 0;
			handlers.onStatus('online');
			heartbeat = setInterval(() => {
				if (current.readyState === WebSocket.OPEN) {
					current.send(JSON.stringify({ type: 'ping' }));
				}
			}, 25_000);
		});

		current.addEventListener('message', (event) => {
			if (closed || token !== generation || current !== socket) return;
			let parsed: ServerEvent;
			try {
				parsed = JSON.parse(String(event.data)) as ServerEvent;
			} catch {
				return;
			}
			if (parsed.type === 'snapshot') handlers.onSnapshot(parsed.snapshot);
			if (parsed.type === 'error') handlers.onError(parsed.message);
		});

		current.addEventListener('close', (event) => {
			if (current !== socket) return;
			connecting = false;
			if (heartbeat) clearInterval(heartbeat);
			heartbeat = null;
			socket = null;
			if (closed) return;
			if (event.code === 4401) {
				handlers.onStatus('offline');
				handlers.onError('El token de conexión no es válido. Reinicia Greek y vuelve a emparejar.');
				return;
			}
			scheduleRetry();
		});

		current.addEventListener('error', () => {
			// El evento close se encarga de reintentar.
		});
	};

	const send = (command: ClientCommand) => {
		if (socket?.readyState === WebSocket.OPEN) {
			socket.send(JSON.stringify(command));
			return;
		}
		handlers.onError('Sin conexión con el núcleo. Reintentando.');
	};

	const close = () => {
		closed = true;
		connecting = false;
		generation += 1;
		clearTimers();
		socket?.close(1000, 'cierre');
		socket = null;
	};

	const retry = () => {
		clearTimers();
		connecting = false;
		generation += 1;
		attempts = 0;
		if (!closed) void connect();
	};

	void connect();

	return { send, close, retry };
}
