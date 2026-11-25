<script lang="ts">
	import type { ChatMessage } from '#lib/types';
	import InlineText from './InlineText.svelte';
	import ToolChips from './ToolChips.svelte';

	let { message }: { message: ChatMessage } = $props();
</script>

{#if message.role === 'user'}
	<div class="user-row">
		<div class="user-bubble">
			<InlineText text={message.text} />
		</div>
	</div>
{:else}
	<div class="agent-row">
		<div class="avatar" aria-hidden="true"></div>
		<div class="agent-content">
			{#if message.tools.length > 0}
				<ToolChips tools={message.tools} />
			{/if}
			{#if message.text}
				<div class="agent-text"><InlineText text={message.text} /></div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.user-row {
		display: flex;
		justify-content: flex-end;
	}

	.user-bubble {
		max-width: min(82%, 560px);
		padding: 11px 16px;
		border-radius: 14px;
		background: var(--surface-2);
		color: #f4f4f5;
		font-size: 15px;
		line-height: 1.5;
	}

	.agent-row {
		display: flex;
		align-items: flex-start;
		gap: 12px;
	}

	.avatar {
		flex: none;
		width: 24px;
		height: 24px;
		margin-top: 3px;
		border-radius: 50%;
		background: #f4f4f5;
	}

	.agent-content {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
		padding-top: 3px;
	}

	.agent-text {
		color: var(--text);
		font-size: 15px;
		line-height: 1.55;
	}
</style>
