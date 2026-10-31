<script lang="ts">
	import { Check, X } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';

	// The rules side by side: what to do (mint) and what not to (danger), each a short sentence.
	let { dos = [], donts = [] }: { dos?: string[]; donts?: string[] } = $props();
	const columns = $derived([
		{ title: 'Do', icon: Check, tone: 'text-accent', bar: 'border-t-accent', items: dos },
		{ title: "Don't", icon: X, tone: 'text-danger', bar: 'border-t-danger', items: donts },
	].filter((column) => column.items.length));
</script>

<div class="grid gap-4 md:grid-cols-2">
	{#each columns as column (column.title)}
		<div class={['flex flex-col gap-3 rounded-md border border-t-2 border-border px-4 pt-3.5 pb-4', column.bar]}>
			<p class={['flex items-center gap-1.5 text-ui font-medium', column.tone]}><Icon icon={column.icon} size={14} />{column.title}</p>
			<ul class="flex flex-col gap-2">
				{#each column.items as item (item)}
					<li class="text-ui leading-normal text-text-2">{item}</li>
				{/each}
			</ul>
		</div>
	{/each}
</div>
