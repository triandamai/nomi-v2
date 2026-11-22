<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Billing terms → a term (plans: write; view shows it without saving): how many months, what it
	// takes off, its name at checkout, its place in the list and whether it's offered. Manage
	// deletes one.
	let { data, form } = $props();

	const canWrite = $derived(can(data.access, 'plans', 'write'));
	const canManage = $derived(can(data.access, 'plans', 'manage'));
	const existing = $derived(data.term?.ok ? data.term.data : null);
	const back = { href: '/admin/billing/terms', label: m.admin_terms_back() };

	const initial = () => (data.term?.ok ? data.term.data : null);
	let active = $state(initial()?.is_active ?? true);
	let deleting = $state(false);
	let deleteForm: HTMLFormElement | undefined = $state();
</script>

{#snippet save()}<Button label={existing ? m.admin_save() : m.admin_term_create()} variant="inverse" type="submit" />{/snippet}

{#if data.term && !data.term.ok}
	<PageHeader title={m.admin_page_terms()} {back} />
	<LoadFailed title={m.admin_terms_failed()} problem={data.term.problem} />
{:else}
	<PageHeader title={data.title} {back} />

	<form method="POST" action="?/save" use:enhance={() => ({ update }) => update({ reset: false })}>
		<FormSection title={m.admin_term_details()} description={m.admin_term_details_desc()} footer={canWrite ? save : undefined}>
			<div class="grid gap-4 sm:grid-cols-2">
				<TextField label={m.admin_term_months_label()} name="months" value={existing ? String(existing.months) : ''} helper={m.admin_term_months_helper()} mono required disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_term_percent_label()} name="discount_percent" value={String(existing?.discount_percent ?? 0)} helper={m.admin_term_percent_helper()} mono disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_term_label_label()} name="label" value={existing?.label ?? ''} helper={m.admin_term_label_helper()} disabled={!canWrite} autocomplete="off" />
				<TextField label={m.admin_plan_order()} name="sort_order" value={String(existing?.sort_order ?? 0)} helper={m.admin_plan_order_helper()} mono disabled={!canWrite} autocomplete="off" />
				<div class="flex flex-col gap-1 sm:col-span-2">
					<div class="flex items-center justify-between gap-3">
						<span class="text-ui font-medium text-text" aria-hidden="true">{m.admin_term_active()}</span>
						<Toggle label={m.admin_term_active()} bind:checked={active} disabled={!canWrite} />
					</div>
					<p class="text-caption text-text-3">{m.admin_term_active_helper()}</p>
					<input type="hidden" name="is_active" value={active ? 'on' : ''} />
				</div>
			</div>
			<Outcome {form} action="save" done={m.admin_term_saved()} failed={m.admin_term_save_failed()} />
		</FormSection>
	</form>

	{#if existing && canManage}
		{#snippet remove()}<Button label={m.admin_term_delete()} variant="danger" onclick={() => (deleting = true)} />{/snippet}
		<FormSection title={m.admin_term_delete_section()} description={m.admin_term_delete_desc()} footer={remove} danger>
			<Outcome {form} action="delete" failed={m.admin_term_delete_failed()} />
		</FormSection>
		<form bind:this={deleteForm} method="POST" action="?/delete" use:enhance hidden></form>
		<ConfirmDialog
			bind:open={deleting}
			title={m.admin_term_delete_title({ term: data.title })}
			body={m.admin_term_delete_body()}
			confirmLabel={m.admin_term_delete()}
			cancelLabel={m.admin_cancel()}
			onconfirm={() => {
				deleting = false;
				deleteForm?.requestSubmit();
			}}
		/>
	{/if}
{/if}
