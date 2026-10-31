<script lang="ts">
	import { Mic } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';

	// Mobile / Ask Bar: floats over the live app so Koda is one tap away; opens the chat about this
	// screen, or starts a voice note. It floats, so it has the float shadow.
	type Props = { onask: () => void; onvoice: () => void };
	let { onask, onvoice }: Props = $props();
</script>

<div class="flex h-13 w-full items-center gap-2.5 rounded-full bg-surface pr-1.5 pl-3.5 shadow-float outline -outline-offset-1 outline-border-strong">
	<button type="button" class="flex h-full min-w-0 flex-1 cursor-pointer items-center gap-2.5 rounded-full text-left outline-offset-2" onclick={onask}>
		<Mascot agent="koda" size={20} />
		<span class="truncate text-body text-text-3">{m.ask_bar({ agent: 'Koda' })}</span>
	</button>
	<button type="button" aria-label={m.composer_record()} class="inline-flex size-10 shrink-0 cursor-pointer items-center justify-center rounded-full bg-surface-2 text-text outline-offset-1" onclick={onvoice}>
		<Icon icon={Mic} size={18} />
	</button>
</div>
