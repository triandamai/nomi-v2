<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Status from '../status/Status.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Settings / Connections (OBmgS) → Connection: one connected app, 248 high on the surface
	// with a border-strong stroke, 18 in and 14 apart. The top: its 40px logo, the name with the
	// account in mono, "on"; what it holds; CAN DO with its chips; and, pushed to the bottom under a
	// line, who uses it and when, with Manage. The pen's "Needs you" variant has nothing behind it
	// (the backend doesn't say when a connection lapses), so it isn't drawn.
	type Props = {
		icon: Component<LucideProps>;
		name: string;
		account: string;
		status: string;
		caption: string;
		canLabel: string;
		can: string[];
		usedBy: { agent: CrewId; name: string }[];
		/** When it was last used, already worded ("2h ago"); left out when it hasn't been. */
		when?: string;
		manage: string;
		onmanage?: () => void;
	};
	let { icon, name, account, status, caption, canLabel, can, usedBy, when, manage, onmanage }: Props = $props();

	const id = $props.id();
</script>

<article aria-labelledby="{id}-name" class="flex min-h-62 min-w-0 flex-col gap-3.5 rounded-md bg-surface p-4.5 outline -outline-offset-1 outline-border-strong">
	<div class="flex items-center gap-3">
		<span data-part="logo" class="inline-flex size-10 shrink-0 items-center justify-center rounded-md bg-surface-2 text-text"><Icon {icon} size={18} /></span>
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span id="{id}-name" class="truncate text-body font-medium text-text">{name}</span>
			<span class="truncate font-mono text-label text-text-3">{account}</span>
		</span>
		<Status kind="ready" label={status} />
	</div>
	<p class="text-caption text-text-2">{caption}</p>
	<div class="flex flex-col gap-1.5">
		<span class="font-mono text-label tracking-label text-text-3 uppercase">{canLabel}</span>
		<ul class="flex flex-wrap gap-1">
			{#each can as thing (thing)}
				<li class="inline-flex h-5.5 items-center rounded-xs bg-surface-2 px-1.75 font-mono text-label text-text-2">{thing}</li>
			{/each}
		</ul>
	</div>
	<div class="flex-1" aria-hidden="true"></div>
	<div class="flex items-center gap-2 border-t border-border pt-2.75">
		<span class="flex min-w-0 flex-1 items-center gap-1">
			{#each usedBy as member (member.agent)}<Mascot agent={member.agent} size={20} />{/each}
			<span class="truncate text-caption text-text-2">{usedBy.map((member) => member.name).join(', ')}</span>
			{#if when}<span class="shrink-0 font-mono text-label text-text-3">· {when}</span>{/if}
		</span>
		<Button variant="secondary" label={manage} onclick={onmanage} />
	</div>
</article>
