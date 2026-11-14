<script lang="ts">
	// "Limits" on Billing: what the plan allows and how much of it is used.
	import { m } from '$lib/paraglide/messages';
	import { formatStorage } from '$lib/storage';
	import { meterShare } from '$lib/billing';
	import type { Entitlements } from '$lib/types';

	let { entitlements }: { entitlements: Entitlements } = $props();

	const rows = $derived([
		{ key: 'projects', label: m.billing_limit_projects(), used: entitlements.usage.projects, limit: entitlements.max_projects, show: (n: number) => String(n) },
		{ key: 'orgs', label: m.billing_limit_orgs(), used: entitlements.usage.orgs, limit: entitlements.max_orgs, show: (n: number) => String(n) },
		{ key: 'storage', label: m.billing_limit_storage(), used: entitlements.usage.storage_mb, limit: entitlements.max_storage_mb, show: formatStorage },
	]);
</script>

<section class="panel" aria-labelledby="limits-heading">
	<h2 id="limits-heading" class="panel__title">{m.billing_limits()}</h2>
	<ul class="meters">
		{#each rows as row (row.key)}
			{@const share = meterShare(row.used, row.limit)}
			<li class="meter">
				<div class="meter__top">
					<span>{row.label}</span>
					<span class="meter__value">{row.limit === null ? m.billing_limit_unlimited_used({ used: row.show(row.used) }) : m.billing_limit_used_of({ used: row.show(row.used), limit: row.show(row.limit) })}</span>
				</div>
				{#if share !== null}
					<div class="meter__track" role="meter" aria-label={row.label} aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(share * 100)}>
						<span class="meter__fill" class:meter__fill--full={share >= 1} style="width: {share * 100}%"></span>
					</div>
				{/if}
			</li>
		{/each}
		<li class="meter meter__top">
			<span>{m.billing_limit_custom_address()}</span>
			<span class="meter__value">{entitlements.custom_domain ? m.billing_yes() : m.billing_no()}</span>
		</li>
	</ul>
</section>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 12px;
		min-width: 0;
		padding: 18px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.panel__title {
		margin: 0;
		font-size: 1.0625rem;
		font-weight: 700;
	}
	.meters {
		display: flex;
		flex-direction: column;
		gap: 14px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.meter {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.meter__top {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: 4px 12px;
		font-size: 0.9375rem;
	}
	.meter__value {
		color: var(--md-sys-color-on-surface-variant);
		font-variant-numeric: tabular-nums;
	}
	.meter__track {
		height: 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		overflow: hidden;
	}
	.meter__fill {
		display: block;
		height: 100%;
		border-radius: inherit;
		background: var(--nomi-chart);
	}
	.meter__fill--full {
		background: var(--md-sys-color-error);
	}
</style>
