<script lang="ts">
	import { Cpu, Plus } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import type { AdminModel } from '$lib/admin/api';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Models (models: view): which model new chats start on, which reads files people's own models
	// can't open, and which builds projects (write); then every model with where it runs, its id,
	// its price per million tokens, its roles, and whether it's on (write switches it).
	let { data, form } = $props();

	const canWrite = $derived(can(data.access, 'models', 'write'));
	const models = $derived(data.models.ok ? data.models.data : []);
	const current = $derived({
		default: models.find((model) => model.is_default)?.id ?? '',
		files: models.find((model) => model.is_files_model)?.id ?? '',
		coding: models.find((model) => model.is_coding_model)?.id ?? '',
	});
	const choices = $derived(models.filter((model) => model.enabled).map((model) => ({ value: model.id, label: model.label })));
	const optional = $derived([{ value: '', label: m.admin_models_none() }, ...choices]);

	const usd = (value: number | null) => (value === null ? '—' : `$${value}`);
	const columns: TableColumn<AdminModel>[] = [
		{ key: 'label', label: m.admin_col_name(), sort: (row) => row.label },
		{ key: 'provider', label: m.admin_col_provider(), sort: (row) => row.connection_label ?? row.provider, phone: false },
		{ key: 'model', label: m.admin_col_model_id(), phone: false },
		{ key: 'price', label: m.admin_col_price_mtok(), align: 'end', phone: false },
		{ key: 'roles', label: m.admin_col_roles() },
		{ key: 'on', label: m.admin_col_on(), align: 'end' },
	];
</script>

{#snippet saveRoles()}<Button label={m.admin_save()} variant="inverse" type="submit" />{/snippet}

<PageHeader title={m.admin_page_models()} intro={m.admin_models_intro()}>
	{#snippet actions()}
		{#if canWrite}<Button label={m.admin_models_add()} variant="inverse" iconStart={Plus} href="/admin/models/new" />{/if}
	{/snippet}
</PageHeader>

{#if data.models.ok}
	{#if models.length === 0}
		<EmptyState icon={Cpu} title={m.admin_models_empty_title()} body={m.admin_models_empty_body()} />
	{:else}
		{#if canWrite}
			<form method="POST" action="?/roles" use:enhance={() => ({ update }) => update({ reset: false })}>
				<FormSection title={m.admin_models_roles()} description={m.admin_models_roles_desc()} footer={saveRoles}>
					<input type="hidden" name="was_default" value={current.default} />
					<input type="hidden" name="was_files" value={current.files} />
					<input type="hidden" name="was_coding" value={current.coding} />
					<div class="grid gap-4 sm:grid-cols-3">
						<Select label={m.admin_models_default()} name="default_id" value={current.default} options={choices} />
						<Select label={m.admin_models_files()} name="files_id" value={current.files} options={optional} />
						<Select label={m.admin_models_coding()} name="coding_id" value={current.coding} options={optional} />
					</div>
					<Outcome {form} action="roles" done={m.admin_models_roles_saved()} failed={m.admin_models_roles_failed()} />
				</FormSection>
			</form>
		{/if}
		<Outcome {form} action="enable" failed={m.admin_model_enable_failed()} />
		<AdminTable label={m.admin_models_table()} {columns} rows={models} rowKey={(row) => row.id} muted={(row) => !row.enabled}>
			{#snippet cell(row, column)}
				{#if column.key === 'label'}
					<a href="/admin/models/{row.id}" class="rounded-xs font-medium text-text outline-offset-2 hover:underline">{row.label}</a>
				{:else if column.key === 'provider'}
					{row.connection_label ?? row.provider}
					{#if !row.connection_id}<span class="font-mono text-label text-text-3"> · {m.admin_model_own_key()}</span>{/if}
				{:else if column.key === 'model'}
					<span class="font-mono text-text-2">{row.model_id}</span>
				{:else if column.key === 'price'}
					<span class="font-mono">{usd(row.input_usd_per_mtok)} / {usd(row.output_usd_per_mtok)}</span>
				{:else if column.key === 'roles'}
					<span class="flex flex-wrap gap-1.5">
						{#if row.is_default}<Tag kind="neutral" label={m.admin_model_role_default()} />{/if}
						{#if row.is_files_model}<Tag kind="neutral" label={m.admin_model_role_files()} />{/if}
						{#if row.is_coding_model}<Tag kind="neutral" label={m.admin_model_role_coding()} />{/if}
					</span>
				{:else if column.key === 'on'}
					{#if canWrite}
						<form method="POST" action="?/enable" use:enhance class="flex justify-end">
							<input type="hidden" name="id" value={row.id} />
							<input type="hidden" name="enabled" value={row.enabled ? '' : 'on'} />
							<Button
								label={row.enabled ? m.admin_model_switch_off() : m.admin_model_switch_on()}
								variant="ghost"
								type="submit"
								aria-label={row.enabled ? m.admin_model_turn_off({ model: row.label }) : m.admin_model_turn_on({ model: row.label })}
							/>
						</form>
					{:else}
						<span class="font-mono text-label text-text-2">{row.enabled ? m.admin_model_on() : m.admin_model_off()}</span>
					{/if}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
{:else}
	<LoadFailed title={m.admin_models_failed()} problem={data.models.problem} />
{/if}
