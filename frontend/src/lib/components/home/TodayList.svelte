<script lang="ts">
	import { agentName, clockTime, RECURRENCE, type TodayItem } from '$lib/home';

	let { items, timezone }: { items: TodayItem[]; timezone: string } = $props();
</script>

<ul class="today">
	{#each items as item (item.id)}
		<li class="today__row">
			<span class="today__time">{clockTime(item.run_at, timezone)}</span>
			<span class="today__text">
				<span class="today__label">{item.label}</span>
				<span class="today__meta">{item.recurrence ? RECURRENCE[item.recurrence] : `From ${agentName(item.agent)}`}</span>
			</span>
		</li>
	{/each}
</ul>

<style>
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.today__row {
		display: grid;
		grid-template-columns: 56px minmax(0, 1fr);
		gap: 12px;
		padding: 12px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.today__row:last-child {
		border-bottom: none;
	}
	.today__time {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		color: var(--md-sys-color-primary);
		padding-top: 2px;
		font-variant-numeric: tabular-nums;
	}
	.today__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.today__label {
		font-weight: 600;
		font-size: 0.9375rem;
	}
	.today__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
