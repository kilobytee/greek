<script lang="ts">
	import { onMount } from 'svelte';
	import { Copy, Eye, EyeOff, Smartphone, X } from '@lucide/svelte';
	import type { ConnectionInfo, ServerInfo } from '#lib/types';
	import { copyText } from '#lib/format';

	let {
		info,
		server,
		onClose
	}: {
		info: ConnectionInfo | null;
		server: ServerInfo | null;
		onClose: () => void;
	} = $props();

	let dialogEl = $state<HTMLDialogElement>();
	let revealed = $state(false);
	let copied = $state<'url' | 'token' | null>(null);
	let copyError = $state('');
	let copyTimer: ReturnType<typeof setTimeout> | null = null;

	const lanUrl = $derived(info?.lanUrl || server?.lanUrl || '');
	const loopbackOnly = $derived(
		!!info && (info.bind.startsWith('127.') || info.bind.startsWith('localhost') || info.bind.startsWith('[::1]'))
	);

	onMount(() => {
		dialogEl?.showModal();
		return () => {
			if (copyTimer) clearTimeout(copyTimer);
		};
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

	async function copy(value: string, kind: 'url' | 'token') {
		copyError = '';
		try {
			await copyText(value);
			copied = kind;
			if (copyTimer) clearTimeout(copyTimer);
			copyTimer = setTimeout(() => (copied = null), 1800);
		} catch {
			copyError = 'No se pudo copiar. Selecciona el texto manualmente.';
		}
	}
</script>

<dialog
	bind:this={dialogEl}
	class="pair-dialog"
	onclose={onDialogClose}
	onclick={onBackdrop}
	aria-label="Conectar iPhone"
>
	<div class="pair-shell">
		<header class="pair-header">
			<div class="pair-title">
				<Smartphone size={17} strokeWidth={1.7} aria-hidden="true" />
				<h2>Conectar iPhone</h2>
			</div>
			<button type="button" class="icon-button" aria-label="Cerrar" onclick={close}>
				<X size={17} strokeWidth={1.8} aria-hidden="true" />
			</button>
		</header>

		<p class="intro">
			Abre Greek en el iPhone, toca el menú y elige <strong>Conectar al PC</strong>. Pega estos
			datos. Ambos equipos deben estar en la misma red Wi-Fi.
		</p>

		{#if info}
			<div class="field">
				<div class="field-heading">
					<label for="pair-url">Dirección</label>
					<button type="button" class="copy-button" onclick={() => copy(lanUrl, 'url')}>
						<Copy size={13} strokeWidth={1.8} aria-hidden="true" />
						{copied === 'url' ? 'Copiado' : 'Copiar'}
					</button>
				</div>
				<input id="pair-url" class="value-box" value={lanUrl} readonly />
			</div>

			<div class="field">
				<div class="field-heading">
					<label for="pair-token">Token</label>
					<div class="field-actions">
						<button
							type="button"
							class="copy-button"
							aria-label={revealed ? 'Ocultar token' : 'Mostrar token'}
							onclick={() => (revealed = !revealed)}
						>
							{#if revealed}
								<EyeOff size={13} strokeWidth={1.8} aria-hidden="true" />
								Ocultar
							{:else}
								<Eye size={13} strokeWidth={1.8} aria-hidden="true" />
								Mostrar
							{/if}
						</button>
						<button type="button" class="copy-button" onclick={() => copy(info.token, 'token')}>
							<Copy size={13} strokeWidth={1.8} aria-hidden="true" />
							{copied === 'token' ? 'Copiado' : 'Copiar'}
						</button>
					</div>
				</div>
				<input
					id="pair-token"
					class="value-box mono"
					value={revealed ? info.token : '••••••••••••••••••••••••••••••••'}
					readonly
				/>
			</div>

			{#if loopbackOnly}
				<div class="warning" role="status">
					El servidor solo escucha en este equipo. Para conectar el iPhone, inicia Greek con
					<code>GREEK_BIND=0.0.0.0:4319</code>.
				</div>
			{/if}

			{#if copyError}
				<p class="error" role="alert">{copyError}</p>
			{/if}

			<ol class="steps">
				<li>Abre la app Greek en el iPhone.</li>
				<li>Toca el botón de menú y elige Conectar al PC.</li>
				<li>Pega la dirección y el token, y pulsa Conectar.</li>
			</ol>

			<p class="footnote">
				El token se guarda en el Credential Manager de Windows o en el Llavero de macOS. No se
				incluye en el código ni en la interfaz móvil.
			</p>
		{:else}
			<div class="warning" role="alert">
				El núcleo aún no está listo. Espera unos segundos y vuelve a abrir este panel.
			</div>
		{/if}

		<div class="actions">
			<button type="button" class="close-button" onclick={close}>Cerrar</button>
		</div>
	</div>
</dialog>

<style>
	.pair-dialog {
		width: min(480px, calc(100vw - 32px));
		max-width: none;
		max-height: calc(100vh - 32px);
		padding: 0;
		margin: auto;
		overflow: auto;
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-lg);
		background: var(--surface);
		color: var(--text);
	}

	.pair-dialog::backdrop {
		background: rgb(0 0 0 / 66%);
	}

	.pair-shell {
		padding: 20px;
	}

	.pair-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 10px;
	}

	.pair-title {
		display: flex;
		align-items: center;
		gap: 9px;
	}

	h2 {
		margin: 0;
		font-size: 17px;
		font-weight: 600;
		letter-spacing: -0.015em;
	}

	.icon-button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: var(--radius-sm);
		color: var(--muted);
	}

	.icon-button:hover {
		background: var(--surface-2);
		color: var(--text);
	}

	.intro {
		margin: 0 0 18px;
		color: var(--muted-strong);
		font-size: 13px;
		line-height: 1.55;
	}

	.intro strong {
		color: var(--text);
		font-weight: 500;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-bottom: 14px;
	}

	.field-heading {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.field-heading label {
		color: var(--muted);
		font-size: 12px;
	}

	.field-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.copy-button {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		min-height: 28px;
		padding: 0 4px;
		border-radius: 4px;
		color: var(--muted-strong);
		font-size: 12px;
	}

	.copy-button:hover {
		color: var(--text);
	}

	.value-box {
		width: 100%;
		height: 40px;
		padding: 0 11px;
		border: 1px solid var(--border);
		border-radius: var(--radius-sm);
		background: var(--bg);
		color: var(--text);
		font-size: 13px;
	}

	.value-box.mono {
		font-family: var(--font-mono);
		font-size: 12px;
		letter-spacing: 0.02em;
	}

	.warning {
		padding: 10px 12px;
		margin: 4px 0 14px;
		border: 1px solid #3f2e12;
		border-radius: var(--radius-sm);
		background: #1b1409;
		color: #fbbf24;
		font-size: 12px;
		line-height: 1.5;
	}

	.warning code {
		padding: 1px 4px;
		border: 1px solid #3f2e12;
		border-radius: 3px;
		font-family: var(--font-mono);
		font-size: 11px;
	}

	.error {
		margin: 0 0 12px;
		color: var(--danger);
		font-size: 12px;
	}

	.steps {
		margin: 16px 0 12px;
		padding-left: 18px;
		color: var(--muted-strong);
		font-size: 12.5px;
		line-height: 1.7;
	}

	.footnote {
		margin: 0;
		color: var(--muted);
		font-size: 11.5px;
		line-height: 1.5;
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		margin-top: 18px;
	}

	.close-button {
		height: 40px;
		padding: 0 16px;
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-sm);
		background: var(--surface-2);
		color: var(--text);
		font-size: 13px;
	}

	.close-button:hover {
		border-color: #3f3f46;
	}
</style>
