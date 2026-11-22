<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import { CURRENCY } from '$lib/admin/billing';
	import { full, localParts } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Coupons → a coupon (plans: write; view shows it without saving): its code and what it takes
	// off, how often it can be used, the days it's valid (UTC, ending at the close of the last one),
	// which plans and how many months it needs, and whether it's on. Manage deletes one nobody used.
	let { data, form } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'plans', 'write'));
	const canManage = $derived(can(data.access, 'plans', 'manage'));
	const existing = $derived(data.coupon?.ok ? data.coupon.data : null);
	const back = { href: '/admin/billing/coupons', label: m.admin_coupons_back() };

	const initial = () => (data.coupon?.ok ? data.coupon.data : null);
	let kind = $state<string>(initial()?.discount_kind ?? 'percent');
	let planIds = $state<string[]>([...(initial()?.plan_ids ?? [])]);
	let firstOnly = $state(initial()?.first_payment_only ?? false);
	let active = $state(initial()?.is_active ?? true);
	let deleting = $state(false);
	let deleteForm: HTMLFormElement | undefined = $state();

	const kinds = [
		{ value: 'percent', label: m.admin_coupon_kind_percent() },
		{ value: 'amount', label: m.admin_coupon_kind_amount({ currency: CURRENCY }) },
	];
</script>

{#snippet save()}<Button label={existing ? m.admin_save() : m.admin_coupon_create()} variant="inverse" type="submit" />{/snippet}

{#if data.coupon && !data.coupon.ok}
	<PageHeader title={m.admin_page_coupons()} {back} />
	<LoadFailed title={m.admin_coupons_failed()} problem={data.coupon.problem} />
{:else}
	<PageHeader title={data.title} meta={existing ? m.admin_coupon_meta({ count: full(existing.redemptions, language) }) : undefined} {back} />

	<form method="POST" action="?/save" use:enhance={() => ({ update }) => update({ reset: false })} class="flex flex-col gap-6">
		<FormSection title={m.admin_coupon_details()} description={m.admin_coupon_details_desc()}>
			<div class="grid gap-4 sm:grid-cols-2">
				<TextField label={m.admin_coupon_code()} name="code" value={existing?.code ?? ''} helper={m.admin_coupon_code_helper()} mono required disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_coupon_description()} name="description" value={existing?.description ?? ''} disabled={!canWrite} autocomplete="off" />
				<Select label={m.admin_coupon_kind()} name="discount_kind" options={kinds} bind:value={kind} disabled={!canWrite} />
				<TextField
					label={m.admin_coupon_value()}
					name="discount_value"
					value={existing ? String(existing.discount_value) : ''}
					helper={kind === 'percent' ? m.admin_coupon_value_percent_helper() : m.admin_coupon_value_amount_helper()}
					mono
					required
					disabled={!canWrite}
					autocomplete="off"
				/>
			</div>
		</FormSection>

		<FormSection title={m.admin_coupon_limits()} description={m.admin_coupon_limits_desc()}>
			<div class="grid gap-4 sm:grid-cols-2">
				<TextField label={m.admin_coupon_cap()} name="max_redemptions" value={existing?.max_redemptions == null ? '' : String(existing.max_redemptions)} helper={m.admin_coupon_cap_helper()} mono disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_coupon_per_user()} name="per_user_limit" value={String(existing?.per_user_limit ?? 1)} mono disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_coupon_starts()} name="starts_at" value={localParts(existing?.starts_at, 'UTC').date} placeholder="2026-12-01" helper={m.admin_coupon_day_helper()} mono disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_coupon_ends()} name="ends_at" value={localParts(existing?.ends_at, 'UTC').date} placeholder="2026-12-31" helper={m.admin_coupon_day_helper()} mono disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_coupon_min_months()} name="min_months" value={String(existing?.min_months ?? 1)} helper={m.admin_coupon_min_months_helper()} mono disabled={!canWrite} autocomplete="off" />
			</div>
			<fieldset class="flex flex-col gap-2">
				<legend class="mb-2 text-ui font-medium text-text">{m.admin_coupon_plans()}</legend>
				<p class="text-caption text-text-3">{m.admin_coupon_plans_helper()}</p>
				<div class="flex flex-wrap gap-x-6 gap-y-2">
					{#each data.plans as plan (plan.id)}
						<Checkbox
							label={plan.name}
							checked={planIds.includes(plan.id)}
							disabled={!canWrite}
							onchange={(on) => (planIds = on ? [...planIds, plan.id] : planIds.filter((id) => id !== plan.id))}
						/>
					{/each}
				</div>
				{#each planIds as id (id)}<input type="hidden" name="plan_ids" value={id} />{/each}
			</fieldset>
		</FormSection>

		<FormSection title={m.admin_coupon_rules()} footer={canWrite ? save : undefined}>
			<div class="flex flex-col gap-4">
				<div class="flex flex-col gap-1">
					<div class="flex items-center justify-between gap-3">
						<span class="text-ui font-medium text-text" aria-hidden="true">{m.admin_coupon_first_only()}</span>
						<Toggle label={m.admin_coupon_first_only()} bind:checked={firstOnly} disabled={!canWrite} />
					</div>
					<p class="text-caption text-text-3">{m.admin_coupon_first_only_helper()}</p>
					<input type="hidden" name="first_payment_only" value={firstOnly ? 'on' : ''} />
				</div>
				<div class="flex flex-col gap-1">
					<div class="flex items-center justify-between gap-3">
						<span class="text-ui font-medium text-text" aria-hidden="true">{m.admin_coupon_active_label()}</span>
						<Toggle label={m.admin_coupon_active_label()} bind:checked={active} disabled={!canWrite} />
					</div>
					<p class="text-caption text-text-3">{m.admin_coupon_active_helper()}</p>
					<input type="hidden" name="is_active" value={active ? 'on' : ''} />
				</div>
			</div>
			<Outcome {form} action="save" done={m.admin_coupon_saved()} failed={m.admin_coupon_save_failed()} />
		</FormSection>
	</form>

	{#if existing && canManage}
		{#snippet remove()}<Button label={m.admin_coupon_delete()} variant="danger" onclick={() => (deleting = true)} />{/snippet}
		<FormSection title={m.admin_coupon_delete_section()} description={m.admin_coupon_delete_desc()} footer={remove} danger>
			<Outcome {form} action="delete" failed={m.admin_coupon_delete_failed()} />
		</FormSection>
		<form bind:this={deleteForm} method="POST" action="?/delete" use:enhance hidden></form>
		<ConfirmDialog
			bind:open={deleting}
			title={m.admin_coupon_delete_title({ code: existing.code })}
			body={m.admin_coupon_delete_body()}
			confirmLabel={m.admin_coupon_delete()}
			cancelLabel={m.admin_cancel()}
			onconfirm={() => {
				deleting = false;
				deleteForm?.requestSubmit();
			}}
		/>
	{/if}
{/if}
