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
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import type { Plan } from '$lib/types';
	import { MB_PER_GB, formatStorage, parseAmount } from './planLimits';
	import { DEFAULT_CURRENCY } from '$lib/billing';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let sheetOpen = $state(false);
	let editingId = $state<string | null>(null);
	let draft = $state(blank());
	let featuresText = $state('');
	let promoEnds = $state('');
	let tokensText = $state('1000000');
	// Prices are typed as text so an empty promo price means none.
	let priceText = $state('0');
	let promoPriceText = $state('');
	let orderText = $state('0');
	// Limits are typed as text so an empty box can mean unlimited.
	let projectsText = $state('');
	let orgsText = $state('');
	let membersText = $state('');
	let crewText = $state('');
	let storageText = $state('');
	// '' is the platform default.
	let sandboxValue = $state('');
	let modelIds = $state<string[]>([]);

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
			price_amount: 0,
			currency: DEFAULT_CURRENCY,
			promo_price_amount: null,
			promo_label: null,
			promo_price_label: null,
			promo_ends_at: null,
			is_default: false,
			is_active: true,
			sort_order: data.plans.length,
			max_projects: null,
			max_orgs: null,
			max_org_members: null,
			max_custom_crew: null,
			max_storage_mb: null,
			custom_domain: true,
			sandbox_size: null,
			all_models: true,
			model_ids: [],
		};
	}

	function openCreate() {
		editingId = null;
		draft = blank();
		featuresText = '';
		promoEnds = '';
		tokensText = String(draft.monthly_tokens);
		priceText = String(draft.price_amount);
		promoPriceText = '';
		orderText = String(draft.sort_order);
		loadLimits(draft);
		slugTouched = false;
		sheetOpen = true;
	}

	function openEdit(plan: Plan) {
		editingId = plan.id;
		draft = { ...plan };
		featuresText = plan.features.join('\n');
		promoEnds = plan.promo_ends_at ? plan.promo_ends_at.slice(0, 10) : '';
		tokensText = String(plan.monthly_tokens);
		priceText = String(plan.price_amount);
		promoPriceText = plan.promo_price_amount === null ? '' : String(plan.promo_price_amount);
		orderText = String(plan.sort_order);
		loadLimits(plan);
		sheetOpen = true;
	}

	function loadLimits(plan: Plan) {
		const text = (value: number | null) => (value === null ? '' : String(value));
		projectsText = text(plan.max_projects);
		orgsText = text(plan.max_orgs);
		membersText = text(plan.max_org_members);
		crewText = text(plan.max_custom_crew);
		storageText = text(plan.max_storage_mb);
		sandboxValue = plan.sandbox_size ?? '';
		modelIds = [...plan.model_ids];
	}

	function toggleModel(id: string, on: boolean) {
		modelIds = on ? [...modelIds, id] : modelIds.filter((x) => x !== id);
	}

	const sandboxOptions = $derived([
		{ value: '', label: m.plans_admin_sandbox_platform() },
		{ value: 'small', label: m.plans_admin_sandbox_small() },
		{ value: 'default', label: m.plans_admin_sandbox_standard() },
		{ value: 'large', label: m.plans_admin_sandbox_large() },
	]);
	const sandboxLabel = (size: Plan['sandbox_size']) => sandboxOptions.find((o) => o.value === (size ?? ''))?.label ?? '';

	const storageMb = $derived(Number(storageText));
	const storageInGb = $derived(storageText.trim() !== '' && storageMb >= MB_PER_GB ? formatStorage(storageMb) : undefined);

	// One short line of what a plan allows, for the list.
	function limitsSummary(plan: Plan): string {
		const parts = [
			plan.max_projects === null ? m.plans_admin_sum_projects_unlimited() : m.plans_admin_sum_projects({ count: plan.max_projects }),
			plan.max_orgs === null ? m.plans_admin_sum_orgs_unlimited() : m.plans_admin_sum_orgs({ count: plan.max_orgs }),
			plan.max_org_members === null ? m.plans_admin_sum_members_unlimited() : m.plans_admin_sum_members({ count: plan.max_org_members }),
			plan.max_custom_crew === null ? m.plans_admin_sum_crew_unlimited() : m.plans_admin_sum_crew({ count: plan.max_custom_crew }),
			plan.max_storage_mb === null ? m.plans_admin_sum_storage_unlimited() : formatStorage(plan.max_storage_mb),
		];
		if (plan.custom_domain) parts.push(m.plans_admin_sum_custom_domain());
		if (plan.sandbox_size) parts.push(m.plans_admin_sum_sandbox({ size: sandboxLabel(plan.sandbox_size) }));
		parts.push(plan.all_models ? m.plans_admin_sum_models_all() : m.plans_admin_sum_models({ count: plan.model_ids.length }));
		return parts.join(' · ');
	}

	// The card as it will look, while the form is filled in.
	const preview = $derived<Plan>({
		...draft,
		name: draft.name || m.plans_admin_name(),
		monthly_tokens: Number(tokensText) || 0,
		price_amount: parseAmount(priceText) ?? 0,
		promo_price_amount: parseAmount(promoPriceText),
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
			<p class="slot__limits">{limitsSummary(plan)}</p>
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
			<div class="row">
				<TextField id="plan-price-amount" name="price_amount" type="number" min="0" step="1" label={m.plans_admin_price_amount()} supportingText={m.plans_admin_price_amount_hint()} bind:value={priceText} />
				<TextField id="plan-currency" name="currency" label={m.plans_admin_currency()} value={draft.currency} readonly />
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
				<TextField id="plan-promo-price-amount" name="promo_price_amount" type="number" min="0" step="1" label={m.plans_admin_promo_price_amount()} supportingText={m.plans_admin_promo_price_amount_hint()} bind:value={promoPriceText} />
				<DatePicker id="plan-promo-ends" name="promo_ends_at" label={m.plans_admin_promo_ends()} bind:value={promoEnds} min={todayISO()} />
			</fieldset>
			<fieldset class="limits">
				<legend class="field__label">{m.plans_admin_limits()}</legend>
				<p class="hint">{m.plans_admin_limits_hint()}</p>
				<div class="row">
					<TextField id="plan-max-projects" name="max_projects" type="number" min="0" label={m.plans_admin_max_projects()} placeholder={m.plans_admin_unlimited()} bind:value={projectsText} />
					<TextField id="plan-max-orgs" name="max_orgs" type="number" min="0" label={m.plans_admin_max_orgs()} placeholder={m.plans_admin_unlimited()} bind:value={orgsText} />
				</div>
				<div class="row">
					<TextField id="plan-max-members" name="max_org_members" type="number" min="0" label={m.plans_admin_max_org_members()} placeholder={m.plans_admin_unlimited()} bind:value={membersText} />
					<TextField id="plan-max-crew" name="max_custom_crew" type="number" min="0" label={m.plans_admin_max_custom_crew()} placeholder={m.plans_admin_unlimited()} bind:value={crewText} />
				</div>
				<div class="row">
					<TextField
						id="plan-max-storage"
						name="max_storage_mb"
						type="number"
						min="0"
						label={m.plans_admin_max_storage()}
						placeholder={m.plans_admin_unlimited()}
						supportingText={storageInGb ? m.plans_admin_storage_gb({ size: storageInGb }) : undefined}
						bind:value={storageText}
					/>
					<Select label={m.plans_admin_sandbox()} name="sandbox_size" options={sandboxOptions} bind:value={sandboxValue} />
				</div>
				<label class="switch-row">
					<Switch name="custom_domain" bind:checked={draft.custom_domain} aria-label={m.plans_admin_custom_domain()} />
					{m.plans_admin_custom_domain()}
				</label>
				<label class="switch-row">
					<Switch name="all_models" bind:checked={draft.all_models} aria-label={m.plans_admin_all_models()} />
					{m.plans_admin_all_models()}
				</label>
				{#if !draft.all_models}
					<div class="models" role="group" aria-label={m.plans_admin_models_pick()}>
						{#each data.models as model (model.id)}
							<Checkbox
								class="models__item"
								checked={modelIds.includes(model.id)}
								onchange={(e) => toggleModel(model.id, (e.currentTarget as HTMLInputElement).checked)}
								label={`${model.label} · ${model.provider}`}
							/>
						{:else}
							<p class="hint">{m.plans_admin_models_none()}</p>
						{/each}
					</div>
				{/if}
				{#each modelIds as id (id)}
					<input type="hidden" name="model_ids" value={id} />
				{/each}
				<p class="hint">{m.plans_admin_models_own_key()}</p>
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
		gap: 14px;
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
	.slot__limits {
		margin: 0 4px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface);
		overflow-wrap: anywhere;
	}
	.sheet-title {
		margin: 0 0 12px;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.3125rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.editor {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 16px;
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
		padding: 12px 12px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-small);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		resize: vertical;
	}
	.tones,
	.promo,
	.limits {
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
	.hint {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.models {
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-height: 200px;
		overflow-y: auto;
		padding: 8px 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
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
