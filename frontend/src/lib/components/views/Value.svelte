<!-- One value of a view, formatted for the person's language (amounts with their cents raised). -->
<script lang="ts">
	import Money from '$lib/components/Money.svelte';
	import { getLocale } from '$lib/paraglide/runtime';
	import { formatValue } from '$lib/views/format';
	import type { Format, Scalar } from '$lib/views/types';

	let { value, format }: { value: Scalar; format?: Format } = $props();

	const text = $derived(formatValue(value, format, getLocale()));
</script>

{#if format === 'money' && typeof value !== 'boolean'}<Money value={text} />{:else}{text}{/if}
