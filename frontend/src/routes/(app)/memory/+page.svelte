<script lang="ts">
	import MemoryGraph3D from '$lib/components/MemoryGraph3D.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	function timeAgo(iso: string): string {
		const diffMs = Date.now() - new Date(iso).getTime();
		const minutes = Math.floor(diffMs / 60000);
		if (minutes < 1) return 'just now';
		if (minutes < 60) return `${minutes}m ago`;
		const hours = Math.floor(minutes / 60);
		if (hours < 24) return `${hours}h ago`;
		return `${Math.floor(hours / 24)}d ago`;
	}

	const RECENCY_LIST_LIMIT = 10;

	const maxWeight = $derived(Math.max(1, ...data.memories.map((m) => m.weight)));

	const recentMemories = $derived(
		[...data.memories]
			.sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
			.slice(0, RECENCY_LIST_LIMIT),
	);
</script>

<div class="h-full overflow-y-auto p-4 md:p-8">
	<div class="max-w-3xl">
		<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Memory</h1>
		<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">
			Long-term facts Nomi has remembered about you from past conversations (RAG memory).
		</p>

		{#if data.memories.length === 0}
			<p class="md-body-medium mt-6" style="color: var(--md-sys-color-on-surface-variant)">
				Nothing remembered yet — Nomi saves durable facts as you chat.
			</p>
		{:else}
			<section class="mt-6">
				<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Memory map</h2>
				<p class="md-body-small mt-1" style="color: var(--md-sys-color-on-surface-variant)">
					Each node is a memory in 3D space; similar memories sit closer together. Bigger nodes are weighted more
					heavily in retrieval.
				</p>
				<div class="mt-3">
					<MemoryGraph3D memories={data.memories} />
				</div>
			</section>

			<section class="mt-8">
				<h2 class="md-title-medium" style="color: var(--md-sys-color-on-surface)">Weight &amp; recency</h2>
				<p class="md-body-small mt-1" style="color: var(--md-sys-color-on-surface-variant)">
					The {RECENCY_LIST_LIMIT} most recently reinforced memories.
				</p>
				<div class="mt-3 space-y-2">
					{#each recentMemories as memory (memory.id)}
						<div>
							<div class="flex items-center justify-between gap-2">
								<span class="md-body-small truncate" style="color: var(--md-sys-color-on-surface)">
									{memory.content}
								</span>
								<span class="md-body-small shrink-0" style="color: var(--md-sys-color-on-surface-variant)">
									{timeAgo(memory.updated_at)}
								</span>
							</div>
							<div
								class="mt-1 h-2 w-full overflow-hidden rounded-full"
								style="background: var(--md-sys-color-surface-container-highest)"
							>
								<div
									class="h-full rounded-full"
									style="width: {(memory.weight / maxWeight) * 100}%; background: var(--md-sys-color-primary)"
								></div>
							</div>
						</div>
					{/each}
				</div>
			</section>
		{/if}
	</div>
</div>
