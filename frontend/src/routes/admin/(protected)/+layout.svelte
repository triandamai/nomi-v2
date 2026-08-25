<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { LayoutData } from './$types';

	let { children }: { data: LayoutData; children: Snippet } = $props();
</script>

<div class="flex h-screen" style="background: var(--md-sys-color-surface)">
	<aside
		class="flex w-56 flex-col p-4"
		style="background: var(--md-sys-color-surface-container); border-right: 1px solid var(--md-sys-color-outline-variant)"
	>
		<h2 class="md-title-large mb-4" style="color: var(--md-sys-color-on-surface)">Admin</h2>
		<nav class="flex flex-col gap-1">
			<a href="/admin" class="m3-nav-link">Dashboard</a>
			<a href="/admin/settings/llm" class="m3-nav-link">LLM Settings</a>
		</nav>
		<form method="POST" action="/logout?redirect_to=/admin/login" class="mt-auto">
			<button type="submit" class="m3-nav-link m3-nav-link--muted w-full text-left">Log out</button>
		</form>
	</aside>
	<main class="flex-1 overflow-y-auto p-8">
		{@render children()}
	</main>
</div>

<style>
	.m3-nav-link {
		display: block;
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-full);
		font-family: var(--md-sys-typescale-label-large-font);
		font-weight: var(--md-sys-typescale-label-large-weight);
		font-size: var(--md-sys-typescale-label-large-size);
		letter-spacing: var(--md-sys-typescale-label-large-tracking);
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		border: none;
		background: transparent;
		cursor: pointer;
	}
	.m3-nav-link:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.m3-nav-link--muted {
		color: var(--md-sys-color-outline);
	}
</style>
