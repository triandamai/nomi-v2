<script lang="ts">
	import { SquarePen } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Chat — Start & answer → Chats → New Chat: the phone's floating way to start a
	// chat, a 56 mint circle with the pen's square-pen at 22 in accent-ink, its own shadow (0 6 16).
	// Whoever places it sets where: the pen floats it 16 from the right, just above the tab bar.
	type Props = { label: string; href: string };
	let { label, href }: Props = $props();
</script>

<a {href} aria-label={label} class="inline-flex size-14 items-center justify-center rounded-full bg-accent text-accent-ink shadow-fab outline-offset-2">
	<Icon icon={SquarePen} size={22} />
</a>
