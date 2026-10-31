<!-- In a row with its own layout: a label wraps the box and the row's text, so the words name it. -->
<script lang="ts">
	import CheckMark from '$lib/ds/inputs/CheckMark.svelte';

	let mask = $state(true);
	let names = $state(false);
</script>

<div class="flex w-full max-w-90 flex-col gap-1">
	<label class="flex cursor-pointer items-center gap-3 rounded-sm px-3 py-2 hover:bg-surface-2">
		<CheckMark bind:checked={mask} />
		<span class={['min-w-0 flex-1 text-ui', mask ? 'text-text' : 'text-text-2']}>Mask amounts</span>
	</label>
	<label class="flex cursor-pointer items-center gap-3 rounded-sm px-3 py-2 hover:bg-surface-2">
		<CheckMark bind:checked={names} />
		<span class={['min-w-0 flex-1 text-ui', names ? 'text-text' : 'text-text-2']}>Include names</span>
	</label>
</div>
