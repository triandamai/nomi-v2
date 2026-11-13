<script lang="ts" module>
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	export type Starter = {
		icon: Component<LucideProps>;
		/** What to build, as a person would describe it. */
		label: string;
		/** "web app · database", in mono. */
		meta: string;
		onpick: (label: string) => void;
	};
</script>

<script lang="ts">
	import { ArrowUpLeft } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';

	// Design System / Projects → Project / Empty / First project (HJAkR): the page before there is
	// any project. 820 wide, 28 in: Koda (32) saying what to do, then starters 40 tall, 2 apart, each
	// under a line drawn inside: its icon, what it is, its kind in mono and ↖ (it goes up into the
	// composer above).
	type Props = { starters: Starter[] };
	let { starters }: Props = $props();

	const id = $props.id();
</script>

<section class="flex w-205 max-w-full flex-col gap-4 rounded-md p-7 outline -outline-offset-1 outline-border" aria-labelledby="{id}-title">
	<div class="flex items-center gap-3">
		<Mascot agent="koda" size={32} />
		<div class="flex min-w-0 flex-1 flex-col gap-1">
			<h2 id="{id}-title" class="text-body font-medium text-text">{m.projects_first_title()}</h2>
			<p class="text-ui text-text-3">{m.projects_first_body()}</p>
		</div>
	</div>
	<ul aria-label={m.projects_starters()} class="flex flex-col gap-0.5">
		{#each starters as starter (starter.label)}
			<li>
				<button
					type="button"
					onclick={() => starter.onpick(starter.label)}
					class="relative flex h-10 w-full cursor-pointer items-center gap-3 rounded-sm px-3 text-left outline-offset-1 before:absolute before:inset-x-0 before:top-0 before:h-px before:bg-border hover:bg-surface-2"
				>
					<span class="inline-flex text-text-2"><Icon icon={starter.icon} size={14} /></span>
					<span class="min-w-0 flex-1 truncate text-ui text-text">{starter.label}</span>
					<span class="shrink-0 font-mono text-label text-text-3">{starter.meta}</span>
					<span class="inline-flex text-text-3"><Icon icon={ArrowUpLeft} size={14} /></span>
				</button>
			</li>
		{/each}
	</ul>
</section>
