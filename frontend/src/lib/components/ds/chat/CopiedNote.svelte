<script lang="ts">
	import { Check } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Copied: a small inverse note where you copied, saying what was copied
	// ("Copied as Markdown").
	let { label }: { label?: string } = $props();
</script>

<span role="status" class="inline-flex items-center gap-1.5 rounded-sm bg-text px-2 py-1 text-caption text-bg">
	<Icon icon={Check} size={12} />{label ?? m.copy_done()}
</span>
