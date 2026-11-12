<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';

	// App / Settings / Plan & usage (C0og49) → Fit Check: Nomi's word on whether the plan fits how
	// the person works, on the quiet accent tint.
	let { text }: { text: string } = $props();
</script>

<p role="status" class="flex items-center gap-2.5 rounded-sm bg-accent-bg px-3.5 py-2.5 text-caption text-text">
	<Mascot agent="nomi" size={20} />
	<span>{text}</span>
</p>
