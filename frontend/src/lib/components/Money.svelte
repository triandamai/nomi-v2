<script lang="ts">
	import { moneyPieces } from '$lib/money';

	// An amount (or a sentence holding amounts) with each amount's cents raised: $12⁵⁰ style.
	// Screen readers get the plain text.
	let { value, class: extraClass = '' }: { value: string; class?: string } = $props();

	const pieces = $derived(moneyPieces(value));
</script>

<span class="money {extraClass}"><span class="sr-only">{value}</span><span aria-hidden="true">{#each pieces as piece, i (i)}{#if piece.cents}<sup class="money__cents">{piece.text}</sup>{:else}{piece.text}{/if}{/each}</span></span>

<style>
	.money {
		white-space: nowrap;
		font-variant-numeric: tabular-nums;
	}
	.money__cents {
		font-size: 0.58em;
		vertical-align: 0.45em;
		line-height: 0;
		margin-left: 0.04em;
		font-weight: inherit;
	}
</style>
