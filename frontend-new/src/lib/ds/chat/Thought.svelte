<script lang="ts">
	import { Lightbulb } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	/** One decision the crew made on the way, said quietly. */
	let { text }: { text: string } = $props();
</script>

<p class="flex w-full gap-2 text-ui leading-relaxed text-text-3">
	<span class="inline-flex shrink-0 pt-0.75"><Icon icon={Lightbulb} size={14} /></span>
	<span>{text}</span>
</p>
