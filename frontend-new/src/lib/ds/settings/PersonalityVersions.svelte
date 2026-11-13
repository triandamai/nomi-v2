<script lang="ts">
	import { FileText } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Tag from '../status/Tag.svelte';

	// App / Settings / Crew (vVUeh) → Soul: the versions of how Nomi speaks, under the soul head (the
	// surface, [8,12], the file icon, its name and count in mono), newest first: each its number, what
	// it says, when, and Restore, or "current" on the one in use. The pen shows the file itself; the
	// backend keeps a description per version, so that's what's listed.
	export type VersionItem = { version: number; description: string; date: string; current: boolean };
	type Props = { title: string; count: string; versions: VersionItem[]; labels: { current: string; restore: string }; busy?: number; onrestore?: (version: number) => void; /** Said in place of the list when there are no versions yet. */ empty?: string };
	let { title, count, versions, labels, busy, onrestore, empty }: Props = $props();
</script>

<div class="overflow-hidden rounded-md outline -outline-offset-1 outline-border">
	<div class="flex h-11.5 items-center gap-2 border-b border-border bg-surface px-3">
		<span class="inline-flex text-text-2"><Icon icon={FileText} size={14} /></span>
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text">{title}</span>
		<span class="shrink-0 font-mono text-label text-text-3">{count}</span>
	</div>
	{#if !versions.length && empty}<p class="px-3 py-3 text-caption text-text-3">{empty}</p>{/if}
	<ul>
		{#each versions as item (item.version)}
			<li class="flex min-h-12 items-center gap-3 border-b border-border px-3 py-2 last:border-b-0">
				<span class="w-7 shrink-0 font-mono text-label text-text-3">v{item.version}</span>
				<span class="min-w-0 flex-1 text-ui text-text">{item.description}</span>
				<span class="shrink-0 font-mono text-label text-text-3">{item.date}</span>
				{#if item.current}
					<Tag kind="neutral" label={labels.current} />
				{:else}
					<Button variant="secondary" label={labels.restore} aria-label="{labels.restore} v{item.version}" loading={busy === item.version} onclick={() => onrestore?.(item.version)} />
				{/if}
			</li>
		{/each}
	</ul>
</div>
