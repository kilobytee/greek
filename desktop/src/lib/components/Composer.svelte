<script lang="ts">
	import { tick } from 'svelte';
	import { ArrowUp, Check, ChevronUp } from '@lucide/svelte';
	import type { ModelOption } from '#lib/types';
	import { shortModelName } from '#lib/format';

	let {
		value,
		model,
		models,
		thinking,
		disabled,
		onValueChange,
		onSend,
		onModelChange
	}: {
		value: string;
		model: string;
		models: ModelOption[];
		thinking: boolean;
		disabled: boolean;
		onValueChange: (value: string) => void;
		onSend: () => void;
		onModelChange: (model: string) => void;
	} = $props();

	let textarea = $state<HTMLTextAreaElement>();
	let triggerEl = $state<HTMLDivElement>();
	let menuEl = $state<HTMLDivElement>();
	let open = $state(false);

	const canSend = $derived(value.trim().length > 0 && !disabled && !thinking);
	const modelLabel = $derived(shortModelName(model));

	$effect(() => {
		const el = textarea;
		if (!el) return;
		void value;
		el.style.height = 'auto';
		el.style.height = `${Math.min(el.scrollHeight, 140)}px`;
	});

	$effect(() => {
		if (!open) return;
		const onPointerDown = (event: PointerEvent) => {
			const target = event.target;
			if (!(target instanceof Node)) return;
			if (menuEl?.contains(target) || triggerEl?.contains(target)) return;
			open = false;
		};
		document.addEventListener('pointerdown', onPointerDown);
		return () => document.removeEventListener('pointerdown', onPointerDown);
	});

	async function toggleMenu() {
		if (open) {
			open = false;
			return;
		}
		open = true;
		await tick();
		const first = menuEl?.querySelector<HTMLButtonElement>(
			'button[role="menuitemradio"]:not(:disabled)'
		);
		first?.focus();
	}

	function closeMenu() {
		open = false;
		triggerEl?.querySelector<HTMLButtonElement>('.model-pill')?.focus();
	}

	function chooseModel(id: string) {
		onModelChange(id);
		closeMenu();
	}

	function onMenuKeydown(event: KeyboardEvent) {
		if (event.key === 'Tab') {
			closeMenu();
			return;
		}
		const items = Array.from(
			menuEl?.querySelectorAll<HTMLButtonElement>('button[role="menuitemradio"]:not(:disabled)') ??
				[]
		);
		if (items.length === 0) return;
		const index = items.indexOf(document.activeElement as HTMLButtonElement);

		if (event.key === 'Escape') {
			event.preventDefault();
			closeMenu();
			return;
		}
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			items[(index + 1 + items.length) % items.length]?.focus();
			return;
		}
		if (event.key === 'ArrowUp') {
			event.preventDefault();
			items[(index - 1 + items.length) % items.length]?.focus();
			return;
		}
		if (event.key === 'Home') {
			event.preventDefault();
			items[0]?.focus();
			return;
		}
		if (event.key === 'End') {
			event.preventDefault();
			items[items.length - 1]?.focus();
		}
	}

	function submit() {
		if (!canSend) return;
		onSend();
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
			event.preventDefault();
			submit();
		}
	}
</script>

<div class="composer-shell">
	<div class="composer">
		<textarea
			bind:this={textarea}
			value={value}
			oninput={(event) => onValueChange(event.currentTarget.value)}
			onkeydown={onKeydown}
			placeholder="Escribe un mensaje..."
			aria-label="Mensaje para Greek"
			rows="1"
			maxlength="8000"
		></textarea>

		<div class="bottom-row">
			<div class="model-control" bind:this={triggerEl}>
				<button
					type="button"
					class="model-pill"
					aria-haspopup="menu"
					aria-expanded={open}
					aria-controls="model-menu"
					onclick={toggleMenu}
				>
					<span class="model-pill-label">{modelLabel}</span>
				</button>
				<button
					type="button"
					class="chevron"
					aria-label="Abrir selector de modelo"
					aria-haspopup="menu"
					aria-expanded={open}
					aria-controls="model-menu"
					onclick={toggleMenu}
				>
					<ChevronUp size={15} strokeWidth={1.8} aria-hidden="true" />
				</button>

				{#if open}
					<div
						id="model-menu"
						class="model-menu"
						role="menu"
						tabindex="-1"
						aria-label="Seleccionar modelo"
						bind:this={menuEl}
						onkeydown={onMenuKeydown}
					>
						<div class="menu-heading">Modelo</div>
						{#each models as option (option.id)}
							<button
								type="button"
								class="menu-item"
								role="menuitemradio"
								aria-checked={model === option.id}
								disabled={!option.available}
								onclick={() => chooseModel(option.id)}
							>
								<span class="menu-copy">
									<span class="menu-label">{option.label}</span>
									<span class="menu-detail">
										{option.provider}{option.detail ? ` · ${option.detail}` : ''}
									</span>
								</span>
								{#if model === option.id}
									<Check size={15} strokeWidth={2} aria-hidden="true" />
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<button
				type="button"
				class="send"
				aria-label="Enviar mensaje"
				disabled={!canSend}
				onclick={submit}
			>
				<ArrowUp size={17} strokeWidth={2.4} aria-hidden="true" />
			</button>
		</div>
	</div>
</div>

<style>
	.composer-shell {
		flex: none;
		width: 100%;
		max-width: var(--content-w);
		padding: 0 24px 24px;
		margin: 0 auto;
	}

	.composer {
		display: flex;
		flex-direction: column;
		padding: 14px 16px 12px;
		border: 1px solid var(--border);
		border-radius: var(--radius-lg);
		background: var(--surface);
		transition: border-color 140ms ease;
	}

	.composer:focus-within {
		border-color: #2e2e33;
	}

	textarea {
		display: block;
		width: 100%;
		min-height: 24px;
		max-height: 140px;
		padding: 0;
		border: 0;
		outline: 0;
		background: transparent;
		color: var(--text);
		font-size: 15px;
		line-height: 1.55;
		resize: none;
	}

	textarea::placeholder {
		color: var(--muted);
	}

	.bottom-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin-top: 12px;
	}

	.model-control {
		position: relative;
		display: flex;
		align-items: center;
		gap: 2px;
		min-width: 0;
	}

	.model-pill {
		display: inline-flex;
		align-items: center;
		height: 32px;
		max-width: 220px;
		padding: 0 12px;
		overflow: hidden;
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font-size: 13.5px;
		white-space: nowrap;
		transition: background 120ms ease, border-color 120ms ease;
	}

	.model-pill-label {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.model-pill:hover,
	.chevron:hover {
		border-color: #3f3f46;
		background: var(--surface-2);
	}

	.chevron {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: 1px solid transparent;
		border-radius: var(--radius-sm);
		color: var(--muted);
		transition: background 120ms ease, color 120ms ease, border-color 120ms ease;
	}

	.chevron:hover {
		color: var(--text);
	}

	.send {
		display: inline-flex;
		flex: none;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: var(--send);
		color: var(--send-ink);
		transition: transform 120ms ease, opacity 120ms ease;
	}

	.send:not(:disabled):hover {
		transform: translateY(-1px);
	}

	.send:disabled {
		opacity: 0.55;
	}

	.model-menu {
		position: absolute;
		bottom: calc(100% + 8px);
		left: 0;
		z-index: 30;
		width: 320px;
		max-width: min(320px, calc(100vw - 64px));
		max-height: 340px;
		padding: 6px;
		overflow-y: auto;
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-md);
		background: #1b1b1e;
		box-shadow: 0 16px 40px rgb(0 0 0 / 45%);
	}

	.menu-heading {
		padding: 6px 10px 8px;
		color: var(--muted);
		font-size: 11.5px;
		font-weight: 500;
	}

	.menu-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		width: 100%;
		min-height: 48px;
		padding: 8px 10px;
		border-radius: 8px;
		color: var(--text);
		text-align: left;
	}

	.menu-item:hover:not(:disabled) {
		background: #27272a;
	}

	.menu-item:disabled {
		color: #52525b;
	}

	.menu-copy {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.menu-label {
		overflow: hidden;
		font-size: 13.5px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.menu-detail {
		overflow: hidden;
		color: var(--muted);
		font-size: 11.5px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.menu-item:disabled .menu-detail {
		color: #52525b;
	}
</style>
