<script lang="ts">
	import { FileText, FolderOpen, Globe, Terminal } from '@lucide/svelte';
	import type { VirtualDesktop, VirtualWindow } from '#lib/types';

	let { desktop, onOpen }: { desktop: VirtualDesktop; onOpen: () => void } = $props();

	const active = $derived(
		desktop.windows.find((window) => window.id === desktop.activeWindowId) ??
			desktop.windows[0] ??
			null
	);

	const selectedFile = $derived(
		desktop.files.find((file) => file.name === desktop.selectedFile) ?? desktop.files[0] ?? null
	);

	function iconFor(window: VirtualWindow) {
		if (window.kind === 'browser') return Globe;
		if (window.kind === 'files') return FolderOpen;
		return Terminal;
	}
</script>

<button type="button" class="frame" onclick={onOpen} aria-label="Abrir escritorio virtual">
	<div class="frame-bar" aria-hidden="true">
		<span class="handle"></span>
	</div>
	<div class="viewport">
		{#if active}
			{@const ActiveIcon = iconFor(active)}
			<div class="mini-window">
				<div class="mini-title">
					<ActiveIcon size={10} strokeWidth={1.8} />
					<span>{active.title}</span>
				</div>
				{#if active.kind === 'browser'}
					{#if desktop.browser}
						<div class="mini-browser">
							<div class="url">{desktop.browser.url}</div>
							<div class="mini-lines">
								{#each desktop.browser.lines.slice(0, 4) as line}
									<span>{line}</span>
								{/each}
							</div>
						</div>
					{:else}
						<div class="mini-empty">Sin página abierta</div>
					{/if}
				{:else if active.kind === 'files'}
					<div class="mini-files">
						{#if desktop.files.length === 0}
							<div class="mini-empty">Sin archivos</div>
						{:else}
							{#each desktop.files.slice(0, 4) as file}
								<div class="file-row" class:selected={file.name === selectedFile?.name}>
									<FileText size={10} strokeWidth={1.8} />
									<span>{file.name}</span>
								</div>
							{/each}
						{/if}
					</div>
				{:else}
					<div class="mini-terminal">
						{#each desktop.terminal.slice(-4) as line}
							<span>{line}</span>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
	</div>
</button>

<style>
	.frame {
		display: flex;
		flex-direction: column;
		width: 100%;
		aspect-ratio: 1.38;
		padding: 8px;
		overflow: hidden;
		border: 1px solid var(--border);
		border-radius: var(--radius-md);
		background: var(--surface);
		text-align: left;
		transition: border-color 140ms ease;
	}

	.frame:hover {
		border-color: var(--border-strong);
	}

	.frame-bar {
		display: flex;
		flex: none;
		align-items: center;
		height: 16px;
		padding-left: 4px;
	}

	.handle {
		display: block;
		width: 88px;
		height: 7px;
		border-radius: 999px;
		background: #27272a;
	}

	.viewport {
		display: flex;
		flex: 1;
		align-items: center;
		justify-content: center;
		min-height: 0;
		overflow: hidden;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--bg);
	}

	.mini-window {
		width: 86%;
		height: 78%;
		overflow: hidden;
		border: 1px solid #27272a;
		border-radius: 6px;
		background: #0f0f11;
	}

	.mini-title {
		display: flex;
		align-items: center;
		gap: 5px;
		height: 22px;
		padding: 0 7px;
		border-bottom: 1px solid var(--border);
		color: var(--muted-strong);
		font-size: 9px;
		white-space: nowrap;
	}

	.mini-title span {
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.mini-browser,
	.mini-files,
	.mini-terminal {
		padding: 7px;
	}

	.url {
		margin-bottom: 7px;
		padding: 3px 5px;
		overflow: hidden;
		border: 1px solid var(--border);
		border-radius: 3px;
		color: var(--muted);
		font-family: var(--font-mono);
		font-size: 8px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.mini-lines,
	.mini-terminal {
		display: flex;
		flex-direction: column;
		gap: 3px;
		color: var(--text);
		font-size: 9px;
		line-height: 1.35;
	}

	.mini-files {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.file-row {
		display: flex;
		align-items: center;
		gap: 5px;
		height: 20px;
		padding: 0 5px;
		border-radius: 3px;
		color: var(--muted-strong);
		font-size: 9px;
	}

	.file-row.selected {
		background: var(--surface-2);
		color: var(--text);
	}

	.mini-empty {
		padding: 16px 8px;
		color: var(--muted);
		font-size: 9px;
		text-align: center;
	}

	.mini-terminal {
		font-family: var(--font-mono);
		font-size: 8px;
	}
</style>
