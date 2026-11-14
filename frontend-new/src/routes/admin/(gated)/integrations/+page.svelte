<script lang="ts">
	import { ArrowRight } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import type { SyncedOrg, WebhookEndpoint } from '$lib/admin/api';
	import { dateTime, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import SecretReveal from '$lib/ds/admin/SecretReveal.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Link from '$lib/ds/actions/Link.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Integrations (integrations: view): Shipyard. Its connection and the webhook Nomi hears it
	// through (manage registers it; write catches up on missed events), its apps tied to projects
	// (write), the organizations synced to it (write syncs, manage removes), and the endpoints Nomi
	// forwards its events to (write switches and tests; manage adds, gives a new secret, deletes).
	// Secrets show once, when they're made.
	let { data, form } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'integrations', 'write'));
	const canManage = $derived(can(data.access, 'integrations', 'manage'));
	const orgs = $derived(data.orgs.ok ? data.orgs.data : []);

	const shown = $derived(
		(form?.action === 'createEndpoint' || form?.action === 'rotateEndpoint') && form.ok ? (form.data as { secret: string | null; own?: boolean }) : null,
	);
	let revealed = $state<string | null>(null);
	$effect(() => {
		revealed = shown?.secret ?? null;
	});
	const caught = $derived(form?.action === 'catchUp' && form.ok ? (form.data as { orgs: number; new_events: number; errors: string[] }) : null);

	// Confirms: removing an organization, deleting an endpoint.
	let removing = $state<SyncedOrg | null>(null);
	let deleting = $state<WebhookEndpoint | null>(null);
	let removeOpen = $state(false);
	let deleteOpen = $state(false);
	let removeForm: HTMLFormElement | undefined = $state();
	let deleteForm: HTMLFormElement | undefined = $state();
</script>

<PageHeader title={m.admin_page_integrations()} intro={m.admin_integrations_intro()} />

{#if data.status.ok}
	{@const status = data.status.data}
	{#snippet shipyardActions()}
		{#if canWrite}
			<form method="POST" action="?/catchUp" use:enhance><Button label={m.admin_shipyard_catch_up()} variant="secondary" type="submit" /></form>
		{/if}
		{#if canManage && !status.webhook_from_env}
			<form method="POST" action="?/registerWebhook" use:enhance><Button label={m.admin_shipyard_register()} variant="inverse" type="submit" /></form>
		{/if}
	{/snippet}
	<FormSection title={m.admin_shipyard()} footer={status.configured && (canWrite || canManage) ? shipyardActions : undefined}>
		{#snippet aside()}
			<Status kind={status.configured ? 'live' : 'idle'} label={status.configured ? m.admin_shipyard_connected() : m.admin_shipyard_not_configured()} />
		{/snippet}
		{#if !status.configured}
			<Notice kind="info" text={m.admin_shipyard_not_configured_body()} />
		{:else}
			<dl class="grid gap-x-6 gap-y-2 text-ui sm:grid-cols-2">
				<dt class="text-text-3">{m.admin_shipyard_webhook()}</dt>
				<dd class="flex flex-wrap items-center gap-2">
					{#if status.webhook_url}<span class="font-mono text-label break-all text-text">{status.webhook_url}</span>{/if}
					<Tag kind="neutral" label={status.webhook_registered ? (status.webhook_enabled === false ? m.admin_shipyard_webhook_off() : m.admin_shipyard_webhook_registered()) : m.admin_shipyard_webhook_unregistered()} />
					{#if status.webhook_from_env}<span class="font-mono text-label text-text-3">{m.admin_shipyard_webhook_env()}</span>{/if}
				</dd>
				<dt class="text-text-3">{m.admin_shipyard_registered_at()}</dt>
				<dd class="font-mono text-label text-text-2">{status.registered_at ? dateTime(status.registered_at, language) : m.admin_shipyard_never()}</dd>
				<dt class="text-text-3">{m.admin_shipyard_last_event()}</dt>
				<dd class="font-mono text-label text-text-2">{status.last_event_at ? dateTime(status.last_event_at, language) : m.admin_shipyard_never()}</dd>
			</dl>
		{/if}
		{#if caught}
			<Notice
				kind={caught.errors.length ? 'warning' : 'success'}
				text={m.admin_shipyard_caught_up({ count: full(caught.new_events, language), orgs: full(caught.orgs, language) })}
				detail={caught.errors.join(' · ') || undefined}
			/>
		{/if}
		<Outcome {form} action="catchUp" failed={m.admin_shipyard_catch_up_failed()} />
		<Outcome {form} action="registerWebhook" done={m.admin_shipyard_registered_done()} failed={m.admin_shipyard_register_failed()} />
		<Link href="/admin/integrations/events" label={m.admin_shipyard_events_link()} />
	</FormSection>
{:else}
	<LoadFailed title={m.admin_shipyard_failed()} problem={data.status.problem} />
{/if}

<section aria-label={m.admin_shipyard_apps()} class="flex flex-col gap-3">
	<SectionLabel label={m.admin_shipyard_apps()} count={data.apps.ok ? data.apps.data.apps.length : undefined} />
	<Outcome {form} action="link" done={m.admin_shipyard_linked()} failed={m.admin_shipyard_link_failed()} />
	{#if data.apps.ok}
		{@const overview = data.apps.data}
		{#each overview.errors as reason (reason)}<Notice kind="warning" text={reason} />{/each}
		{#if overview.apps.length === 0}
			<p class="text-ui text-text-3">{m.admin_shipyard_apps_empty()}</p>
		{:else}
			<ul class="flex flex-col rounded-md outline -outline-offset-1 outline-border">
				{#each overview.apps as app (`${app.org_id}/${app.key}`)}
					{@const candidates = overview.candidates.find((one) => one.org_id === app.org_id)?.projects ?? []}
					<li class="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-border px-4 py-3 last:border-b-0">
						<span class="flex min-w-0 flex-1 basis-60 flex-col gap-0.5">
							<a href="/admin/integrations/apps/{app.org_id}/{app.key}" class="rounded-xs text-ui font-medium text-text outline-offset-2 hover:underline">{app.name}</a>
							<span class="font-mono text-label text-text-3">{app.org_name} · {app.key} · {app.environments.join(', ')}</span>
						</span>
						{#if app.project}
							<span class="text-caption text-text-2">{m.admin_shipyard_app_project({ name: app.project.name })}</span>
						{:else if canWrite}
							<form method="POST" action="?/link" use:enhance class="flex flex-wrap items-end gap-2">
								<input type="hidden" name="org_id" value={app.org_id} />
								<input type="hidden" name="app" value={app.key} />
								<div class="w-56">
									<Select
										label={m.admin_shipyard_link_to({ app: app.name })}
										name="project_id"
										options={[{ value: '', label: m.admin_shipyard_new_project() }, ...candidates.map((project) => ({ value: project.id, label: project.name }))]}
									/>
								</div>
								<Button label={m.admin_shipyard_link()} variant="secondary" type="submit" />
							</form>
						{:else}
							<span class="text-caption text-text-3">{m.admin_shipyard_app_unlinked()}</span>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	{:else}
		<LoadFailed title={m.admin_shipyard_apps_failed()} problem={data.apps.problem} />
	{/if}
</section>

<section aria-label={m.admin_shipyard_orgs()} class="flex flex-col gap-3">
	<SectionLabel label={m.admin_shipyard_orgs()} count={data.orgs.ok ? orgs.length : undefined} />
	<Outcome {form} action={['syncOrg', 'removeOrg']} done={form?.action === 'syncOrg' ? m.admin_shipyard_synced() : m.admin_shipyard_removed()} failed={m.admin_shipyard_org_failed()} />
	{#if data.orgs.ok}
		{#if orgs.length === 0}
			<p class="text-ui text-text-3">{m.admin_shipyard_orgs_empty()}</p>
		{:else}
			<ul class="flex flex-col rounded-md outline -outline-offset-1 outline-border">
				{#each orgs as org (org.id)}
					<li class="flex flex-wrap items-center gap-x-4 gap-y-1 border-b border-border px-4 py-3 last:border-b-0">
						<span class="flex min-w-0 flex-1 basis-60 flex-col gap-0.5">
							<span class="text-ui font-medium text-text">{org.name}</span>
							<span class="font-mono text-label text-text-3">
								{m.admin_shipyard_org_counts({ projects: full(org.projects, language), published: full(org.published, language), when: org.synced_at ? dateTime(org.synced_at, language) : m.admin_shipyard_never() })}
							</span>
						</span>
						{#if canWrite}
							<form method="POST" action="?/syncOrg" use:enhance>
								<input type="hidden" name="id" value={org.id} />
								<Button label={m.admin_shipyard_sync()} variant="ghost" type="submit" aria-label={m.admin_shipyard_sync_named({ name: org.name })} />
							</form>
						{/if}
						{#if canManage}
							<Button label={m.admin_shipyard_remove()} variant="ghost-danger" aria-label={m.admin_shipyard_remove_named({ name: org.name })} onclick={() => ((removing = org), (removeOpen = true))} />
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	{:else}
		<LoadFailed title={m.admin_shipyard_orgs_failed()} problem={data.orgs.problem} />
	{/if}
</section>

<section aria-label={m.admin_endpoints()} class="flex flex-col gap-3">
	<SectionLabel label={m.admin_endpoints()} count={data.webhooks.ok ? data.webhooks.data.length : undefined} />
	<p class="text-ui text-text-2">{m.admin_endpoints_desc()}</p>
	{#if revealed}
		<SecretReveal title={m.admin_endpoint_secret()} secret={revealed} copyLabel={m.admin_copy()} copiedLabel={m.admin_copied()} doneLabel={m.admin_done()} ondone={() => (revealed = null)} />
		<p class="text-caption text-text-2">{m.admin_endpoint_secret_once()}</p>
	{:else if shown?.own}
		<Notice kind="success" text={form?.action === 'rotateEndpoint' ? m.admin_endpoint_rotated_own() : m.admin_endpoint_added_own()} />
	{/if}
	<Outcome {form} action="createEndpoint" failed={m.admin_endpoint_add_failed()} />
	<Outcome {form} action={['toggleEndpoint', 'rotateEndpoint']} failed={m.admin_endpoint_failed()} />
	<Outcome {form} action="testEndpoint" done={m.admin_endpoint_tested()} failed={m.admin_endpoint_failed()} />
	<Outcome {form} action="deleteEndpoint" done={m.admin_endpoint_deleted()} failed={m.admin_endpoint_failed()} />
	{#if data.webhooks.ok}
		{#if data.webhooks.data.length === 0}
			<p class="text-ui text-text-3">{m.admin_endpoints_empty()}</p>
		{:else}
			<ul class="flex flex-col rounded-md outline -outline-offset-1 outline-border">
				{#each data.webhooks.data as endpoint (endpoint.id)}
					<li class={['flex flex-wrap items-center gap-x-3 gap-y-2 border-b border-border px-4 py-3 last:border-b-0', !endpoint.enabled && 'opacity-60']}>
						<span class="flex min-w-0 flex-1 basis-72 flex-col gap-0.5">
							<span class="font-mono text-ui break-all text-text">{endpoint.url}</span>
							<span class="font-mono text-label text-text-3">{endpoint.events.join(', ')}{endpoint.org ? ` · ${orgs.find((one) => one.id === endpoint.org)?.name ?? endpoint.org}` : ''}</span>
						</span>
						<a
							href="/admin/integrations/webhooks/{endpoint.id}"
							class="inline-flex items-center gap-1 rounded-xs text-caption text-text-2 outline-offset-2 hover:text-text"
							aria-label={m.admin_endpoint_deliveries_named({ url: endpoint.url })}>{m.admin_endpoint_deliveries()} <Icon icon={ArrowRight} size={12} /></a
						>
						{#if canWrite}
							<form method="POST" action="?/testEndpoint" use:enhance>
								<input type="hidden" name="id" value={endpoint.id} />
								<Button label={m.admin_endpoint_test()} variant="ghost" type="submit" aria-label={m.admin_endpoint_test_named({ url: endpoint.url })} />
							</form>
							<form method="POST" action="?/toggleEndpoint" use:enhance>
								<input type="hidden" name="id" value={endpoint.id} />
								<input type="hidden" name="enabled" value={endpoint.enabled ? '' : 'on'} />
								<Button
									label={endpoint.enabled ? m.admin_model_switch_off() : m.admin_model_switch_on()}
									variant="ghost"
									type="submit"
									aria-label={endpoint.enabled ? m.admin_endpoint_turn_off({ url: endpoint.url }) : m.admin_endpoint_turn_on({ url: endpoint.url })}
								/>
							</form>
						{/if}
						{#if canManage}
							<form method="POST" action="?/rotateEndpoint" use:enhance>
								<input type="hidden" name="id" value={endpoint.id} />
								<Button label={m.admin_endpoint_rotate()} variant="ghost" type="submit" aria-label={m.admin_endpoint_rotate_named({ url: endpoint.url })} />
							</form>
							<Button label={m.admin_endpoint_delete()} variant="ghost-danger" aria-label={m.admin_endpoint_delete_named({ url: endpoint.url })} onclick={() => ((deleting = endpoint), (deleteOpen = true))} />
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	{:else}
		<LoadFailed title={m.admin_endpoints_failed()} problem={data.webhooks.problem} />
	{/if}
	{#if canManage}
		<form method="POST" action="?/createEndpoint" use:enhance class="flex flex-wrap items-end gap-3">
			<div class="min-w-60 flex-2"><TextField label={m.admin_endpoint_url()} name="url" placeholder="https://hooks.example.com/nomi" mono required autocomplete="off" /></div>
			<div class="min-w-60 flex-2"><TextField label={m.admin_endpoint_events()} name="events" placeholder="deployment.* publish.succeeded" helper={m.admin_endpoint_events_helper()} mono required autocomplete="off" /></div>
			<div class="min-w-48 flex-1"><Select label={m.admin_endpoint_org()} name="org" options={[{ value: '', label: m.admin_endpoint_any_org() }, ...orgs.map((org) => ({ value: org.id, label: org.name }))]} /></div>
			<Button label={m.admin_endpoint_add()} variant="secondary" type="submit" />
		</form>
	{/if}
</section>

<form bind:this={removeForm} method="POST" action="?/removeOrg" use:enhance hidden><input type="hidden" name="id" value={removing?.id ?? ''} /></form>
<ConfirmDialog
	bind:open={removeOpen}
	title={m.admin_shipyard_remove_title({ name: removing?.name ?? '' })}
	body={m.admin_shipyard_remove_body()}
	confirmLabel={m.admin_shipyard_remove()}
	cancelLabel={m.admin_cancel()}
	onconfirm={() => {
		removeOpen = false;
		removeForm?.requestSubmit();
	}}
/>
<form bind:this={deleteForm} method="POST" action="?/deleteEndpoint" use:enhance hidden><input type="hidden" name="id" value={deleting?.id ?? ''} /></form>
<ConfirmDialog
	bind:open={deleteOpen}
	title={m.admin_endpoint_delete_title()}
	body={m.admin_endpoint_delete_body({ url: deleting?.url ?? '' })}
	confirmLabel={m.admin_endpoint_delete()}
	cancelLabel={m.admin_cancel()}
	onconfirm={() => {
		deleteOpen = false;
		deleteForm?.requestSubmit();
	}}
/>
