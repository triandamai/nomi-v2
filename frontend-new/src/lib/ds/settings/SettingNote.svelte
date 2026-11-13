<script lang="ts">
	import { Info } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / General (m7lvY1) → Note: a quiet word under a group's rows ("Signing in with
	// Google doesn't give the crew your Gmail"), the info glyph before it, 12 above and below.
	let { text }: { text: string } = $props();
</script>

<p class="flex items-center gap-2 py-3 text-caption text-text-3">
	<span class="inline-flex shrink-0"><Icon icon={Info} size={12} /></span>
	<span>{text}</span>
</p>
