<script lang="ts">
	import type { ToolChip } from '#lib/types';

	let { tools }: { tools: ToolChip[] } = $props();
</script>

{#if tools.length > 0}
	<div class="chips" aria-label="Herramientas usadas">
		{#each tools as tool}
			<span class="chip" title={tool.detail ? `${tool.label} · ${tool.detail}` : tool.label}>
				<span>{tool.label}</span>
				{#if tool.detail}
					<span class="sep" aria-hidden="true">·</span>
					<span class="detail">{tool.detail}</span>
				{/if}
			</span>
		{/each}
	</div>
{/if}

<style>
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		max-width: 100%;
		height: 30px;
		padding: 0 11px;
		overflow: hidden;
		border: 1px solid var(--border-strong);
		border-radius: 999px;
		color: #d4d4d8;
		font-size: 13px;
		line-height: 1;
		white-space: nowrap;
	}

	.sep {
		color: var(--muted);
	}

	.detail {
		min-width: 0;
		overflow: hidden;
		color: #d4d4d8;
		text-overflow: ellipsis;
	}
</style>
