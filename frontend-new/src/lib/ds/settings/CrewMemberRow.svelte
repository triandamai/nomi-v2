<script lang="ts">
	import CrewMark from './CrewMark.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Settings / Crew (vVUeh) → Member: one crew member in the roster, a 42px link [0,8] with
	// its mark, the name over what it does, and its state in mono on the right. Chosen, it wears the
	// member's own tint, the name at 500 and the rest in the member's ink; a state that needs the
	// person (waiting, the pen's "draft") is in the warning colour.
	type Props = { agent?: CrewId; initial?: string; name: string; role: string; state: string; tone?: 'quiet' | 'warning'; selected?: boolean; href: string };
	let { agent, initial, name, role, state: label, tone = 'quiet', selected = false, href }: Props = $props();

	const ink = $derived(selected && agent ? `color: var(--crew-${agent}-ink)` : undefined);
</script>

<a
	{href}
	aria-current={selected ? 'page' : undefined}
	data-sveltekit-noscroll
	data-sveltekit-replacestate
	style={selected && agent ? `background: var(--crew-${agent}-bg)` : undefined}
	class={['flex h-10.5 items-center gap-2.5 rounded-sm px-2 outline-offset-1', selected ? !agent && 'bg-surface-2' : 'hover:bg-surface-2']}
>
	<CrewMark {agent} initial={initial ?? name.charAt(0)} />
	<span class="flex min-w-0 flex-1 flex-col gap-px">
		<span class={['truncate text-ui text-text', selected && 'font-medium']}>{name}</span>
		<span class="truncate text-caption text-text-3" style={ink}>{role}</span>
	</span>
	<span class={['shrink-0 font-mono text-label', tone === 'warning' ? 'text-warning' : 'text-text-3']} style={tone === 'quiet' ? ink : undefined}>{label}</span>
</a>
