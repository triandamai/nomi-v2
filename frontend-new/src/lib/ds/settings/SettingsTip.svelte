<script lang="ts">
	import { CircleCheck } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / General (m7lvY1) → Tip: one line about how the page works, under a stroke,
	// with the accent check.
	let { text }: { text: string } = $props();
</script>

<p class="flex items-start gap-2 border-t border-border pt-2.75 text-caption text-text-3">
	<span class="inline-flex shrink-0 text-accent"><Icon icon={CircleCheck} size={14} /></span>
	<span>{text}</span>
</p>
