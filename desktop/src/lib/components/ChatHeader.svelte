<script lang="ts">
	let { title }: { title: string } = $props();
</script>

<header class="chat-header">
	<div class="title" title={title}>{title}</div>
</header>

<style>
	.chat-header {
		display: flex;
		flex: none;
		align-items: center;
		justify-content: center;
		height: 56px;
		padding: 0 24px;
		border-bottom: 1px solid var(--border);
		background: var(--bg);
	}

	.title {
		overflow: hidden;
		max-width: 100%;
		color: var(--muted-strong);
		font-size: 13.5px;
		font-weight: 500;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
