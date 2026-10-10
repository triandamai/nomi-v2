<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import DatePicker from '$lib/components/m3/DatePicker.svelte';
	import { todayISO } from '$lib/dates';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import PlanCard from '$lib/components/PlanCard.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import { GRADIENT_STOPS, GRADIENT_TONES } from '$lib/components/m3/shapes';
	import type { Plan } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let sheetOpen = $state(false);
	let editingId = $state<string | null>(null);
	let draft = $state(blank());
	let featuresText = $state('');
	let promoEnds = $state('');
	let tokensText = $state('1000000');
	let orderText = $state('0');

	function blank(): Plan {
		return {
			id: '',
			slug: '',
			name: '',
			description: '',
			monthly_tokens: 1_000_000,
			price_label: '',
			features: [],
			card_tone: 'glow',
			promo_label: null,
			promo_price_label: null,
			promo_ends_at: null,
			is_default: false,
			is_active: true,
			sort_order: data.plans.length,
		};
	}

	function openCreate() {
		editingId = null;
		draft = blank();
		featuresText = '';
		promoEnds = '';
		tokensText = String(draft.monthly_tokens);
		orderText = String(draft.sort_order);
		slugTouched = false;
		sheetOpen = true;
	}

	function openEdit(plan: Plan) {
		editingId = plan.id;
		draft = { ...plan };
		featuresText = plan.features.join('\n');
		promoEnds = plan.promo_ends_at ? plan.promo_ends_at.slice(0, 10) : '';
		tokensText = String(plan.monthly_tokens);
		orderText = String(plan.sort_order);
		sheetOpen = true;
	}

	// The card as it will look, while the form is filled in.
	const preview = $derived<Plan>({
		...draft,
		name: draft.name || m.plans_admin_name(),
		monthly_tokens: Number(tokensText) || 0,
		features: featuresText.split('\n').map((f) => f.trim()).filter(Boolean),
		promo_ends_at: promoEnds ? `${promoEnds}T23:59:59` : null,
	});

	// A new plan's slug follows its name until it's edited by hand.
	let slugTouched = $state(false);
	$effect(() => {
		if (!editingId && !slugTouched) draft.slug = draft.name.toLowerCase().trim().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
	});
</script>

<PageHeader title={m.admin_plans()} lede={m.plans_admin_lede()} agent="money">
	{#snippet actions()}
		<Button type="button" variant="filled" onclick={openCreate}><IconPlus size={18} /> {m.plans_admin_add()}</Button>
	{/snippet}
</PageHeader>

{#if form && 'error' in form && form.error}
	<p class="error" role="alert">{form.error}</p>
{/if}

<div class="grid">
	{#each data.plans as plan (plan.id)}
		<div class="slot" class:slot--hidden={!plan.is_active}>
			<PlanCard {plan}>
				{#snippet actions()}
					<Button type="button" variant="tonal" size="xs" onclick={() => openEdit(plan)}>{m.dyn_edit()}</Button>
					{#if !plan.is_default}
						<form method="POST" action="?/setDefault" use:enhance>
							<input type="hidden" name="id" value={plan.id} />
							<Button type="submit" variant="text" size="xs">{m.plans_admin_make_default()}</Button>
						</form>
						{#if plan.subscribers === 0}
							<form method="POST" action="?/delete" use:enhance>
								<input type="hidden" name="id" value={plan.id} />
								<Button type="submit" variant="text" size="xs" class="danger">{m.common_delete()}</Button>
							</form>
						{/if}
					{/if}
				{/snippet}
			</PlanCard>
			<p class="slot__meta">
				{m.plans_admin_people({ count: plan.subscribers })}
				{#if plan.is_default} · {m.plans_admin_default()}{/if}
				{#if !plan.is_active} · {m.plans_admin_hidden()}{/if}
			</p>
		</div>
	{/each}
</div>

<BottomSheet bind:open={sheetOpen}>
	<h2 class="sheet-title">{editingId ? m.plans_admin_edit({ name: draft.name }) : m.plans_admin_add()}</h2>
	<div class="editor">
		<form
			method="POST"
			action="?/save"
			class="editor__form"
			use:enhance={() =>
				async ({ result, update }) => {
					await update({ reset: false });
					if (result.type === 'success') sheetOpen = false;
				}}
		>
			<input type="hidden" name="id" value={editingId ?? ''} />
			<input type="hidden" name="is_active" value={String(draft.is_active)} />
			<input type="hidden" name="card_tone" value={draft.card_tone} />
			<div class="row">
				<TextField id="plan-name" name="name" label={m.plans_admin_name()} bind:value={draft.name} required />
				<TextField id="plan-slug" name="slug" label={m.plans_admin_slug()} bind:value={draft.slug} oninput={() => (slugTouched = true)} required />
			</div>
			<TextField id="plan-description" name="description" label={m.plans_admin_description()} bind:value={draft.description} />
			<div class="row">
				<TextField id="plan-tokens" name="monthly_tokens" type="number" min="1" label={m.plans_admin_tokens()} bind:value={tokensText} required />
				<TextField id="plan-price" name="price_label" label={m.plans_admin_price()} bind:value={draft.price_label} placeholder="Rp 49.000 / month" />
			</div>
			<label class="field">
				<span class="field__label">{m.plans_admin_features()}</span>
				<textarea name="features" rows="4" bind:value={featuresText} placeholder={m.plans_admin_features_hint()}></textarea>
			</label>
			<fieldset class="tones">
				<legend class="field__label">{m.plans_admin_card()}</legend>
				<div class="tones__list">
					{#each GRADIENT_TONES as tone (tone)}
						<button
							type="button"
							class="tone"
							class:tone--on={draft.card_tone === tone}
							style:background={`linear-gradient(110deg, ${GRADIENT_STOPS[tone].join(', ')})`}
							aria-label={tone}
							aria-pressed={draft.card_tone === tone}
							onclick={() => (draft.card_tone = tone)}
						></button>
					{/each}
				</div>
			</fieldset>
			<fieldset class="promo">
				<legend class="field__label">{m.plans_admin_promo()}</legend>
				<div class="row">
					<TextField id="plan-promo" name="promo_label" label={m.plans_admin_promo_label()} value={draft.promo_label ?? ''} oninput={(e) => (draft.promo_label = (e.currentTarget as HTMLInputElement).value || null)} />
					<TextField id="plan-promo-price" name="promo_price_label" label={m.plans_admin_promo_price()} value={draft.promo_price_label ?? ''} oninput={(e) => (draft.promo_price_label = (e.currentTarget as HTMLInputElement).value || null)} />
				</div>
				<DatePicker id="plan-promo-ends" name="promo_ends_at" label={m.plans_admin_promo_ends()} bind:value={promoEnds} min={todayISO()} />
			</fieldset>
			<div class="row row--end">
				<TextField id="plan-order" name="sort_order" type="number" label={m.plans_admin_order()} bind:value={orderText} />
				<label class="switch-row">
					<Switch bind:checked={draft.is_active} aria-label={m.plans_admin_shown()} disabled={draft.is_default} />
					{m.plans_admin_shown()}
				</label>
			</div>
			<div class="actions">
				<Button type="submit" variant="filled">{editingId ? m.common_save() : m.plans_admin_add()}</Button>
				<Button type="button" variant="outlined" onclick={() => (sheetOpen = false)}>{m.common_cancel()}</Button>
			</div>
		</form>
		<div class="editor__preview" aria-label={m.plans_admin_preview()}>
			<p class="field__label">{m.plans_admin_preview()}</p>
			<PlanCard plan={preview} />
		</div>
	</div>
</BottomSheet>

<style>
	.error {
		margin: 0 0 12px;
		color: var(--md-sys-color-error);
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
		gap: 16px;
	}
	.slot {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.slot--hidden {
		opacity: 0.6;
	}
	.slot__meta {
		margin: 0 4px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.sheet-title {
		margin: 0 0 12px;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.editor {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 20px;
	}
	@media (min-width: 900px) {
		.editor {
			grid-template-columns: minmax(0, 1fr) 300px;
		}
	}
	.editor__form {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.editor__preview {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.row {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
		gap: 12px;
	}
	.row--end {
		align-items: center;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.field__label {
		padding: 0;
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.field textarea {
		padding: 12px 14px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-small);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		resize: vertical;
	}
	.tones,
	.promo {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 0;
		border: none;
	}
	.tones__list {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.tone {
		width: 44px;
		height: 44px;
		border: 3px solid transparent;
		border-radius: var(--md-sys-shape-corner-medium);
		cursor: pointer;
	}
	.tone--on {
		border-color: var(--md-sys-color-on-surface);
	}
	.tone:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
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
		padding-top: 4px;
	}
</style>
