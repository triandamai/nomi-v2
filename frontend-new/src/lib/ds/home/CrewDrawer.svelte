<script lang="ts" module>
	import type { CrewId } from '../agents/crew';

	export type CrewDrawerMember = { agent?: CrewId; name: string; role: string; state?: string; active?: boolean };
</script>

<script lang="ts">
	import { X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../actions/IconButton.svelte';
	import Status from '../status/Status.svelte';
	import SectionLabel from '../status/SectionLabel.svelte';
	import CrewRow from '../identity/CrewRow.svelte';

	// What the crew capsule opens: a panel from the right edge, over the scrim, the crew column's
	// width (320, the whole width on a phone). The head is the crew column's (EXxti: "Your crew" and
	// the summary, 52 high over a stroke) with a close button; then who's at it now (working or
	// waiting on you) above everyone who's idle, each a Crew Row. Nobody busy says so in a line,
	// rather than an empty list. A native modal dialog: focus stays inside, Escape and a click on
	// the scrim close it. It slides in, unless reduced motion is asked for.
	type Props = { open?: boolean; summary: { kind: 'idle' | 'live'; label: string }; members: CrewDrawerMember[]; onclose?: () => void };
	let { open = $bindable(false), summary, members, onclose }: Props = $props();

	const id = $props.id();
	const busy = $derived(members.filter((member) => member.active));
	const idle = $derived(members.filter((member) => !member.active));

	let wasOpen = false;
	$effect(() => {
		if (wasOpen && !open) onclose?.();
		wasOpen = open;
	});

	function modal(dialog: HTMLDialogElement) {
		dialog.showModal();
	}
</script>

{#snippet row(member: CrewDrawerMember)}
	<li>
		{#if member.agent}
			<CrewRow agent={member.agent} name={member.name} role={member.role} state={member.state} />
		{:else}
			<div class="flex w-full items-center gap-3 rounded-sm px-3 py-2">
				<span aria-hidden="true" class="inline-flex size-8 shrink-0 items-center justify-center rounded-full bg-surface-2 text-ui font-semibold text-text-2">{member.name.charAt(0).toUpperCase()}</span>
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-ui font-medium text-text">{member.name}</span>
					<span class="text-caption leading-cozy text-text-3">{member.role}</span>
				</span>
				{#if member.state}<span class="font-mono text-label text-text-3">{member.state}</span>{/if}
			</div>
		{/if}
	</li>
{/snippet}

{#if open}
	<dialog
		use:modal
		aria-labelledby="{id}-title"
		onclose={() => (open = false)}
		onclick={(event) => {
			if (event.target === event.currentTarget) open = false;
		}}
		class="fixed inset-y-0 right-0 left-auto m-0 h-dvh max-h-none w-full max-w-80 bg-transparent p-0 text-text backdrop:bg-overlay max-sm:max-w-none"
	>
		<div class="flex h-full translate-x-0 flex-col overflow-y-auto border-l border-border bg-surface shadow-float transition-transform duration-200 ease-out starting:translate-x-full motion-reduce:transition-none">
			<header class="flex h-13 shrink-0 items-center gap-3 border-b border-border pr-3 pl-4.75">
				<h2 id="{id}-title" class="flex-1 text-ui font-medium text-text">{m.crew_panel_title()}</h2>
				<Status kind={summary.kind === 'live' ? 'ready' : 'idle'} label={summary.label} />
				<IconButton icon={X} label={m.action_close()} variant="ghost" onclick={() => (open = false)} />
			</header>
			<section aria-labelledby="{id}-busy" class="flex flex-col gap-1 pt-4">
				<span id="{id}-busy" class="px-4.75"><SectionLabel label={m.crew_drawer_busy()} /></span>
				{#if busy.length > 0}
					<ul class="flex flex-col gap-0.5 px-1.75">
						{#each busy as member, index (index)}{@render row(member)}{/each}
					</ul>
				{:else}
					<p class="px-4.75 py-2 text-caption leading-cozy text-text-3">{m.crew_drawer_none_busy()}</p>
				{/if}
			</section>
			{#if idle.length > 0}
				<section aria-labelledby="{id}-idle" class="flex flex-col gap-1 pt-4 pb-4">
					<span id="{id}-idle" class="px-4.75"><SectionLabel label={m.crew_drawer_idle()} /></span>
					<ul class="flex flex-col gap-0.5 px-1.75">
						{#each idle as member, index (index)}{@render row(member)}{/each}
					</ul>
				</section>
			{/if}
		</div>
	</dialog>
{/if}
