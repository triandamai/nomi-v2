<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Loading Older: at the top of a long thread while earlier messages load.
</script>

<p role="status" class="flex w-full items-center justify-center gap-1.5 font-mono text-label text-text-3">
	<span class="inline-flex animate-spin motion-reduce:animate-none"><Icon icon={LoaderCircle} size={12} /></span>{m.loading_older()}
</p>
