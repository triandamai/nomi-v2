<script lang="ts">
	import green from './assets/dot-green.svg?url';
	import yellow from './assets/dot-yellow.svg?url';

	// Figma, Nomi Builder → Badge (170:350), Subtle: green for live or ready, yellow for in progress,
	// with the 6px dot. 8 at the sides, 2 above and below, 12 / 16 medium.
	type Props = { label: string; tone: 'green' | 'yellow'; dot?: boolean };
	let { label, tone, dot = true }: Props = $props();
</script>

<span class={['badge', tone]}>
	{#if dot}<img src={tone === 'green' ? green : yellow} alt="" width="6" height="6" />{/if}
	{label}
</span>

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: 999px;
		font-size: 12px;
		line-height: 16px;
		font-weight: 500;
		white-space: nowrap;
	}

	.badge img {
		display: block;
		width: 6px;
		height: 6px;
	}

	.green {
		background: var(--lp-green-subtle);
		color: var(--lp-green-fg);
	}

	.yellow {
		background: var(--lp-yellow-subtle);
		color: var(--lp-yellow-fg);
	}
</style>
