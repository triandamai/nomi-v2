<script lang="ts">
	import { Trash2 } from '@lucide/svelte';
	import IconButton from '../actions/IconButton.svelte';

	// Flow / Build — Workspace tools (U16wU) → Screen / Environment (r85wCf) → Variable: one
	// environment variable of the app, over its line: its name in mono with when and who last set it,
	// and its value masked on the right (values are never shown again), with Remove for the
	// project's admins.
	type Props = { name: string; meta: string; masked: string; remove: string; onremove?: () => void };
	let { name, meta, masked, remove, onremove }: Props = $props();
</script>

<li class="flex min-h-13 items-center gap-3 border-b border-border pt-2.5 pb-2.25">
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="truncate font-mono text-ui text-text">{name}</span>
		<span class="truncate font-mono text-label text-text-3">{meta}</span>
	</span>
	<span class="shrink-0 font-mono text-label text-text-2">{masked}</span>
	{#if onremove}<IconButton icon={Trash2} label={remove} variant="ghost" iconSize={14} onclick={onremove} />{/if}
</li>
