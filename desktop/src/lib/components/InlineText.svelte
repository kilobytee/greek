<script lang="ts">
	let { text }: { text: string } = $props();

	type Part = { code: boolean; value: string };

	function parseInline(value: string): Part[] {
		const parts: Part[] = [];
		let index = 0;

		while (index < value.length) {
			const open = value.indexOf('`', index);
			if (open === -1) {
				parts.push({ code: false, value: value.slice(index) });
				break;
			}
			const close = value.indexOf('`', open + 1);
			if (close === -1) {
				parts.push({ code: false, value: value.slice(index) });
				break;
			}
			if (open > index) parts.push({ code: false, value: value.slice(index, open) });
			parts.push({ code: true, value: value.slice(open + 1, close) });
			index = close + 1;
		}

		return parts.filter((part) => part.value.length > 0);
	}

	const parts = $derived(parseInline(text));
</script>

<span class="inline-text"
	>{#each parts as part}{#if part.code}<code>{part.value}</code>{:else}{part.value}{/if}{/each}</span
>

<style>
	.inline-text {
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}

	code {
		display: inline;
		padding: 1px 5px;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--surface);
		color: var(--text);
		font-family: var(--font-mono);
		font-size: 0.86em;
		line-height: inherit;
	}
</style>
