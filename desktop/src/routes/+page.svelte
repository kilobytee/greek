<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { AlertCircle, LoaderCircle, RefreshCw } from '@lucide/svelte';
	import type {
		ConnectionInfo,
		ConnectionStatus,
		Snapshot,
		VirtualDesktop
	} from '#lib/types';
	import { prefersReducedMotion } from '#lib/format';
	import { startConnection, type GreekTransport } from '#lib/transport';
	import Sidebar from '#lib/components/Sidebar.svelte';
	import ChatHeader from '#lib/components/ChatHeader.svelte';
	import MessageBubble from '#lib/components/MessageBubble.svelte';
	import TypingIndicator from '#lib/components/TypingIndicator.svelte';
	import Composer from '#lib/components/Composer.svelte';
	import ActivityPanel from '#lib/components/ActivityPanel.svelte';
	import ScreenViewer from '#lib/components/ScreenViewer.svelte';
	import PairingDialog from '#lib/components/PairingDialog.svelte';

	const emptyDesktop: VirtualDesktop = {
		windows: [],
		activeWindowId: null,
		files: [],
		selectedFile: null,
		browser: null,
		terminal: []
	};

	let snapshot = $state<Snapshot | null>(null);
	let status = $state<ConnectionStatus>('connecting');
	let errorMessage = $state('');
	let connectionInfo = $state<ConnectionInfo | null>(null);
	let draft = $state('');
	let now = $state(Date.now());
	let screenOpen = $state(false);
	let pairingOpen = $state(false);
	let messagesEnd = $state<HTMLDivElement>();
	let transport: GreekTransport | undefined;

	const activeChat = $derived(
		snapshot?.chats.find((chat) => chat.id === snapshot?.activeChatId) ?? null
	);
	const desktop = $derived(snapshot?.desktop ?? emptyDesktop);
	const thinking = $derived(
		!!snapshot && snapshot.thinkingChatId !== null && snapshot.thinkingChatId === snapshot.activeChatId
	);
	const selectedModel = $derived(activeChat?.model ?? 'auto');
	const models = $derived(snapshot?.models ?? []);
	const canSend = $derived(status === 'online' && !thinking);
	let lastSignature = '';

	onMount(() => {
		const clock = setInterval(() => (now = Date.now()), 30_000);

		transport = startConnection({
			onSnapshot: (next) => {
				snapshot = next;
			},
			onStatus: (next) => {
				status = next;
				if (next === 'online') errorMessage = '';
			},
			onError: (message) => {
				errorMessage = message;
			},
			onConnectionInfo: (info) => {
				connectionInfo = info;
			}
		});

		const onKeydown = (event: KeyboardEvent) => {
			if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'n') {
				event.preventDefault();
				screenOpen = false;
				pairingOpen = false;
				newChat();
			}
			if (event.key === 'Escape') {
				screenOpen = false;
				pairingOpen = false;
			}
		};
		window.addEventListener('keydown', onKeydown);

		return () => {
			clearInterval(clock);
			window.removeEventListener('keydown', onKeydown);
			transport?.close();
		};
	});

	$effect(() => {
		if (status === 'online' && errorMessage) {
			const timer = setTimeout(() => (errorMessage = ''), 6000);
			return () => clearTimeout(timer);
		}
	});

	$effect(() => {
		const signature = `${snapshot?.activeChatId ?? ''}:${activeChat?.messages.length ?? 0}:${thinking}`;
		if (signature === lastSignature) return;
		lastSignature = signature;
		void tick().then(() => {
			messagesEnd?.scrollIntoView({
				behavior: prefersReducedMotion() ? 'auto' : 'smooth',
				block: 'end'
			});
		});
	});

	async function focusComposer() {
		await tick();
		document.querySelector<HTMLTextAreaElement>('.composer textarea')?.focus();
	}

	function requireConnection(): boolean {
		if (status === 'online') return true;
		errorMessage = 'Sin conexión con el núcleo.';
		return false;
	}

	function newChat() {
		if (!requireConnection()) return;
		draft = '';
		transport?.send({ type: 'newChat' });
		void focusComposer();
	}

	function selectChat(id: string) {
		if (!requireConnection() || id === snapshot?.activeChatId) return;
		draft = '';
		transport?.send({ type: 'selectChat', chatId: id });
		void focusComposer();
	}

	function sendMessage() {
		const text = draft.trim();
		if (!text || !canSend || !requireConnection()) return;
		transport?.send({ type: 'sendMessage', text, model: selectedModel });
		draft = '';
		void focusComposer();
	}

	function changeModel(model: string) {
		if (!requireConnection()) return;
		transport?.send({ type: 'setModel', model });
	}

	function openDesktopApp(app: 'browser' | 'files') {
		if (!requireConnection()) return;
		transport?.send({ type: 'desktopOpen', app });
	}

	function closeDesktopWindow(windowId: string) {
		transport?.send({ type: 'desktopClose', windowId });
	}

	function focusDesktopWindow(windowId: string) {
		transport?.send({ type: 'desktopFocus', windowId });
	}

	function selectDesktopFile(file: string) {
		transport?.send({ type: 'desktopSelectFile', file });
	}

	function useSuggestion(text: string) {
		draft = text;
		void focusComposer();
	}

	function retry() {
		errorMessage = '';
		transport?.retry();
	}
</script>

<div class="app">
	<Sidebar
		chats={snapshot?.chats ?? []}
		activeChatId={snapshot?.activeChatId ?? ''}
		{now}
		onNewChat={newChat}
		onSelectChat={selectChat}
	/>

	<main class="center">
		<ChatHeader title={activeChat?.title ?? 'Greek'} />

		{#if !snapshot && status === 'connecting'}
			<div class="state">
				<span class="spin">
					<LoaderCircle size={20} strokeWidth={1.6} aria-hidden="true" />
				</span>
				<p>Conectando con Greek...</p>
				<span>El núcleo se está iniciando en este equipo.</span>
			</div>
		{:else if !snapshot}
			<div class="state error-state">
				<AlertCircle size={22} strokeWidth={1.6} aria-hidden="true" />
				<p>No se pudo conectar con el núcleo</p>
				<span>{errorMessage || 'Revisa que el servidor local esté activo.'}</span>
				<button type="button" class="retry-button" onclick={retry}>
					<RefreshCw size={14} strokeWidth={1.8} aria-hidden="true" />
					Reintentar
				</button>
			</div>
		{:else}
			<div class="messages-scroll">
				<div class="messages">
					{#if !activeChat || activeChat.messages.length === 0}
						<div class="empty-chat">
							<p>Escribe un mensaje para empezar.</p>
							<span>Puedo consultar el clima y guardar archivos en el escritorio virtual.</span>
							<div class="suggestions">
								<button
									type="button"
									onclick={() => useSuggestion('Busca el clima de Bogotá y guárdalo en un archivo')}
								>
									Busca el clima de Bogotá y guárdalo en un archivo
								</button>
								<button type="button" onclick={() => useSuggestion('Abre el navegador')}>
									Abre el navegador
								</button>
							</div>
						</div>
					{/if}

					{#each activeChat?.messages ?? [] as message (message.id)}
						<MessageBubble {message} />
					{/each}

					{#if thinking}
						<TypingIndicator />
					{/if}
					<div class="messages-end" bind:this={messagesEnd}></div>
				</div>
			</div>

			<div class="composer-area">
				{#if status !== 'online'}
					<div class="connection-banner" role="status">
						<span class="banner-dot" aria-hidden="true"></span>
						<span>
							{status === 'connecting'
								? 'Reconectando con el núcleo...'
								: errorMessage || 'Sin conexión con el núcleo.'}
						</span>
						{#if status === 'offline'}
							<button type="button" onclick={retry}>Reintentar</button>
						{/if}
					</div>
				{:else if errorMessage}
					<div class="connection-banner warning" role="alert">
						<AlertCircle size={14} strokeWidth={1.8} aria-hidden="true" />
						<span>{errorMessage}</span>
						<button type="button" onclick={() => (errorMessage = '')}>Descartar</button>
					</div>
				{/if}

				<Composer
					value={draft}
					model={selectedModel}
					{models}
					{thinking}
					disabled={status !== 'online'}
					onValueChange={(value) => (draft = value)}
					onSend={sendMessage}
					onModelChange={changeModel}
				/>
			</div>
		{/if}
	</main>

	<div class="activity-slot">
		<ActivityPanel
			chat={activeChat}
			{desktop}
			server={snapshot?.server ?? null}
			onOpenScreen={() => (screenOpen = true)}
			onOpenPairing={() => (pairingOpen = true)}
		/>
	</div>
</div>

{#if screenOpen}
	<ScreenViewer
		{desktop}
		onClose={() => (screenOpen = false)}
		onOpenApp={openDesktopApp}
		onCloseWindow={closeDesktopWindow}
		onFocusWindow={focusDesktopWindow}
		onSelectFile={selectDesktopFile}
	/>
{/if}

{#if pairingOpen}
	<PairingDialog
		info={connectionInfo}
		server={snapshot?.server ?? null}
		onClose={() => (pairingOpen = false)}
	/>
{/if}

<style>
	.app {
		display: grid;
		grid-template-columns: var(--sidebar-w) minmax(0, 1fr) var(--activity-w);
		width: 100%;
		height: 100vh;
		height: 100dvh;
		overflow: hidden;
		background: var(--bg);
	}

	.center {
		display: flex;
		flex-direction: column;
		min-width: 0;
		height: 100%;
		overflow: hidden;
	}

	.activity-slot {
		min-width: 0;
		height: 100%;
	}

	.messages-scroll {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		overscroll-behavior: contain;
	}

	.messages {
		display: flex;
		flex-direction: column;
		gap: 20px;
		width: 100%;
		max-width: var(--content-w);
		min-height: 100%;
		padding: 32px 24px 16px;
		margin: 0 auto;
	}

	.messages-end {
		height: 1px;
	}

	.composer-area {
		flex: none;
		width: 100%;
		padding-top: 10px;
	}

	.connection-banner {
		display: flex;
		align-items: center;
		gap: 8px;
		width: calc(100% - 48px);
		max-width: calc(var(--content-w) - 48px);
		min-height: 34px;
		padding: 7px 12px;
		margin: 0 auto 10px;
		border: 1px solid #3f2e12;
		border-radius: var(--radius-sm);
		background: #1b1409;
		color: #fbbf24;
		font-size: 12.5px;
		line-height: 1.4;
	}

	.connection-banner.warning {
		border-color: #3a2626;
		background: #1b0f0f;
		color: #fca5a5;
	}

	.connection-banner span:nth-child(2) {
		flex: 1;
		min-width: 0;
	}

	.connection-banner button {
		flex: none;
		min-height: 24px;
		padding: 0 6px;
		border-radius: 4px;
		color: inherit;
		font-size: 12px;
		text-decoration: underline;
		text-underline-offset: 2px;
	}

	.connection-banner button:hover {
		background: rgb(255 255 255 / 6%);
	}

	.banner-dot {
		flex: none;
		width: 7px;
		height: 7px;
		border-radius: 50%;
		background: var(--warning);
	}

	.state {
		display: flex;
		flex: 1;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 6px;
		padding: 32px;
		color: var(--muted);
		text-align: center;
	}

	.state p {
		margin: 8px 0 0;
		color: var(--text);
		font-size: 15px;
	}

	.state span {
		max-width: 360px;
		font-size: 13px;
		line-height: 1.5;
	}

	.error-state {
		color: var(--danger);
	}

	.spin {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.retry-button {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		height: 38px;
		padding: 0 13px;
		margin-top: 10px;
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font-size: 13px;
	}

	.retry-button:hover {
		background: var(--surface-2);
	}

	.empty-chat {
		display: flex;
		flex: 1;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 4px;
		padding: 48px 0;
		text-align: center;
	}

	.empty-chat p {
		margin: 0;
		color: var(--text);
		font-size: 15px;
	}

	.empty-chat span {
		max-width: 420px;
		color: var(--muted);
		font-size: 13px;
		line-height: 1.5;
	}

	.suggestions {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 8px;
		margin-top: 16px;
	}

	.suggestions button {
		min-height: 36px;
		padding: 0 12px;
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--muted-strong);
		font-size: 12.5px;
	}

	.suggestions button:hover {
		border-color: #3f3f46;
		color: var(--text);
	}

	@media (max-width: 1180px) {
		.app {
			grid-template-columns: var(--sidebar-w) minmax(0, 1fr);
		}

		.activity-slot {
			display: none;
		}
	}

	@media (max-width: 860px) {
		.app {
			grid-template-columns: minmax(0, 1fr);
		}

		.app > :global(.sidebar) {
			display: none;
		}

		.center > :global(.chat-header) {
			display: none;
		}
	}
</style>
