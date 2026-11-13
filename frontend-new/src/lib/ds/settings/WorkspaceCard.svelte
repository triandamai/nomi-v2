<script lang="ts">
	import { Check, Settings2 } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Tag from '../status/Tag.svelte';
	import WorkspaceTile from './WorkspaceTile.svelte';

	// App / Settings / Workspaces (l222Mz) → Workspace: one workspace, 236 high on the surface, 18 in
	// and 14 apart. Its tile, name (with CURRENT on the one projects come from) and the person's role
	// in mono; its numbers; and, under a line at the bottom, where projects come from (the accent
	// note) or the button to make it so, with Settings. The one in use has the stronger edge.
	type Props = {
		name: string;
		initial: string;
		tone: 'plain' | 'teal' | 'amber';
		meta: string;
		stats: { value: number; label: string }[];
		current: boolean;
		href: string;
		settings: string;
		currentTag: string;
		currentNote: string;
		use: string;
		busy?: boolean;
		onuse?: () => void;
	};
	let { name, initial, tone, meta, stats, current, href, settings, currentTag, currentNote, use, busy = false, onuse }: Props = $props();

	const id = $props.id();
</script>

<article aria-labelledby="{id}-name" class={['flex min-h-59 min-w-0 flex-col gap-3.5 rounded-md bg-surface p-4.5 outline -outline-offset-1', current ? 'outline-text-2' : 'outline-border-strong']}>
	<div class="flex items-center gap-3">
		<WorkspaceTile {initial} {tone} />
		<div class="flex min-w-0 flex-1 flex-col gap-0.75">
			<div class="flex min-w-0 items-center gap-1.5">
				<span id="{id}-name" class="truncate text-body font-medium text-text">{name}</span>
				{#if current}<Tag kind="go" label={currentTag} />{/if}
			</div>
			<span class="truncate font-mono text-label text-text-3">{meta}</span>
		</div>
	</div>
	<dl class="flex gap-4">
		{#each stats as stat (stat.label)}
			<div class="flex flex-col-reverse gap-px">
				<dt class="text-caption text-text-3">{stat.label}</dt>
				<dd class="font-mono text-body font-medium text-text">{stat.value}</dd>
			</div>
		{/each}
	</dl>
	<div class="flex-1" aria-hidden="true"></div>
	<div class="flex items-center gap-2 border-t border-border pt-2.75">
		{#if current}
			<span class="flex min-w-0 flex-1 items-center gap-1.5 text-caption text-accent"><Icon icon={Check} size={14} />{currentNote}</span>
		{:else}
			<Button variant="secondary" label={use} loading={busy} onclick={onuse} />
			<span class="flex-1" aria-hidden="true"></span>
		{/if}
		<Button variant="ghost" iconStart={Settings2} label={settings} {href} />
	</div>
</article>
