<!-- In a card: outlined on bg, choosing what a share link covers. -->
<script lang="ts">
	import Segmented from '$lib/ds/inputs/Segmented.svelte';

	let scope = $state('reply');
</script>

<div class="flex w-full max-w-100 flex-col gap-3 rounded-md bg-surface p-5 outline -outline-offset-1 outline-border-strong">
	<p class="text-body font-medium text-text">Share</p>
	<Segmented
		label="What to share"
		surface="bg"
		bind:value={scope}
		options={[
			{ value: 'reply', label: 'This reply' },
			{ value: 'chat', label: 'Whole chat', count: '24 msgs' },
		]}
	/>
</div>
