<script lang="ts">
	import type { Snippet } from 'svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: Snippet } = $props();
</script>

<div class="flex h-screen" style="background: var(--md-sys-color-surface)">
	<Sidebar sessions={data.sessions} userEmail={data.userEmail} />
	<main class="flex-1 overflow-hidden">
		{@render children()}
	</main>
</div>
