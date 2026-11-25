<script lang="ts">
	import { Smartphone } from '@lucide/svelte';
	import type { Chat, ServerInfo, VirtualDesktop } from '#lib/types';
	import DesktopPreview from './DesktopPreview.svelte';

	let {
		chat,
		desktop,
		server,
		onOpenScreen,
		onOpenPairing
	}: {
		chat: Chat | null;
		desktop: VirtualDesktop;
		server: ServerInfo | null;
		onOpenScreen: () => void;
		onOpenPairing: () => void;
	} = $props();

	const model = $derived(chat?.model === 'auto' || !chat?.model ? 'Auto' : chat.model);
	const reason = $derived(chat?.routeReason || 'esperando mensaje');
	const tools = $derived(chat && chat.tools.length > 0 ? chat.tools.join(', ') : 'Ninguna');
</script>

<aside class="activity" aria-label="Actividad del agente">
	<div class="activity-heading">
		<h2>Actividad</h2>
		<button
			type="button"
			class="pair-button"
			title="Conectar iPhone"
			aria-label="Conectar iPhone"
			onclick={onOpenPairing}
		>
			<Smartphone size={15} strokeWidth={1.7} aria-hidden="true" />
		</button>
	</div>

	<div class="meta">
		<div class="meta-block">
			<div class="meta-label">Modelo</div>
			<div class="meta-value model-value">{model}</div>
		</div>
		<div class="meta-block">
			<div class="meta-label">Motivo</div>
			<div class="meta-value">{reason}</div>
		</div>
		<div class="meta-block">
			<div class="meta-label">Herramientas</div>
			<div class="meta-value">{tools}</div>
		</div>
	</div>

	<div class="desktop-block">
		<div class="meta-label">Escritorio virtual</div>
		<DesktopPreview {desktop} onOpen={onOpenScreen} />
		{#if server && !server.online}
			<p class="server-note">
				Solo en este equipo. Para el iPhone, inicia con
				<code>GREEK_BIND=0.0.0.0:4319</code>.
			</p>
		{/if}
	</div>
</aside>

<style>
	.activity {
		display: flex;
		flex-direction: column;
		width: 100%;
		min-width: 0;
		height: 100%;
		padding: 20px;
		overflow-y: auto;
		border-left: 1px solid var(--border);
		background: var(--bg);
	}

	.activity-heading {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 18px;
	}

	h2 {
		margin: 0;
		color: var(--muted-strong);
		font-size: 14px;
		font-weight: 500;
		line-height: 1.2;
	}

	.pair-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		margin: -4px -4px -4px 0;
		border-radius: 6px;
		color: var(--muted);
		transition: color 120ms ease, background 120ms ease;
	}

	.pair-button:hover {
		background: var(--surface-2);
		color: var(--text);
	}

	.meta {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.meta-block {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.meta-label {
		color: var(--muted);
		font-size: 12px;
		line-height: 1.4;
	}

	.meta-value {
		color: var(--text);
		font-size: 15px;
		line-height: 1.4;
		overflow-wrap: anywhere;
	}

	.model-value {
		font-family: var(--font-sans);
		letter-spacing: -0.01em;
	}

	.desktop-block {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 22px;
	}

	.server-note {
		margin: 2px 0 0;
		color: var(--muted);
		font-size: 11.5px;
		line-height: 1.5;
	}

	.server-note code {
		color: var(--muted-strong);
		font-family: var(--font-mono);
		font-size: 10.5px;
	}
</style>
