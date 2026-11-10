<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';

	// Auth / Enter code → New Note: Nomi saying one useful thing about what comes next, on the quiet
	// accent tint with its 20px mascot. When it reports what just happened (a password changed), it
	// is announced as a status.
	type Props = { text: string; announce?: boolean };
	let { text, announce = false }: Props = $props();
</script>

<p role={announce ? 'status' : undefined} class="flex w-full items-center gap-2.5 rounded-sm bg-accent-bg px-3 py-2.5">
	<Mascot agent="nomi" size={20} />
	<span class="min-w-0 flex-1 text-caption text-text">{text}</span>
</p>
