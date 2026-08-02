<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { LayoutData } from './$types';

	let { children }: { data: LayoutData; children: Snippet } = $props();
</script>

<div class="flex h-screen bg-neutral-50">
	<aside class="w-56 border-r border-neutral-200 bg-white p-4">
		<h2 class="mb-4 text-lg font-semibold">Admin</h2>
		<nav class="space-y-2 text-sm">
			<a href="/admin" class="block text-neutral-700 hover:underline">Dashboard</a>
			<a href="/admin/settings/llm" class="block text-neutral-700 hover:underline">LLM Settings</a>
		</nav>
		<form method="POST" action="/logout?redirect_to=/admin/login" class="mt-6">
			<button type="submit" class="text-sm text-neutral-500 underline">Log out</button>
		</form>
	</aside>
	<main class="flex-1 overflow-y-auto p-8">
		{@render children()}
	</main>
</div>
