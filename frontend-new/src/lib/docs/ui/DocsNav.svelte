<script module lang="ts">
	export type DocsLink = { title: string; href: string; active: boolean };
	export type DocsFolder = { title: string; links: DocsLink[] };
	export type DocsGroup = { title: string; links?: DocsLink[]; folders?: DocsFolder[] };
</script>

<script lang="ts">
	import { onMount } from 'svelte';
	import { ChevronRight } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import NavGroupLabel from '$lib/ds/navigation/NavGroupLabel.svelte';

	// The docs' sidebar, in the app's own sidebar language: group labels in mono capitals, rows that
	// fill with surface-2 when they're the page you're on. Long lists (components, patterns) fold
	// into folders; the one holding the current page starts open. The others fill in once the page
	// runs, so each of the static copy's pages doesn't carry every link twice over.
	let { groups }: { groups: DocsGroup[] } = $props();

	let running = $state(false);
	onMount(() => (running = true));
</script>

{#snippet link(item: DocsLink)}
	<a href={item.href} aria-current={item.active ? 'page' : undefined} class="row">{item.title}</a>
{/snippet}

<nav aria-label="Design system" class="flex flex-col gap-4">
	{#each groups as group (group.title)}
		<div class="flex flex-col gap-0.5">
			<NavGroupLabel label={group.title} />
			{#each group.links ?? [] as item (item.href)}{@render link(item)}{/each}
			{#each group.folders ?? [] as folder (folder.title)}
				{@const current = folder.links.some((item) => item.active)}
				<details open={current} class="group/folder">
					<summary class="row folder cursor-pointer list-none gap-1.5">
						<span class="inline-flex text-text-3 transition-transform group-open/folder:rotate-90 motion-reduce:transition-none"><Icon icon={ChevronRight} size={12} /></span>
						<span class="min-w-0 flex-1">{folder.title}</span>
						<span class="font-mono text-label text-text-3">{folder.links.length}</span>
					</summary>
					<div class="ml-3.5 flex flex-col gap-0.5 border-l border-border pl-2">
						{#if running || current}
							{#each folder.links as item (item.href)}{@render link(item)}{/each}
						{/if}
					</div>
				</details>
			{/each}
		</div>
	{/each}
</nav>

<style>
	.row {
		display: flex;
		min-height: 30px;
		align-items: center;
		padding: 6px 10px;
		border-radius: var(--nm-radius-sm);
		font-size: var(--nm-size-ui);
		line-height: 17px;
		color: var(--nm-text-2);
	}
	.row:hover {
		background: var(--nm-surface-2);
		color: var(--nm-text);
	}
	.row[aria-current='page'] {
		background: var(--nm-surface-2);
		color: var(--nm-text);
		font-weight: 500;
	}
	.folder::-webkit-details-marker {
		display: none;
	}
</style>
