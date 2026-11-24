<script lang="ts">
	import type { CrewId } from '../agents/crew';
	import Mascot from '../agents/Mascot.svelte';

	// Flow / Mobile — Home, Inbox & You, step 1 (dl13j) → Crew (S9NsEX): five of the crew across the
	// phone, each its 32 mascot over its name at 12, opening the crew in Settings.
	type Props = { members: { agent: CrewId; name: string }[]; href: string; label: string };
	let { members, href, label }: Props = $props();
</script>

<nav aria-label={label} class="flex gap-1">
	{#each members.slice(0, 5) as member (member.agent)}
		<a {href} class="flex min-w-0 flex-1 flex-col items-center gap-1.5 rounded-lg py-1.5 outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus">
			<Mascot agent={member.agent} size={32} live={false} />
			<span class="w-full truncate text-center text-caption text-text-2">{member.name}</span>
		</a>
	{/each}
</nav>
