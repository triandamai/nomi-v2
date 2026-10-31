<script module lang="ts">
	export type SearchEntry = { title: string; group: string; href: string; blurb?: string };
</script>

<script lang="ts">
	import Search from '$lib/ds/inputs/Search.svelte';

	// Find any page: guides, components, patterns. ⌘K (Ctrl K) jumps here; arrows pick, Enter goes.
	let { entries }: { entries: SearchEntry[] } = $props();

	let query = $state('');
	let active = $state(0);
	let focused = $state(false);
	let box: HTMLDivElement;

	function rank(entry: SearchEntry, words: string[]) {
		const title = entry.title.toLowerCase();
		const text = `${title} ${entry.group.toLowerCase()} ${(entry.blurb ?? '').toLowerCase()}`;
		if (!words.every((word) => text.includes(word))) return 0;
		return words.reduce((score, word) => score + (title.startsWith(word) ? 4 : title.includes(word) ? 2 : 1), 0);
	}

	const results = $derived.by(() => {
		const words = query.toLowerCase().split(/\s+/).filter(Boolean);
		if (!words.length) return [];
		return entries
			.map((entry) => ({ entry, score: rank(entry, words) }))
			.filter((result) => result.score > 0)
			.sort((a, b) => b.score - a.score || a.entry.title.length - b.entry.title.length)
			.slice(0, 8)
			.map((result) => result.entry);
	});

	$effect(() => {
		void query;
		active = 0;
	});

	function onwindowkeydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
			event.preventDefault();
			box.querySelector('input')?.focus();
		} else if (box.contains(event.target as Node)) {
			move(event);
		}
	}

	function move(event: KeyboardEvent) {
		if (!results.length) return;
		if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
			event.preventDefault();
			active = (active + (event.key === 'ArrowDown' ? 1 : -1) + results.length) % results.length;
		} else if (event.key === 'Enter') {
			event.preventDefault();
			location.assign(results[active].href);
		} else if (event.key === 'Escape') {
			query = '';
		}
	}
</script>

<svelte:window onkeydown={onwindowkeydown} />

<div bind:this={box} class="relative w-full" role="search" onfocusin={() => (focused = true)} onfocusout={(event) => (focused = box.contains(event.relatedTarget as Node))}>
	<Search placeholder="Search the design system" label="Search the design system" shortcut="⌘K" bind:value={query} />
	{#if focused && query.trim()}
		<div class="absolute top-full right-0 z-30 mt-1.5 flex w-full min-w-80 flex-col rounded-md border border-border bg-surface p-1 shadow-float">
			{#each results as result, index (result.href)}
				<a
					href={result.href}
					class={['flex flex-col gap-0.5 rounded-sm px-2.5 py-2', index === active ? 'bg-surface-2' : 'hover:bg-surface-2']}
					onmouseenter={() => (active = index)}
				>
					<span class="flex items-baseline gap-2">
						<span class="min-w-0 flex-1 truncate text-ui text-text">{result.title}</span>
						<span class="shrink-0 font-mono text-label text-text-3">{result.group}</span>
					</span>
					{#if result.blurb}<span class="truncate text-caption text-text-3">{result.blurb}</span>{/if}
				</a>
			{:else}
				<p class="px-2.5 py-2 text-ui text-text-3">Nothing called “{query.trim()}”. Try a component name, or a word like colour.</p>
			{/each}
		</div>
	{/if}
</div>
