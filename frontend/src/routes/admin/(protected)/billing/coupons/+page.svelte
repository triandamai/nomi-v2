<script lang="ts">
	// Admin → Billing → Coupons: the codes people can enter at checkout, what each takes off and
	// how often it has been used.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import DatePicker from '$lib/components/m3/DatePicker.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import ListToolbar from '$lib/components/ListToolbar.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import { CURRENCY, formatMoney } from '$lib/billing';
	import { toISODate } from '$lib/dates';
	import { searchItems } from '$lib/listView';
	import { can } from '$lib/permissions';
	import type { AdminCoupon } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const canWrite = $derived(can(data.access, 'plans', 'write'));
	const canManage = $derived(can(data.access, 'plans', 'manage'));

	let query = $state('');
	const found = $derived(searchItems(data.coupons, query, (c) => c.code));

	let sheetOpen = $state(false);
	let editingId = $state<string | null>(null);
	let code = $state('');
	let description = $state('');
	let kind = $state('percent');
	let valueText = $state('');
	let capText = $state('');
	let perUserText = $state('1');
	let startsAt = $state('');
	let endsAt = $state('');
	let planIds = $state<string[]>([]);
	let minMonthsText = $state('1');
	let firstOnly = $state(false);
	let active = $state(true);

	const kindOptions = $derived([
		{ value: 'percent', label: m.billing_admin_coupon_percent() },
		{ value: 'amount', label: m.billing_admin_coupon_amount({ currency: CURRENCY }) },
	]);

	function load(coupon: AdminCoupon | null) {
		editingId = coupon?.id ?? null;
		code = coupon?.code ?? '';
		description = coupon?.description ?? '';
		kind = coupon?.discount_kind ?? 'percent';
		valueText = coupon ? String(coupon.discount_value) : '';
		capText = coupon?.max_redemptions == null ? '' : String(coupon.max_redemptions);
		perUserText = String(coupon?.per_user_limit ?? 1);
		startsAt = coupon?.starts_at ? toISODate(new Date(coupon.starts_at)) : '';
		endsAt = coupon?.ends_at ? toISODate(new Date(coupon.ends_at)) : '';
		planIds = coupon ? [...coupon.plan_ids] : [];
		minMonthsText = String(coupon?.min_months ?? 1);
		firstOnly = coupon?.first_payment_only ?? false;
		active = coupon?.is_active ?? true;
		sheetOpen = true;
	}

	function togglePlan(id: string, on: boolean) {
		planIds = on ? [...planIds, id] : planIds.filter((x) => x !== id);
	}

	const discountText = (c: AdminCoupon) => (c.discount_kind === 'percent' ? `${c.discount_value}%` : formatMoney(c.discount_value, CURRENCY, getLocale()));
	const dateText = (iso: string) => new Intl.DateTimeFormat(getLocale(), { day: 'numeric', month: 'short', year: 'numeric' }).format(new Date(iso));
	function validity(c: AdminCoupon): string {
		if (c.starts_at && c.ends_at) return m.billing_admin_coupon_valid_range({ from: dateText(c.starts_at), to: dateText(c.ends_at) });
		if (c.ends_at) return m.billing_admin_coupon_valid_until({ date: dateText(c.ends_at) });
		if (c.starts_at) return m.billing_admin_coupon_valid_from({ date: dateText(c.starts_at) });
		return m.billing_admin_coupon_always();
	}
	const planNames = (c: AdminCoupon) => c.plan_ids.map((id) => data.plans.find((p) => p.id === id)?.name ?? id).join(', ');
</script>

<PageHeader title={m.billing_admin_coupons()} lede={m.billing_admin_coupons_page_lede()} agent="money">
	{#snippet actions()}
		{#if canWrite}<Button type="button" variant="filled" onclick={() => load(null)}><IconPlus size={18} /> {m.billing_admin_coupon_add()}</Button>{/if}
	{/snippet}
</PageHeader>

{#if form && 'error' in form && form.error}
	<p class="error" role="alert">{form.error}</p>
{/if}

{#if data.coupons.length === 0}
	<p class="muted">{m.billing_admin_coupons_none()}</p>
{:else}
	<ListToolbar bind:query layouts={false} placeholder={m.billing_admin_coupons_search()} />
	{#if found.length === 0}
		<p class="muted">{m.list_no_match({ query })}</p>
	{:else}
		<ul class="rows">
			{#each found as coupon (coupon.id)}
				<li class="row" class:row--hidden={!coupon.is_active}>
					<div class="row__text">
						<span class="row__title"><code>{coupon.code}</code> · {discountText(coupon)}</span>
						<span class="row__meta">
							{coupon.max_redemptions === null
								? m.billing_admin_coupon_used({ count: coupon.redemptions })
								: m.billing_admin_coupon_used_of({ count: coupon.redemptions, cap: coupon.max_redemptions })}
							· {validity(coupon)}{#if !coupon.is_active} · {m.plans_admin_hidden()}{/if}
						</span>
						<span class="row__meta">
							{coupon.plan_ids.length > 0 ? m.billing_admin_coupon_only_plans({ plans: planNames(coupon) }) : m.billing_admin_coupon_any_plan()}
							{#if coupon.min_months > 1} · {m.billing_admin_coupon_from_months({ count: coupon.min_months })}{/if}
							{#if coupon.first_payment_only} · {m.billing_admin_coupon_first_only()}{/if}
						</span>
						{#if coupon.description}<span class="row__meta">{coupon.description}</span>{/if}
					</div>
					<div class="row__actions">
						{#if canWrite}<Button type="button" variant="tonal" size="xs" onclick={() => load(coupon)}>{m.dyn_edit()}</Button>{/if}
						{#if canManage}
							<form method="POST" action="?/delete" use:enhance>
								<input type="hidden" name="id" value={coupon.id} />
								<Button type="submit" variant="text" size="xs" class="danger">{m.common_delete()}</Button>
							</form>
						{/if}
					</div>
				</li>
			{/each}
		</ul>
	{/if}
{/if}

<BottomSheet bind:open={sheetOpen}>
	<h2 class="sheet-title">{editingId ? m.billing_admin_coupon_edit({ code }) : m.billing_admin_coupon_add()}</h2>
	<form
		method="POST"
		action="?/save"
		class="form"
		use:enhance={() =>
			async ({ result, update }) => {
				await update({ reset: false });
				if (result.type === 'success') sheetOpen = false;
			}}
	>
		<input type="hidden" name="id" value={editingId ?? ''} />
		<input type="hidden" name="first_payment_only" value={String(firstOnly)} />
		<input type="hidden" name="is_active" value={String(active)} />
		<div class="grid">
			<TextField id="coupon-code" name="code" label={m.billing_admin_coupon_code()} bind:value={code} autocapitalize="characters" required />
			<TextField id="coupon-description" name="description" label={m.plans_admin_description()} bind:value={description} />
		</div>
		<div class="grid">
			<Select label={m.billing_admin_coupon_kind()} name="discount_kind" options={kindOptions} bind:value={kind} />
			<TextField id="coupon-value" name="discount_value" type="number" min="1" max={kind === 'percent' ? 100 : undefined} step="1" label={m.billing_admin_coupon_value()} bind:value={valueText} required />
		</div>
		<div class="grid">
			<TextField id="coupon-cap" name="max_redemptions" type="number" min="1" step="1" label={m.billing_admin_coupon_cap()} placeholder={m.plans_admin_unlimited()} bind:value={capText} />
			<TextField id="coupon-per-user" name="per_user_limit" type="number" min="1" step="1" label={m.billing_admin_coupon_per_user()} bind:value={perUserText} />
		</div>
		<div class="grid">
			<DatePicker id="coupon-starts" name="starts_at" label={m.billing_admin_coupon_starts()} bind:value={startsAt} />
			<DatePicker id="coupon-ends" name="ends_at" label={m.billing_admin_coupon_ends()} bind:value={endsAt} min={startsAt || undefined} />
		</div>
		<fieldset class="plans">
			<legend class="field__label">{m.billing_admin_coupon_plans()}</legend>
			<p class="hint">{m.billing_admin_coupon_plans_hint()}</p>
			<div class="plans__list">
				{#each data.plans as plan (plan.id)}
					<Checkbox checked={planIds.includes(plan.id)} onchange={(e) => togglePlan(plan.id, (e.currentTarget as HTMLInputElement).checked)} label={plan.name} />
				{/each}
			</div>
			{#each planIds as id (id)}
				<input type="hidden" name="plan_ids" value={id} />
			{/each}
		</fieldset>
		<TextField id="coupon-min-months" name="min_months" type="number" min="1" step="1" label={m.billing_admin_coupon_min_months()} bind:value={minMonthsText} />
		<label class="switch-row">
			<Switch bind:checked={firstOnly} aria-label={m.billing_admin_coupon_first_only()} />
			{m.billing_admin_coupon_first_only()}
		</label>
		<label class="switch-row">
			<Switch bind:checked={active} aria-label={m.billing_admin_active()} />
			{m.billing_admin_active()}
		</label>
		<div class="actions">
			<Button type="submit" variant="filled">{m.common_save()}</Button>
			<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>{m.common_cancel()}</Button>
		</div>
	</form>
</BottomSheet>

<style>
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
		overflow-wrap: anywhere;
	}
	.muted,
	.hint {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.rows {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 16px;
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.row--hidden {
		opacity: 0.65;
	}
	.row__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__title {
		font-weight: 700;
		overflow-wrap: anywhere;
	}
	.row__meta {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		overflow-wrap: anywhere;
	}
	.row__actions {
		display: flex;
		gap: 8px;
	}
	.sheet-title {
		margin: 0 0 12px;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.3125rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.form {
		display: flex;
		flex-direction: column;
		gap: 12px;
		max-width: 620px;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
		gap: 12px;
	}
	.plans {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		border: none;
	}
	.plans__list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-height: 200px;
		overflow-y: auto;
		padding: 8px 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
	}
	.field__label {
		padding: 0;
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.switch-row {
		display: flex;
		align-items: center;
		gap: 10px;
		color: var(--md-sys-color-on-surface);
	}
	.actions {
		display: flex;
		gap: 8px;
	}
</style>
