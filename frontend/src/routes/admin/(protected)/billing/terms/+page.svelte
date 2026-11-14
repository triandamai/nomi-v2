<script lang="ts">
	// Admin → Billing → Terms: the lengths of time a plan can be paid for, and what each takes off.
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import { termLabel } from '$lib/billing';
	import { can } from '$lib/permissions';
	import type { BillingTerm } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
	const canWrite = $derived(can(data.access, 'plans', 'write'));
	const canManage = $derived(can(data.access, 'plans', 'manage'));

	/** The example the form shows beside the fields. */
	const EXAMPLE_MONTHS = 6;
	const EXAMPLE_PERCENT = 10;

	let sheetOpen = $state(false);
	let editingId = $state<string | null>(null);
	let monthsText = $state('');
	let percentText = $state('0');
	let label = $state('');
	let orderText = $state('0');
	let active = $state(true);

	function openCreate() {
		editingId = null;
		monthsText = '';
		percentText = '0';
		label = '';
		orderText = String(data.terms.length);
		active = true;
		sheetOpen = true;
	}

	function openEdit(term: BillingTerm) {
		editingId = term.id;
		monthsText = String(term.months);
		percentText = String(term.discount_percent);
		label = term.label;
		orderText = String(term.sort_order);
		active = term.is_active;
		sheetOpen = true;
	}
</script>

<PageHeader title={m.billing_admin_terms()} lede={m.billing_admin_terms_page_lede()} agent="money">
	{#snippet actions()}
		{#if canWrite}<Button type="button" variant="filled" onclick={openCreate}><IconPlus size={18} /> {m.billing_admin_term_add()}</Button>{/if}
	{/snippet}
</PageHeader>

<p class="example">{m.billing_admin_terms_example({ months: EXAMPLE_MONTHS, percent: EXAMPLE_PERCENT })}</p>

{#if form && 'error' in form && form.error}
	<p class="error" role="alert">{form.error}</p>
{/if}

{#if data.terms.length === 0}
	<p class="muted">{m.billing_admin_terms_none()}</p>
{:else}
	<ul class="rows">
		{#each data.terms as term (term.id)}
			<li class="row" class:row--hidden={!term.is_active}>
				<div class="row__text">
					<span class="row__title">{termLabel(term.months)}{term.label ? ` · ${term.label}` : ''}</span>
					<span class="row__meta">
						{term.discount_percent > 0 ? m.billing_admin_term_off({ percent: term.discount_percent }) : m.billing_admin_term_no_discount()}
						· {m.billing_admin_term_order({ order: term.sort_order })}{#if !term.is_active} · {m.plans_admin_hidden()}{/if}
					</span>
				</div>
				<div class="row__actions">
					{#if canWrite}<Button type="button" variant="tonal" size="xs" onclick={() => openEdit(term)}>{m.dyn_edit()}</Button>{/if}
					{#if canManage}
						<form method="POST" action="?/delete" use:enhance>
							<input type="hidden" name="id" value={term.id} />
							<Button type="submit" variant="text" size="xs" class="danger">{m.common_delete()}</Button>
						</form>
					{/if}
				</div>
			</li>
		{/each}
	</ul>
{/if}

<BottomSheet bind:open={sheetOpen}>
	<h2 class="sheet-title">{editingId ? m.billing_admin_term_edit() : m.billing_admin_term_add()}</h2>
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
		<input type="hidden" name="is_active" value={String(active)} />
		<div class="grid">
			<TextField id="term-months" name="months" type="number" min="1" step="1" label={m.billing_admin_term_months()} bind:value={monthsText} required />
			<TextField id="term-percent" name="discount_percent" type="number" min="0" max="100" step="1" label={m.billing_admin_term_percent()} bind:value={percentText} required />
		</div>
		<TextField id="term-label" name="label" label={m.billing_admin_term_label()} bind:value={label} />
		<div class="grid grid--end">
			<TextField id="term-order" name="sort_order" type="number" step="1" label={m.plans_admin_order()} bind:value={orderText} />
			<label class="switch-row">
				<Switch bind:checked={active} aria-label={m.billing_admin_active()} />
				{m.billing_admin_active()}
			</label>
		</div>
		<p class="muted">{m.billing_admin_terms_example({ months: EXAMPLE_MONTHS, percent: EXAMPLE_PERCENT })}</p>
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
	.example {
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
		max-width: 520px;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
		gap: 12px;
	}
	.grid--end {
		align-items: center;
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
