<script lang="ts">
	import { PlugZap } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import type { ConfigField } from '$lib/admin/api';
	import { TESTABLE, fieldLabel, sectionLabel } from '$lib/admin/configuration';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import SecretField from '$lib/ds/admin/SecretField.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Configuration (configuration: view; manage changes it): every setting Nomi reads, section by
	// section. A field shows what's saved here; one from the environment shows that value as its
	// placeholder, so saving the section doesn't copy it in. Emptied, a value goes back to the
	// environment's; an empty secret keeps the saved one. Storage and Xendit can be tested.
	let { data, form } = $props();

	const source = (field: ConfigField) => (field.source === 'saved' ? m.admin_config_from_saved() : field.source === 'environment' ? m.admin_config_from_env({ env: field.env ?? '' }) : m.admin_config_from_none());
	const secretStatus = (field: ConfigField) =>
		field.source === 'saved' ? m.admin_config_secret_saved() : field.source === 'environment' ? m.admin_config_secret_env({ env: field.env ?? '' }) : m.admin_config_secret_none();
	const mine = (action: string, section: string) =>
		form?.action === action && (form.ok ? (form.data as { section: string }).section === section : 'section' in form && form.section === section);
</script>

<PageHeader title={m.admin_page_configuration()} intro={m.admin_config_intro()} />

{#if data.configuration.ok}
	{@const canManage = data.configuration.data.can_manage}
	{#if !canManage}<Notice kind="info" text={m.admin_config_readonly()} />{/if}
	{#each data.configuration.data.sections as section (section.name)}
		{@const label = sectionLabel(section.name)}
		{#snippet actions()}
			{#if TESTABLE.includes(section.name)}
				<Button
					label={m.admin_config_test()}
					variant="secondary"
					iconStart={PlugZap}
					onclick={(event) => {
						const owner = (event.currentTarget as HTMLElement).closest('form');
						owner?.requestSubmit(owner.querySelector<HTMLButtonElement>('button[data-test]'));
					}}
				/>
			{/if}
			<Button label={m.admin_save()} variant="inverse" type="submit" />
		{/snippet}
		<form method="POST" action="?/save" use:enhance={() => ({ update }) => update({ reset: false })}>
			<input type="hidden" name="section" value={section.name} />
			<!-- "Test connection" sends this same form to the test action. -->
			<button type="submit" formaction="?/test" data-test hidden tabindex="-1" aria-hidden="true"></button>
			<FormSection title={label.title} description={label.description} footer={canManage ? actions : undefined}>
				<div class="grid gap-4 sm:grid-cols-2">
					{#each section.fields as field (field.name)}
						{#if field.kind === 'secret'}
							<SecretField label={fieldLabel(section.name, field.name)} name="value_{field.name}" set={field.set} status={secretStatus(field)} disabled={!canManage} />
						{:else if field.kind === 'choice'}
							<Select
								label={fieldLabel(section.name, field.name)}
								name="value_{field.name}"
								value={field.source === 'saved' ? (field.value ?? '') : ''}
								options={[{ value: '', label: field.source === 'environment' && field.value ? m.admin_config_env_value({ value: field.value }) : m.admin_config_choice_default() }, ...(field.choices ?? []).map((choice) => ({ value: choice, label: choice }))]}
								helper={source(field)}
								mono
								disabled={!canManage}
							/>
						{:else}
							<TextField
								label={fieldLabel(section.name, field.name)}
								name="value_{field.name}"
								value={field.source === 'saved' ? (field.value ?? '') : ''}
								placeholder={field.source === 'environment' ? (field.value ?? undefined) : undefined}
								helper={source(field)}
								mono={field.kind !== 'text'}
								disabled={!canManage}
								autocomplete="off"
							/>
						{/if}
					{/each}
				</div>
				{#if mine('save', section.name)}
					<Notice kind={form?.ok ? 'success' : 'error'} text={form?.ok ? m.admin_config_saved() : m.admin_config_save_failed()} detail={form && !form.ok ? form.problem?.message || undefined : undefined} />
				{:else if mine('test', section.name)}
					{@const result = form?.ok ? (form.data as { success: boolean; message: string }) : null}
					<Notice
						kind={result?.success ? 'success' : 'error'}
						text={result?.success ? m.admin_config_test_ok() : m.admin_config_test_failed()}
						detail={result?.message || (form && !form.ok ? form.problem?.message : undefined) || undefined}
					/>
				{/if}
			</FormSection>
		</form>
	{/each}
{:else}
	<LoadFailed title={m.admin_config_failed()} problem={data.configuration.problem} />
{/if}
