<script lang="ts">
	let { label = 'Trabajando' }: { label?: string } = $props();
</script>

<div class="typing" role="status" aria-live="polite">
	<div class="avatar" aria-hidden="true"></div>
	<div class="content">
		<span class="label">{label}</span>
		<span class="dots" aria-hidden="true">
			<span></span>
			<span></span>
			<span></span>
		</span>
	</div>
</div>

<style>
	.typing {
		display: flex;
		align-items: center;
		gap: 12px;
		color: var(--muted);
	}

	.avatar {
		flex: none;
		width: 24px;
		height: 24px;
		margin-top: 3px;
		border-radius: 50%;
		background: #f4f4f5;
	}

	.content {
		display: flex;
		align-items: center;
		gap: 8px;
		padding-top: 3px;
	}

	.label {
		font-size: 13px;
	}

	.dots {
		display: inline-flex;
		gap: 3px;
		align-items: center;
	}

	.dots span {
		width: 3px;
		height: 3px;
		border-radius: 50%;
		background: var(--muted);
		animation: pulse 1.1s ease-in-out infinite;
	}

	.dots span:nth-child(2) {
		animation-delay: 140ms;
	}

	.dots span:nth-child(3) {
		animation-delay: 280ms;
	}

	@keyframes pulse {
		0%,
		80%,
		100% {
			opacity: 0.35;
		}
		40% {
			opacity: 1;
		}
	}
</style>
