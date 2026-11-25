<script lang="ts">
	import '@fontsource-variable/inter';
	import '../app.css';
	import favicon from '#lib/assets/favicon.svg';
	import type { LayoutProps } from './$types';

	let { children }: LayoutProps = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Greek</title>
	<meta
		name="description"
		content="Greek, un agente personal con un escritorio virtual aislado."
	/>
</svelte:head>

{@render children()}
