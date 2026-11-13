<script lang="ts">
	import type { Component } from 'svelte';
	import { Plus, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Settings / Connections (OBmgS) → App: one app to add, 136 high, outlined, 16 in and 10
	// apart. Its 36px logo, the name and the outlined plus; what it does; and, at the bottom, the
	// member it's for in mono with the access it asks for on the right. One this Nomi can't connect
	// says why in mono where the plus would be. The plus is IconButton's outline with the pen's
	// quieter glyph (text-3).
	type Props = {
		icon: Component<LucideProps>;
		name: string;
		caption: string;
		agent: CrewId;
		forLabel: string;
		access: string;
		/** Add's accessible name ("Add Hevy"). */
		add: string;
		unavailable?: string;
		onadd?: () => void;
	};
	let { icon, name, caption, agent, forLabel, access, add, unavailable, onadd }: Props = $props();

	const id = $props.id();
</script>

<article aria-labelledby="{id}-name" class="flex min-h-34 min-w-0 flex-col gap-2.5 rounded-md p-4 outline -outline-offset-1 outline-border">
	<div class="flex items-center gap-2.5">
		<span data-part="logo" class="inline-flex size-9 shrink-0 items-center justify-center rounded-sm bg-surface-2 text-text"><Icon {icon} /></span>
		<span id="{id}-name" class="min-w-0 flex-1 truncate text-ui font-medium text-text">{name}</span>
		{#if unavailable}
			<span class="shrink-0 font-mono text-label text-text-3">{unavailable}</span>
		{:else}
			<button
				type="button"
				aria-label={add}
				onclick={onadd}
				class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-sm border border-border-strong text-text-3 outline-offset-1 transition-colors hover:bg-surface-2 hover:text-text pointer-coarse:size-11"
			>
				<Icon icon={Plus} />
			</button>
		{/if}
	</div>
	<p class="text-caption leading-comfy text-text-2">{caption}</p>
	<div class="flex-1" aria-hidden="true"></div>
	<div class="flex items-center gap-1.5">
		<Mascot {agent} size={20} />
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{forLabel}</span>
		<span class="shrink-0 font-mono text-label text-text-3">{access}</span>
	</div>
</article>
