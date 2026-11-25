<script lang="ts">
	import { onMount } from 'svelte';
	import {
		FileText,
		FolderOpen,
		Globe,
		Monitor,
		Terminal,
		X
	} from '@lucide/svelte';
	import type { VirtualDesktop, VirtualWindow } from '#lib/types';

	let {
		desktop,
		onClose,
		onOpenApp,
		onCloseWindow,
		onFocusWindow,
		onSelectFile
	}: {
		desktop: VirtualDesktop;
		onClose: () => void;
		onOpenApp: (app: 'browser' | 'files') => void;
		onCloseWindow: (id: string) => void;
		onFocusWindow: (id: string) => void;
		onSelectFile: (file: string) => void;
	} = $props();

	let dialogEl = $state<HTMLDialogElement>();

	const selectedFile = $derived(
		desktop.files.find((file) => file.name === desktop.selectedFile) ?? desktop.files[0] ?? null
	);
	const activeWindow = $derived(
		desktop.windows.find((window) => window.id === desktop.activeWindowId) ?? null
	);

	onMount(() => {
		dialogEl?.showModal();
	});

	function close() {
		if (dialogEl?.open) dialogEl.close();
		else onClose();
	}

	function onDialogClose() {
		onClose();
	}

	function onBackdrop(event: MouseEvent) {
		if (event.target === dialogEl) close();
	}

	function windowStyle(window: VirtualWindow): string {
		return [
			`left:${(window.x / 900) * 100}%`,
			`top:${(window.y / 600) * 100}%`,
			`width:${(window.width / 900) * 100}%`,
			`height:${(window.height / 600) * 100}%`
		].join(';');
	}

	function titleIcon(window: VirtualWindow) {
		if (window.kind === 'browser') return Globe;
		if (window.kind === 'files') return FolderOpen;
		return Terminal;
	}

	function activate(window: VirtualWindow) {
		onFocusWindow(window.id);
	}
</script>

<dialog
	bind:this={dialogEl}
	class="screen-dialog"
	onclose={onDialogClose}
	onclick={onBackdrop}
	aria-label="Escritorio virtual"
>
	<div class="screen-shell">
		<header class="screen-header">
			<div class="screen-title">
				<Monitor size={16} strokeWidth={1.7} aria-hidden="true" />
				<span>Escritorio virtual</span>
			</div>
			<button type="button" class="icon-button" aria-label="Cerrar escritorio virtual" onclick={close}>
				<X size={17} strokeWidth={1.8} aria-hidden="true" />
			</button>
		</header>

		<div class="screen-body">
			<div class="canvas" aria-label="Vista del entorno aislado">
				{#if desktop.windows.length === 0}
					<div class="empty">
						<p>Sin ventanas abiertas</p>
						<span>Abre Navegador o Archivos para ver el entorno.</span>
					</div>
				{/if}

				{#each desktop.windows as window (window.id)}
					{@const TitleIcon = titleIcon(window)}
					<div
						class="window"
						class:active={window.id === desktop.activeWindowId}
						style={windowStyle(window)}
					>
						<div
							class="window-titlebar"
							role="button"
							tabindex="0"
							onclick={() => activate(window)}
							onkeydown={(event) => {
								if (event.key === 'Enter' || event.key === ' ') {
									event.preventDefault();
									activate(window);
								}
							}}
						>
							<span class="window-icon"><TitleIcon size={12} strokeWidth={1.8} /></span>
							<span class="window-title">{window.title}</span>
							<span class="window-control" aria-hidden="true"></span>
						</div>

						<div class="window-body">
							{#if window.kind === 'browser'}
								{#if desktop.browser}
									<div class="url-bar">{desktop.browser.url}</div>
									<div class="page">
										<div class="page-kicker">Open-Meteo</div>
										{#each desktop.browser.lines as line}
											<p>{line}</p>
										{/each}
									</div>
								{:else}
									<div class="window-empty">Sin página abierta. Pide el clima para navegar.</div>
								{/if}
							{:else if window.kind === 'files'}
								<div class="files-layout">
									<div class="file-list">
										{#if desktop.files.length === 0}
											<div class="window-empty">Sin archivos todavía.</div>
										{:else}
											{#each desktop.files as file}
												<button
													type="button"
													class="file-item"
													class:selected={file.name === selectedFile?.name}
													onclick={() => onSelectFile(file.name)}
												>
													<FileText size={12} strokeWidth={1.8} aria-hidden="true" />
													<span>{file.name}</span>
												</button>
											{/each}
										{/if}
									</div>
									{#if selectedFile}
										<pre class="file-content">{selectedFile.content}</pre>
									{/if}
								</div>
							{:else}
								<pre class="terminal">{desktop.terminal.join('\n') || 'Sin actividad.'}</pre>
							{/if}
						</div>
					</div>
				{/each}
			</div>

			<p class="scope-note">
				Entorno aislado local. Los archivos viven en la carpeta sandbox del equipo.
			</p>
		</div>

		<footer class="screen-toolbar">
			<div class="toolbar-group">
				<button type="button" class="toolbar-button" onclick={() => onOpenApp('browser')}>
					<Globe size={15} strokeWidth={1.8} aria-hidden="true" />
					Navegador
				</button>
				<button type="button" class="toolbar-button" onclick={() => onOpenApp('files')}>
					<FolderOpen size={15} strokeWidth={1.8} aria-hidden="true" />
					Archivos
				</button>
			</div>
			<button
				type="button"
				class="toolbar-button danger"
				disabled={!activeWindow}
				onclick={() => activeWindow && onCloseWindow(activeWindow.id)}
			>
				Cerrar ventana
			</button>
		</footer>
	</div>
</dialog>

<style>
	.screen-dialog {
		width: 100vw;
		max-width: none;
		height: 100vh;
		max-height: none;
		padding: 0;
		margin: 0;
		border: 0;
		background: var(--bg);
		color: var(--text);
	}

	.screen-dialog::backdrop {
		background: rgb(0 0 0 / 70%);
	}

	.screen-shell {
		display: flex;
		flex-direction: column;
		width: 100%;
		height: 100%;
		background: var(--bg);
	}

	.screen-header {
		display: flex;
		flex: none;
		align-items: center;
		justify-content: space-between;
		height: 56px;
		padding: 0 20px;
		border-bottom: 1px solid var(--border);
	}

	.screen-title {
		display: flex;
		align-items: center;
		gap: 9px;
		color: var(--text);
		font-size: 14px;
		font-weight: 500;
	}

	.icon-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border-radius: var(--radius-sm);
		color: var(--muted);
		transition: color 120ms ease, background 120ms ease;
	}

	.icon-button:hover {
		background: var(--surface-2);
		color: var(--text);
	}

	.screen-body {
		display: flex;
		flex: 1;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		min-height: 0;
		padding: 24px;
	}

	.canvas {
		position: relative;
		width: min(100%, 1100px);
		aspect-ratio: 3 / 2;
		overflow: hidden;
		border: 1px solid var(--border);
		border-radius: var(--radius-md);
		background: #0b0b0c;
	}

	.empty {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 4px;
		color: var(--muted);
		text-align: center;
	}

	.empty p {
		margin: 0;
		color: var(--muted-strong);
		font-size: 14px;
	}

	.empty span {
		font-size: 12px;
	}

	.window {
		position: absolute;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		border: 1px solid #2b2b30;
		border-radius: 7px;
		background: var(--surface);
		box-shadow: 0 16px 32px rgb(0 0 0 / 35%);
	}

	.window.active {
		border-color: #3f3f46;
	}

	.window-titlebar {
		display: flex;
		flex: none;
		align-items: center;
		gap: 6px;
		height: 26px;
		padding: 0 8px;
		border-bottom: 1px solid var(--border);
		color: var(--muted-strong);
		font-size: 10px;
		cursor: default;
	}

	.window-icon {
		display: inline-flex;
		color: var(--muted);
	}

	.window-title {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.window-control {
		width: 7px;
		height: 7px;
		margin-left: auto;
		border-radius: 50%;
		background: #3f3f46;
	}

	.window-body {
		flex: 1;
		min-height: 0;
		overflow: auto;
	}

	.url-bar {
		margin: 8px;
		padding: 4px 7px;
		overflow: hidden;
		border: 1px solid var(--border);
		border-radius: 4px;
		color: var(--muted);
		font-family: var(--font-mono);
		font-size: 9px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.page {
		padding: 2px 12px 14px;
	}

	.page-kicker {
		margin-bottom: 8px;
		color: var(--muted);
		font-size: 9px;
	}

	.page p {
		margin: 0 0 5px;
		color: var(--text);
		font-size: 12px;
		line-height: 1.45;
	}

	.window-empty {
		padding: 24px 16px;
		color: var(--muted);
		font-size: 11px;
		line-height: 1.5;
		text-align: center;
	}

	.files-layout {
		display: flex;
		height: 100%;
		min-height: 0;
	}

	.file-list {
		flex: none;
		width: 38%;
		padding: 6px;
		overflow-y: auto;
		border-right: 1px solid var(--border);
	}

	.file-item {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		min-height: 28px;
		padding: 0 6px;
		border-radius: 4px;
		color: var(--muted-strong);
		font-size: 10px;
		text-align: left;
	}

	.file-item:hover,
	.file-item.selected {
		background: var(--surface-2);
		color: var(--text);
	}

	.file-content,
	.terminal {
		flex: 1;
		min-width: 0;
		padding: 10px;
		margin: 0;
		overflow: auto;
		color: var(--text);
		font-family: var(--font-mono);
		font-size: 10px;
		line-height: 1.55;
		white-space: pre-wrap;
	}

	.scope-note {
		margin: 0;
		color: var(--muted);
		font-size: 11.5px;
	}

	.screen-toolbar {
		display: flex;
		flex: none;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 12px 20px;
		border-top: 1px solid var(--border);
	}

	.toolbar-group {
		display: flex;
		gap: 8px;
	}

	.toolbar-button {
		display: inline-flex;
		align-items: center;
		gap: 7px;
		height: 40px;
		padding: 0 14px;
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font-size: 13px;
		transition: background 120ms ease, border-color 120ms ease;
	}

	.toolbar-button:hover:not(:disabled) {
		border-color: #3f3f46;
		background: var(--surface-2);
	}

	.toolbar-button.danger {
		color: var(--muted-strong);
	}

	.toolbar-button:disabled {
		opacity: 0.45;
	}

	@media (max-width: 760px) {
		.screen-body {
			padding: 12px;
		}

		.screen-toolbar {
			padding: 10px 12px;
		}

		.toolbar-button {
			padding: 0 10px;
		}
	}
</style>
