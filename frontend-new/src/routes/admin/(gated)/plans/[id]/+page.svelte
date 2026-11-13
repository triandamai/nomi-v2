<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import { full, localParts, slugify } from '$lib/admin/format';
	import { CARD_TONES } from '$lib/admin/plans';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import TextArea from '$lib/ds/admin/TextArea.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Plans → a plan (plans: write; view shows it without saving): what people see when they choose
	// it, what it gives a month, whether it's offered, a promotion for a while, and deleting a plan
	// nobody has been on. A new plan's slug follows its name until the slug is typed in.
	let { data, form } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'plans', 'write'));
	const existing = $derived(data.plan?.ok ? data.plan.data : null);

	const initial = () => (data.plan?.ok ? data.plan.data : null);
	let name = $state(initial()?.name ?? '');
	let slug = $state(initial()?.slug ?? '');
	let slugTyped = $state(Boolean(initial()));
	let active = $state(initial()?.is_active ?? true);
	let deleting = $state(false);
	let deleteForm: HTMLFormElement | undefined = $state();

	const tones = CARD_TONES.map((tone) => ({ value: tone, label: tone }));
</script>

{#snippet save()}<Button label={existing ? m.admin_save() : m.admin_plan_create()} variant="inverse" type="submit" />{/snippet}

{#if data.plan && !data.plan.ok}
	<PageHeader title={m.admin_page_plans()} back={{ href: '/admin/plans', label: m.admin_plan_back() }} />
	<LoadFailed title={m.admin_plan_failed()} problem={data.plan.problem} />
{:else}
	<PageHeader
		title={existing?.name ?? m.admin_plan_new()}
		meta={existing ? m.admin_plan_meta({ slug: existing.slug, count: full(existing.subscribers, language) }) : undefined}
		back={{ href: '/admin/plans', label: m.admin_plan_back() }}
	/>

	<form method="POST" action="?/save" use:enhance={() => ({ update }) => update({ reset: false })} class="flex flex-col gap-6">
		<FormSection title={m.admin_plan_details()} description={m.admin_plan_details_desc()}>
			<div class="grid gap-4 sm:grid-cols-2">
				<TextField
					label={m.admin_plan_name()}
					name="name"
					bind:value={name}
					required
					disabled={!canWrite}
					autocomplete="off"
					oninput={(value) => {
						if (!slugTyped) slug = slugify(value);
					}}
				/>
				<TextField label={m.admin_plan_slug()} name="slug" bind:value={slug} helper={m.admin_plan_slug_helper()} mono required disabled={!canWrite} autocomplete="off" oninput={() => (slugTyped = true)} />
				<TextField label={m.admin_plan_tokens()} name="monthly_tokens" value={existing ? String(existing.monthly_tokens) : ''} helper={m.admin_plan_tokens_helper()} mono required disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_plan_price()} name="price_label" value={existing?.price_label ?? ''} helper={m.admin_plan_price_helper()} disabled={!canWrite} autocomplete="off" />
				<div class="sm:col-span-2"><TextArea label={m.admin_plan_description()} name="description" value={existing?.description ?? ''} disabled={!canWrite} /></div>
				<div class="sm:col-span-2"><TextArea label={m.admin_plan_features()} name="features" value={existing?.features.join('\n') ?? ''} helper={m.admin_plan_features_helper()} disabled={!canWrite} /></div>
				<Select label={m.admin_plan_tone()} name="card_tone" value={existing?.card_tone ?? 'glow'} options={tones} mono disabled={!canWrite} />
				<TextField label={m.admin_plan_order()} name="sort_order" value={String(existing?.sort_order ?? 0)} helper={m.admin_plan_order_helper()} mono disabled={!canWrite} autocomplete="off" />
				<div class="flex flex-col gap-1 sm:col-span-2">
					<Toggle label={m.admin_plan_active()} bind:checked={active} disabled={!canWrite} />
					<p class="text-caption text-text-3">{m.admin_plan_active_helper()}</p>
					<input type="hidden" name="is_active" value={active ? 'on' : ''} />
				</div>
			</div>
		</FormSection>

		<FormSection title={m.admin_plan_promo_section()} description={m.admin_plan_promo_desc()} footer={canWrite ? save : undefined}>
			<div class="grid gap-4 sm:grid-cols-3">
				<TextField label={m.admin_plan_promo_label()} name="promo_label" value={existing?.promo_label ?? ''} disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_plan_promo_price()} name="promo_price_label" value={existing?.promo_price_label ?? ''} disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_plan_promo_ends()} name="promo_ends_at" value={localParts(existing?.promo_ends_at, 'UTC').date} placeholder="2026-12-31" mono disabled={!canWrite} autocomplete="off" />
			</div>
			<Outcome {form} action="save" done={m.admin_plan_saved()} failed={m.admin_plan_save_failed()} />
		</FormSection>
	</form>

	{#if existing && canWrite}
		{#snippet remove()}<Button label={m.admin_plan_delete()} variant="danger" onclick={() => (deleting = true)} />{/snippet}
		<FormSection title={m.admin_plan_delete_section()} description={m.admin_plan_delete_desc()} footer={remove} danger>
			<Outcome {form} action="delete" failed={m.admin_plan_delete_failed()} />
		</FormSection>
		<form bind:this={deleteForm} method="POST" action="?/delete" use:enhance hidden></form>
		<ConfirmDialog
			bind:open={deleting}
			title={m.admin_plan_delete_title({ name: existing.name })}
			body={m.admin_plan_delete_body()}
			confirmLabel={m.admin_plan_delete()}
			cancelLabel={m.admin_cancel()}
			onconfirm={() => {
				deleting = false;
				deleteForm?.requestSubmit();
			}}
		/>
	{/if}
{/if}
