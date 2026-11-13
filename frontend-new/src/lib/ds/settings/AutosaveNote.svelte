<script lang="ts">
	import { CircleCheck } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / General (m7lvY1) → Top Bar: that this page saves each change as it's made,
	// in mono with the check, where other pages put their actions. On a phone the check alone says
	// it, so the bar keeps room for the page's name; screen readers still hear the words.
	let { text }: { text: string } = $props();
</script>

<span class="flex items-center gap-2.5 font-mono text-label text-text-3">
	<span class="max-sm:sr-only">{text}</span>
	<span class="inline-flex text-accent"><Icon icon={CircleCheck} size={14} /></span>
</span>
