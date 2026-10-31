<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Mobile / Chat Row: a chat in the phone's list. Who last answered (in their tint), the title,
	// when, the last line, and a mint dot when it's unread.
	type Props = { href: string; agent: CrewId; title: string; time?: string; preview?: string; unread?: boolean };
	let { href, agent, title, time, preview, unread = false }: Props = $props();
</script>

<a {href} class="flex w-full items-center gap-3 px-5 py-3 -outline-offset-2">
	<span class="inline-flex size-10 shrink-0 items-center justify-center rounded-full" style="background: var(--crew-{agent}-bg)"><Mascot {agent} size={32} /></span>
	<span class="flex min-w-0 flex-1 flex-col gap-0.75">
		<span class="flex items-center gap-2">
			<span class="min-w-0 flex-1 truncate text-phone font-medium text-text">{title}</span>
			{#if time}<span class="font-mono text-label text-text-3">{time}</span>{/if}
		</span>
		<span class="flex items-center gap-2">
			<span class="line-clamp-2 min-w-0 flex-1 text-phone-sm text-text-3">{preview}</span>
			{#if unread}<span class="size-2 shrink-0 rounded-full bg-accent"><span class="sr-only">{m.mobile_unread()}</span></span>{/if}
		</span>
	</span>
</a>
