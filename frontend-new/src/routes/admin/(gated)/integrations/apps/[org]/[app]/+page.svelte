<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import { dateTime, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Integrations → a Shipyard app (integrations: view): each environment with its services and
	// last publish (write duplicates one; manage deletes one), its secrets, written and never read
	// back (manage sets or deletes one), and deleting the app (manage).
	let { data, form } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'integrations', 'write'));
	const canManage = $derived(can(data.access, 'integrations', 'manage'));
	const environments = $derived(data.app.ok ? data.app.data.environments.map((env) => env.environment) : []);

	let pendingEnv = $state<string | null>(null);
	let envOpen = $state(false);
	let envForm: HTMLFormElement | undefined = $state();
	let appOpen = $state(false);
	let appForm: HTMLFormElement | undefined = $state();
</script>

<PageHeader title={data.title} meta={data.title} back={{ href: '/admin/integrations', label: m.admin_app_back() }} />

{#if data.app.ok}
	{@const app = data.app.data}
	<FormSection title={m.admin_app_environments()}>
		{#snippet aside()}{#if app.dev_database}<Tag kind="neutral" label={m.admin_app_dev_database()} />{/if}{/snippet}
		<ul class="flex flex-col gap-4">
			{#each app.environments as env (env.environment)}
				<li class="flex flex-col gap-1.5">
					<span class="flex flex-wrap items-center gap-3">
						<span class="font-mono text-ui text-text">{env.environment}</span>
						<span class="text-caption text-text-3">
							{env.last_publish ? m.admin_app_last_publish({ when: dateTime(env.last_publish.created_at, language), status: env.last_publish.status }) : m.admin_app_never_published()}
						</span>
						{#if canManage}
							<Button label={m.admin_app_delete_env({ env: env.environment })} variant="ghost-danger" onclick={() => ((pendingEnv = env.environment), (envOpen = true))} />
						{/if}
					</span>
					{#each env.services as service (service.name)}
						<span class="font-mono text-label text-text-2">
							{m.admin_app_service({ name: service.name, type: service.type, running: full(service.running, language), replicas: full(service.replicas, language), status: service.status })}
							{#each service.urls as url (url)} · {url}{/each}
						</span>
					{/each}
				</li>
			{/each}
		</ul>
		<Outcome {form} action="deleteEnv" done={m.admin_app_env_deleted()} failed={m.admin_app_env_failed()} />
	</FormSection>

	{#if canWrite && environments.length > 0}
		{#snippet duplicate()}<Button label={m.admin_app_duplicate_action()} variant="inverse" type="submit" />{/snippet}
		<form method="POST" action="?/duplicateEnv" use:enhance>
			<FormSection title={m.admin_app_duplicate()} footer={duplicate}>
				<div class="grid gap-4 sm:grid-cols-2">
					<Select label={m.admin_app_duplicate_from()} name="from" options={environments.map((env) => ({ value: env, label: env }))} mono />
					<TextField label={m.admin_app_duplicate_to()} name="to" placeholder="staging" mono required autocomplete="off" />
					<Checkbox label={m.admin_app_duplicate_secrets()} name="copy_secrets" checked />
				</div>
				<Outcome {form} action="duplicateEnv" done={m.admin_app_duplicated()} failed={m.admin_app_env_failed()} />
			</FormSection>
		</form>
	{/if}
{:else}
	<LoadFailed title={m.admin_app_failed()} problem={data.app.problem} />
{/if}

<FormSection title={m.admin_app_secrets()} description={m.admin_app_secrets_desc()}>
	<Outcome {form} action={['setSecret', 'deleteSecret']} done={form?.action === 'setSecret' ? m.admin_app_secret_saved() : m.admin_app_secret_deleted()} failed={m.admin_app_secret_failed()} />
	{#if data.secrets.ok}
		{#if data.secrets.data.length === 0}
			<p class="text-ui text-text-3">{m.admin_app_secrets_empty()}</p>
		{:else}
			<ul class="flex flex-col">
				{#each data.secrets.data as secret (secret.ref)}
					<li class="flex flex-wrap items-center gap-3 border-b border-border py-2.5 last:border-b-0">
						<span class="flex min-w-0 flex-1 basis-60 flex-col gap-0.5">
							<span class="font-mono text-ui text-text">{secret.name}</span>
							<span class="font-mono text-label text-text-3">
								{m.admin_app_secret_scope({ kind: secret.kind, environment: secret.environment ?? m.admin_app_every_env(), used: secret.used_by.join(', ') || '—' })} · {dateTime(secret.updated_at, language)}
							</span>
						</span>
						{#if canManage}
							<form method="POST" action="?/deleteSecret" use:enhance>
								<input type="hidden" name="name" value={secret.name} />
								<input type="hidden" name="environment" value={secret.environment ?? ''} />
								<Button label={m.admin_app_secret_delete({ name: secret.name })} variant="ghost-danger" type="submit" />
							</form>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	{:else}
		<LoadFailed title={m.admin_app_secrets_failed()} problem={data.secrets.problem} />
	{/if}
	{#if canManage}
		<form method="POST" action="?/setSecret" use:enhance class="flex flex-wrap items-end gap-3">
			<div class="min-w-48 flex-1"><TextField label={m.admin_app_secret_name()} name="name" placeholder="DATABASE_URL" mono required autocomplete="off" /></div>
			<div class="min-w-48 flex-1"><TextField label={m.admin_app_secret_value()} name="value" type="password" mono autocomplete="off" /></div>
			<div class="w-48">
				<Select label={m.admin_app_secret_env()} name="environment" options={[{ value: '', label: m.admin_app_every_env() }, ...environments.map((env) => ({ value: env, label: env }))]} mono />
			</div>
			<Button label={m.admin_app_secret_set()} variant="secondary" type="submit" />
		</form>
	{/if}
</FormSection>

{#if canManage}
	{#snippet remove()}<Button label={m.admin_app_delete()} variant="danger" onclick={() => (appOpen = true)} />{/snippet}
	<FormSection title={m.admin_app_delete_section()} description={m.admin_app_delete_desc()} footer={remove} danger>
		<Outcome {form} action="deleteApp" failed={m.admin_app_delete_failed()} />
	</FormSection>
{/if}

<form bind:this={envForm} method="POST" action="?/deleteEnv" use:enhance hidden><input type="hidden" name="env" value={pendingEnv ?? ''} /></form>
<ConfirmDialog
	bind:open={envOpen}
	title={m.admin_app_delete_env_title({ env: pendingEnv ?? '' })}
	body={m.admin_app_delete_env_body()}
	confirmLabel={m.admin_app_delete_env({ env: pendingEnv ?? '' })}
	cancelLabel={m.admin_cancel()}
	onconfirm={() => {
		envOpen = false;
		envForm?.requestSubmit();
	}}
/>
<form bind:this={appForm} method="POST" action="?/deleteApp" use:enhance hidden></form>
<ConfirmDialog
	bind:open={appOpen}
	title={m.admin_app_delete_title({ app: data.title })}
	body={m.admin_app_delete_body()}
	confirmLabel={m.admin_app_delete()}
	cancelLabel={m.admin_cancel()}
	onconfirm={() => {
		appOpen = false;
		appForm?.requestSubmit();
	}}
/>
