<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import CrewRow from '../identity/CrewRow.svelte';
	import Status from '../status/Status.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Home → Crew Column (EXxti): "Your crew" and how many are idle (or working) in a 52px
	// head over a bottom stroke, 20 in; then each member as a Crew Row, 2 apart, 8 in. The column's
	// own width and left stroke come from AppShell's panel, whose stroke takes 1px of room, so the
	// head and the list sit 1px less in on the left (AGENTS.md §3) and land where the pen has them.
	// A member the design has no mascot for (one an admin added, or the inspector) shows their
	// initial on a quiet 32px disc in a row drawn like Crew Row (whose props stay the old app's
	// twin's: it always takes a mascot).
	type Member = { agent?: CrewId; name: string; role: string; state?: string };
	type Props = { summary: { kind: 'idle' | 'live'; label: string }; members: Member[] };
	let { summary, members }: Props = $props();

	const id = $props.id();
</script>

<aside aria-labelledby="{id}-title" class="flex flex-col">
	<header class="flex h-13 shrink-0 items-center justify-between border-b border-border pr-5 pl-4.75">
		<h2 id="{id}-title" class="text-ui font-medium text-text">{m.crew_panel_title()}</h2>
		<!-- The pen's head is a dot and mono words: idle in text-3; while some work, the accent dot. -->
		<Status kind={summary.kind === 'live' ? 'ready' : 'idle'} label={summary.label} />
	</header>
	<ul class="flex flex-col gap-0.5 py-2 pr-2 pl-1.75">
		{#each members as member, index (index)}
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
		{/each}
	</ul>
</aside>
