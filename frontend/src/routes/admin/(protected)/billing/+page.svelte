<script lang="ts">
	// Admin → Billing: where the paid side of plans is run. Each part has a page of its own; the
	// referral and renewal settings live in Configuration.
	import { m } from '$lib/paraglide/messages';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { can } from '$lib/permissions';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const pages = $derived([
		{ href: '/admin/billing/terms', title: m.billing_admin_terms(), lede: m.billing_admin_terms_lede() },
		{ href: '/admin/billing/coupons', title: m.billing_admin_coupons(), lede: m.billing_admin_coupons_lede() },
		{ href: '/admin/billing/payments', title: m.billing_admin_payments(), lede: m.billing_admin_payments_lede() },
		...(can(data.access, 'configuration') ? [{ href: '/admin/configuration/billing', title: m.billing_admin_settings(), lede: m.billing_admin_settings_lede() }] : []),
	]);
</script>

<PageHeader title={m.admin_billing()} lede={m.billing_admin_lede()} agent="money" />

<ul class="list">
	{#each pages as item (item.href)}
		<li>
			<a class="row" href={item.href}>
				<span class="row__title">{item.title}</span>
				<span class="row__lede">{item.lede}</span>
			</a>
		</li>
	{/each}
</ul>

<style>
	.list {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 320px), 1fr));
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		flex-direction: column;
		gap: 4px;
		height: 100%;
		min-height: 44px;
		padding: 16px;
		box-sizing: border-box;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.row:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.row:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.row__title {
		font-weight: 700;
	}
	.row__lede {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
</style>
