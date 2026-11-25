<script lang="ts">
	import type { Chat } from '#lib/types';
	import { relativeTime } from '#lib/format';

	let {
		chats,
		activeChatId,
		now,
		onNewChat,
		onSelectChat
	}: {
		chats: Chat[];
		activeChatId: string;
		now: number;
		onNewChat: () => void;
		onSelectChat: (id: string) => void;
	} = $props();
</script>

<aside class="sidebar" aria-label="Conversaciones">
	<div class="brand">Greek</div>

	<button class="new-chat" type="button" onclick={onNewChat} aria-keyshortcuts="Control+N">
		<span>Nuevo chat</span>
		<span class="shortcut" aria-hidden="true">Ctrl N</span>
	</button>

	<div class="recent-heading">Recientes</div>

	<nav class="chat-list" aria-label="Chats recientes">
		{#if chats.length === 0}
			<p class="empty">Aún no hay chats.</p>
		{:else}
			{#each chats as chat (chat.id)}
				<button
					type="button"
					class="chat-item"
					class:active={chat.id === activeChatId}
					aria-current={chat.id === activeChatId ? 'page' : undefined}
					onclick={() => onSelectChat(chat.id)}
				>
					<span class="chat-title">{chat.title}</span>
					<span class="chat-time">{relativeTime(chat.updatedAt, now)}</span>
				</button>
			{/each}
		{/if}
	</nav>
</aside>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		width: var(--sidebar-w);
		min-width: var(--sidebar-w);
		height: 100%;
		padding: 20px 16px 16px;
		border-right: 1px solid var(--border);
		background: var(--bg);
	}

	.brand {
		padding: 0 2px;
		margin-bottom: 18px;
		color: #f4f4f5;
		font-size: 28px;
		font-weight: 600;
		letter-spacing: -0.035em;
		line-height: 1.1;
	}

	.new-chat {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		height: 44px;
		padding: 0 12px;
		border: 1px solid var(--border-strong);
		border-radius: var(--radius-sm);
		background: var(--surface);
		color: var(--text);
		font-size: 14px;
		text-align: left;
		transition: background 120ms ease, border-color 120ms ease;
	}

	.new-chat:hover {
		border-color: #3f3f46;
		background: var(--surface-2);
	}

	.shortcut {
		color: var(--muted);
		font-size: 12px;
		letter-spacing: 0;
	}

	.recent-heading {
		padding: 0 10px;
		margin: 24px 0 8px;
		color: var(--muted);
		font-size: 12px;
		font-weight: 500;
	}

	.chat-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		overflow-y: auto;
	}

	.chat-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		width: 100%;
		height: 40px;
		padding: 0 10px;
		border-radius: var(--radius-sm);
		color: #d4d4d8;
		text-align: left;
		transition: background 120ms ease, color 120ms ease;
	}

	.chat-item:hover {
		background: #111113;
		color: var(--text);
	}

	.chat-item.active {
		background: var(--surface-2);
		color: var(--text);
	}

	.chat-title {
		overflow: hidden;
		font-size: 14px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.chat-time {
		flex: none;
		color: var(--muted);
		font-size: 12px;
		font-variant-numeric: tabular-nums;
	}

	.empty {
		padding: 0 10px;
		margin: 4px 0;
		color: var(--muted);
		font-size: 13px;
	}

	@media (max-width: 980px) {
		.sidebar {
			width: 216px;
			min-width: 216px;
			padding-inline: 12px;
		}
	}
</style>
