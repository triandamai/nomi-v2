<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Mascot from '../agents/Mascot.svelte';

	// Flow / Mobile Build → Mobile / Ask Bar (xWDVb) over the workspace's preview: Koda one tap away.
	// Tapping it goes to the chat with the composer ready. The copy is the pen's own "Ask Koda for a
	// change…" (Qw22t), not "about this screen": the frame is sandboxed, so Koda can't read it (G14).
	// The mic waits for the phone composer to record voice. It floats, so it has the float shadow.
	type Props = { label?: string; onask: () => void };
	let { label, onask }: Props = $props();
</script>

<button type="button" class="flex h-13 w-full cursor-pointer items-center gap-2.5 rounded-full bg-surface px-3.5 text-left shadow-float outline -outline-offset-1 outline-border-strong focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus" onclick={onask}>
	<Mascot agent="koda" size={20} />
	<span class="min-w-0 flex-1 truncate text-body text-text-3">{label ?? m.pb_ask_bar()}</span>
</button>
