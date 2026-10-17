<script lang="ts">
	// Admin → Integrations: everything Nomi does with Shipyard, in one place. The connection and
	// its webhook, every webhook endpoint (on/off, test, new secret, deliveries), Nomi's
	// organizations there, every app with its environments and secrets, and the events received.
	// What each person can do follows their access to "integrations": view, write or manage.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import ButtonGroup from '$lib/components/m3/ButtonGroup.svelte';
	import Chip from '$lib/components/m3/Chip.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { adminApi } from '$lib/adminApi';
	import { can } from '$lib/permissions';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const canWrite = $derived(can(data.access, 'integrations', 'write'));
	const canManage = $derived(can(data.access, 'integrations', 'manage'));

	interface Status {
		configured: boolean;
		can_reveal: boolean;
		/** Nomi's endpoint as Shipyard lists it. */
		webhook_url: string | null;
		/** Whether Shipyard has an endpoint for Nomi; null when Shipyard couldn't be asked. */
		webhook_registered: boolean | null;
		webhook_enabled: boolean | null;
		/** Where registering points Shipyard; null until APP_URL is set. */
		webhook_target: string | null;
		webhook_from_env: boolean;
		registered_at: string | null;
		last_event_at: string | null;
	}
	interface App { org_id: string; org_name: string; key: string; name: string; environments: string[]; project: { id: string; name: string; session_id: string } | null }
	interface Overview { apps: App[]; candidates: { org_id: string; projects: { id: string; name: string }[] }[]; errors: string[] }
	interface Endpoint { id: string; url: string; events: string[]; enabled: boolean; org: string | null; created_at: string }
	interface Delivery { id: string; event_id: string; event_type: string; status: string; attempts: number; last_status_code: number | null; last_error: string | null; created_at: string; delivered_at: string | null }
	interface SyncedOrg { id: string; name: string; synced_at: string | null; projects: number; published: number }
	interface Service { name: string; type: string; status: string; replicas: number; running: number; urls: string[] }
	interface Env { environment: string; services: Service[]; last_publish: { status: string; urls: string[]; created_at: string } | null }
	interface AppDetail { key: string; environments: Env[]; dev_database: boolean }
	interface Secret { ref: string; name: string; kind: string; environment: string | null; used_by: string[]; updated_at: string }
	interface ReceivedEvent { id: string; type: string; project_name: string | null; environment: string | null; service: string | null; created_at: string; received_at: string }

	type Tab = 'overview' | 'webhooks' | 'orgs' | 'apps' | 'events';
	const TABS: { id: Tab; label: () => string }[] = [
		{ id: 'overview', label: () => m.integ_tab_overview() },
		{ id: 'webhooks', label: () => m.integ_tab_webhooks() },
		{ id: 'orgs', label: () => m.integ_tab_orgs() },
		{ id: 'apps', label: () => m.integ_tab_apps() },
		{ id: 'events', label: () => m.integ_tab_events() },
	];
	let tab = $state<Tab>('overview');

	let status = $state<Status | null>(null);
	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);
	let busy = $state<string | null>(null);

	const when = (at: string | null) => (at ? new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(at)) : '—');

	async function run<T>(key: string, work: () => Promise<T>): Promise<T | undefined> {
		busy = key;
		error = null;
		notice = null;
		try {
			return await work();
		} catch (e) {
			error = (e as Error).message;
		} finally {
			busy = null;
		}
	}

	// A destructive step waits for a yes in a dialog.
	let confirmOpen = $state(false);
	let confirmText = $state('');
	let confirmAction: (() => Promise<void>) | null = null;
	function confirmThen(text: string, action: () => Promise<void>) {
		confirmText = text;
		confirmAction = action;
		confirmOpen = true;
	}

	// ─── Overview ────────────────────────────────────────────────
	async function loadStatus() {
		await run('status', async () => {
			status = await adminApi<Status>('shipyard');
		});
	}
	$effect(() => {
		void loadStatus();
	});

	// Shipyard's own list decides: missing there (never registered, or deleted) means no events.
	const webhookMissing = $derived(status?.configured === true && status.webhook_registered === false);
	const webhookLabel = (s: Status) =>
		s.webhook_registered === null
			? m.integ_webhook_unknown()
			: !s.webhook_url
				? m.integ_webhook_not_on_shipyard()
				: s.webhook_enabled === false
					? m.integ_webhook_off({ url: s.webhook_url })
					: s.webhook_url;

	async function registerWebhook() {
		const registered = await run('register', async () => {
			status = await adminApi<Status>('shipyard/webhook', { method: 'POST' });
			return true;
		});
		if (!registered) return;
		// Reloading the list clears messages, so the confirmation comes after.
		if (webhooks) await loadWebhooks();
		notice = m.integ_registered();
	}

	async function catchUp() {
		await run('catchup', async () => {
			const done = await adminApi<{ orgs: number; new_events: number; errors: string[] }>('shipyard/events/catch-up', { method: 'POST' });
			notice = m.integ_caught_up({ events: done.new_events, orgs: done.orgs });
			if (done.errors.length) error = done.errors.join('\n');
			events = null;
		});
	}

	// ─── Webhooks ────────────────────────────────────────────────
	let webhooks = $state<{ endpoints: Endpoint[]; own_id: string | null } | null>(null);
	let openDeliveries = $state<string | null>(null);
	let deliveries = $state<Delivery[]>([]);
	let deliveryFilter = $state('');
	let newSecret = $state<string | null>(null);

	async function loadWebhooks() {
		await run('webhooks', async () => {
			webhooks = await adminApi('shipyard/webhooks');
		});
	}

	async function toggleEndpoint(endpoint: Endpoint, enabled: boolean) {
		await run(endpoint.id, async () => {
			await adminApi(`shipyard/webhooks/${endpoint.id}`, { method: 'PATCH', body: { enabled } });
			endpoint.enabled = enabled;
		});
	}

	async function testEndpoint(endpoint: Endpoint) {
		await run(endpoint.id, async () => {
			await adminApi(`shipyard/webhooks/${endpoint.id}/test`, { method: 'POST' });
			notice = m.integ_test_sent();
		});
	}

	function rotateEndpoint(endpoint: Endpoint) {
		confirmThen(m.integ_rotate_confirm(), async () => {
			await run(endpoint.id, async () => {
				const rotated = await adminApi<{ secret: string | null }>(`shipyard/webhooks/${endpoint.id}/rotate`, { method: 'POST' });
				newSecret = rotated.secret;
				notice = rotated.secret ? null : m.integ_rotated_own();
			});
		});
	}

	function deleteEndpoint(endpoint: Endpoint) {
		confirmThen(m.integ_delete_endpoint_confirm({ url: endpoint.url }), async () => {
			await run(endpoint.id, async () => {
				await adminApi(`shipyard/webhooks/${endpoint.id}`, { method: 'DELETE' });
				await loadWebhooks();
				await loadStatus();
			});
		});
	}

	async function loadDeliveries(id: string) {
		await run(`deliveries-${id}`, async () => {
			const query = deliveryFilter ? `?status=${deliveryFilter}` : '';
			deliveries = await adminApi<Delivery[]>(`shipyard/webhooks/${id}/deliveries${query}`);
			openDeliveries = id;
		});
	}

	async function redeliver(delivery: Delivery) {
		await run(delivery.id, async () => {
			await adminApi(`shipyard/deliveries/${delivery.id}/redeliver`, { method: 'POST' });
			notice = m.integ_redelivery_queued();
			if (openDeliveries) await loadDeliveries(openDeliveries);
		});
	}

	// ─── Organizations ───────────────────────────────────────────
	let orgs = $state<SyncedOrg[] | null>(null);
	async function loadOrgs() {
		await run('orgs', async () => {
			orgs = await adminApi<SyncedOrg[]>('shipyard/orgs');
		});
	}
	async function syncOrg(org: SyncedOrg) {
		await run(org.id, async () => {
			await adminApi(`shipyard/orgs/${org.id}/sync`, { method: 'POST' });
			await loadOrgs();
		});
	}
	function removeOrg(org: SyncedOrg) {
		confirmThen(m.integ_remove_org_confirm({ name: org.name }), async () => {
			await run(org.id, async () => {
				await adminApi(`shipyard/orgs/${org.id}`, { method: 'DELETE' });
				await loadOrgs();
				overview = null;
			});
		});
	}

	// ─── Apps ────────────────────────────────────────────────────
	let overview = $state<Overview | null>(null);
	let choice = $state<Record<string, string>>({});
	async function loadApps() {
		await run('apps', async () => {
			overview = await adminApi<Overview>('shipyard/apps');
		});
	}
	async function link(app: App) {
		await run(app.key, async () => {
			await adminApi('shipyard/apps/link', { method: 'POST', body: { org_id: app.org_id, app: app.key, project_id: choice[app.key] || null } });
			await loadApps();
		});
	}
	const candidatesFor = (orgId: string) => overview?.candidates.find((c) => c.org_id === orgId)?.projects ?? [];
	const orphans = $derived(overview?.apps.filter((a) => !a.project).length ?? 0);

	let sheetOpen = $state(false);
	let current = $state<App | null>(null);
	let detail = $state<AppDetail | null>(null);
	let secrets = $state<Secret[]>([]);
	let dupFrom = $state('');
	let dupTo = $state('staging');
	let dupCopy = $state(true);
	let secretName = $state('');
	let secretValue = $state('');
	let secretEnv = $state('');
	let secretGenerate = $state(false);

	const appPath = (app: App) => `shipyard/orgs/${app.org_id}/apps/${app.key}`;

	async function openApp(app: App) {
		current = app;
		detail = null;
		secrets = [];
		sheetOpen = true;
		await reloadApp();
	}
	async function reloadApp() {
		const app = current;
		if (!app) return;
		await run('app', async () => {
			const [d, s] = await Promise.all([adminApi<AppDetail>(appPath(app)), adminApi<Secret[]>(`${appPath(app)}/secrets`)]);
			detail = d;
			secrets = s;
			dupFrom = d.environments[0]?.environment ?? '';
		});
	}
	async function duplicate() {
		const app = current;
		if (!app || !dupFrom) return;
		await run('duplicate', async () => {
			await adminApi(`${appPath(app)}/envs/${dupFrom}/duplicate`, { method: 'POST', body: { to: dupTo.trim(), copy_secrets: dupCopy } });
			notice = m.integ_duplicating({ to: dupTo.trim() });
			await reloadApp();
		});
	}
	function deleteEnv(env: string) {
		const app = current;
		if (!app) return;
		confirmThen(m.integ_delete_env_confirm({ env }), async () => {
			await run(`env-${env}`, async () => {
				await adminApi(`${appPath(app)}/envs/${env}`, { method: 'DELETE' });
				await reloadApp();
			});
		});
	}
	async function saveSecret() {
		const app = current;
		if (!app) return;
		await run('secret', async () => {
			await adminApi(`${appPath(app)}/secrets/${secretName.trim()}`, {
				method: 'PUT',
				body: { value: secretGenerate ? null : secretValue, environment: secretEnv || null },
			});
			secretName = '';
			secretValue = '';
			secretGenerate = false;
			notice = m.integ_secret_saved();
			secrets = await adminApi<Secret[]>(`${appPath(app)}/secrets`);
		});
	}
	function deleteSecret(secret: Secret) {
		const app = current;
		if (!app) return;
		confirmThen(m.integ_delete_secret_confirm({ name: secret.name }), async () => {
			await run(secret.ref, async () => {
				const scope = secret.environment ? `?environment=${secret.environment}` : '';
				await adminApi(`${appPath(app)}/secrets/${secret.name}${scope}`, { method: 'DELETE' });
				secrets = await adminApi<Secret[]>(`${appPath(app)}/secrets`);
			});
		});
	}
	function terminateApp() {
		const app = current;
		if (!app) return;
		confirmThen(m.integ_terminate_confirm({ name: app.name }), async () => {
			await run('terminate', async () => {
				await adminApi(appPath(app), { method: 'DELETE' });
				sheetOpen = false;
				await loadApps();
			});
		});
	}
	const SECRET_NAME = /^[A-Z_][A-Z0-9_]*$/;

	// ─── Events ──────────────────────────────────────────────────
	let events = $state<{ events: ReceivedEvent[]; types: string[] } | null>(null);
	async function loadEvents() {
		await run('events', async () => {
			events = await adminApi('shipyard/events');
		});
	}

	// ─── Add endpoint ────────────────────────────────────────────
	// Registers an address with Shipyard (POST /webhooks): every event or chosen ones, for every
	// organization or one. Nomi's own address keeps its secret; any other shows it once here.
	let addOpen = $state(false);
	let addUrl = $state('');
	let addAll = $state<'all' | 'some'>('all');
	let addEvents = $state<string[]>([]);
	let addOrg = $state('');
	let addError = $state<string | null>(null);
	let secretCopied = $state(false);
	// Prefixes first (deployment.*), then every exact type Shipyard lists.
	const eventChoices = $derived.by(() => {
		const types = events?.types ?? [];
		const prefixes = [...new Set(types.filter((t) => t.includes('.')).map((t) => `${t.split('.')[0]}.*`))];
		return [...prefixes, ...types];
	});
	const orgChoices = $derived([{ value: '', label: m.integ_add_any_org() }, ...(orgs ?? []).filter((o) => o.synced_at).map((o) => ({ value: o.id, label: o.name }))]);

	async function openAddEndpoint() {
		addUrl = webhooks?.own_id ? '' : (status?.webhook_target ?? '');
		addAll = 'all';
		addEvents = [];
		addOrg = '';
		addError = null;
		addOpen = true;
		// What to choose from: Shipyard's event types and the organizations it knows.
		if (!events) void loadEvents();
		if (!orgs) void loadOrgs();
	}

	function toggleEvent(event: string) {
		addEvents = addEvents.includes(event) ? addEvents.filter((e) => e !== event) : [...addEvents, event];
	}

	async function addEndpoint() {
		addError = null;
		busy = 'add';
		try {
			const created = await adminApi<{ id: string; secret: string | null; own: boolean }>('shipyard/webhooks', {
				method: 'POST',
				body: { url: addUrl.trim(), events: addAll === 'all' ? ['*'] : addEvents, org: addOrg || null },
			});
			addOpen = false;
			newSecret = created.secret;
			secretCopied = false;
			// Reloading clears messages, so the confirmation comes after.
			await loadWebhooks();
			await loadStatus();
			notice = created.own ? m.integ_registered() : null;
		} catch (e) {
			addError = (e as Error).message;
		} finally {
			busy = null;
		}
	}

	async function copySecret() {
		if (!newSecret) return;
		await navigator.clipboard.writeText(newSecret).catch(() => {});
		secretCopied = true;
	}

	// Each tab loads the first time it's opened.
	$effect(() => {
		if (!status?.configured) return;
		if (tab === 'webhooks' && !webhooks) void loadWebhooks();
		if (tab === 'orgs' && !orgs) void loadOrgs();
		if (tab === 'apps' && !overview) void loadApps();
		if (tab === 'events' && !events) void loadEvents();
	});
</script>

{#snippet webhookSetup(s: Status)}
	<div class="setup">
		<span class="setup__text">
			<strong>{m.integ_webhook_missing()}</strong>
			<span class="muted small">{m.integ_webhook_missing_hint()}</span>
			<span class="muted small break">{s.webhook_target ? m.integ_webhook_target({ url: s.webhook_target }) : m.integ_webhook_needs_app_url()}</span>
		</span>
		{#if canManage}
			<Button variant="filled" size="s" disabled={busy === 'register' || !s.webhook_target} onclick={registerWebhook}>{m.integ_register_webhook()}</Button>
		{/if}
	</div>
{/snippet}

<PageHeader title={m.admin_integrations()} lede={m.integ_lede()} agent="coding" />

<section class="integration">
	<header class="integration__head">
		<span class="integration__name">Shipyard</span>
		<span class="integration__what">{m.integ_shipyard_what()}</span>
		{#if status}
			<span class="chip" data-on={status.configured}>{status.configured ? m.shipyard_connected() : m.shipyard_not_configured()}</span>
		{/if}
	</header>

	<div class="tabs" role="tablist" aria-label="Shipyard">
		{#each TABS as t (t.id)}
			<button
				type="button"
				role="tab"
				class="tab"
				aria-selected={tab === t.id}
				disabled={t.id !== 'overview' && !status?.configured}
				onclick={() => (tab = t.id)}>{t.label()}</button
			>
		{/each}
	</div>

	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#if notice}<p class="notice" role="status">{notice}</p>{/if}
	{#if newSecret}
		<div class="secret-once" role="status">
			<span>{m.integ_new_secret()}</span>
			<code>{newSecret}</code>
			<Button variant="text" size="s" onclick={copySecret}>{secretCopied ? m.integ_copied() : m.integ_copy()}</Button>
			<Button variant="text" size="s" onclick={() => (newSecret = null)}>{m.integ_done()}</Button>
		</div>
	{/if}

	{#if tab === 'overview'}
		{#if status}
			<dl class="facts">
				<div><dt>{m.shipyard_connection()}</dt><dd>{status.configured ? m.shipyard_connected() : m.shipyard_not_configured()}</dd></div>
				<div><dt>{m.shipyard_webhook()}</dt><dd class="break">{webhookLabel(status)}</dd></div>
				<div><dt>{m.shipyard_last_event()}</dt><dd>{when(status.last_event_at)}</dd></div>
				<div><dt>{m.shipyard_db_access()}</dt><dd>{status.can_reveal ? m.shipyard_db_on() : m.shipyard_db_off()}</dd></div>
			</dl>
			{#if !status.configured}
				<p class="muted">{m.integ_setup_hint()}</p>
			{:else}
				{#if webhookMissing}
					{@render webhookSetup(status)}
				{:else if status.webhook_enabled === false}
					<p class="muted small">{m.integ_webhook_switched_off()}</p>
				{/if}
				<div class="actions">
					{#if canWrite}
						<Button variant="outlined" size="s" disabled={busy === 'catchup'} onclick={catchUp}>{m.integ_catch_up()}</Button>
					{/if}
				</div>
				<p class="muted small">{m.integ_catch_up_hint()}</p>
			{/if}
		{/if}
	{:else if tab === 'webhooks'}
		{#if webhooks}
			{#if canManage}
				<div class="actions">
					<Button variant="tonal" size="s" onclick={openAddEndpoint}>{m.integ_add_endpoint()}</Button>
				</div>
			{/if}
			<!-- Nomi's own endpoint isn't on Shipyard (never registered, or deleted there): put it back. -->
			{#if !webhooks.own_id && status}
				{@render webhookSetup(status)}
			{/if}
			{#if webhooks.endpoints.length === 0}
				<p class="muted">{m.integ_no_endpoints()}</p>
			{:else}
				<ul class="list">
					{#each webhooks.endpoints as endpoint (endpoint.id)}
						<li class="row">
							<div class="row__main">
								<div class="row__text">
									<span class="row__title">{endpoint.url}{#if endpoint.id === webhooks.own_id}<span class="tag">{m.integ_own_endpoint()}</span>{/if}</span>
									<span class="row__meta">{endpoint.events.includes('*') ? m.integ_all_events() : endpoint.events.join(', ')} · {when(endpoint.created_at)}</span>
								</div>
								<Switch checked={endpoint.enabled} disabled={!canWrite || busy === endpoint.id} aria-label={m.integ_endpoint_on({ url: endpoint.url })} onchange={(on) => toggleEndpoint(endpoint, on)} />
							</div>
							<div class="actions">
								<Button variant="text" size="s" onclick={() => (openDeliveries === endpoint.id ? (openDeliveries = null) : loadDeliveries(endpoint.id))}>{m.integ_deliveries()}</Button>
								{#if canWrite}<Button variant="text" size="s" disabled={busy === endpoint.id} onclick={() => testEndpoint(endpoint)}>{m.integ_test()}</Button>{/if}
								{#if canManage}
									<Button variant="text" size="s" disabled={busy === endpoint.id} onclick={() => rotateEndpoint(endpoint)}>{m.integ_rotate()}</Button>
									<Button variant="text" size="s" disabled={busy === endpoint.id} onclick={() => deleteEndpoint(endpoint)}>{m.common_delete()}</Button>
								{/if}
							</div>
							{#if openDeliveries === endpoint.id}
								<div class="sub">
									<label class="filter">
										<span>{m.integ_status()}</span>
										<select bind:value={deliveryFilter} onchange={() => loadDeliveries(endpoint.id)}>
											<option value="">{m.integ_any()}</option>
											<option value="pending">pending</option>
											<option value="delivered">delivered</option>
											<option value="failed">failed</option>
										</select>
									</label>
									{#if deliveries.length === 0}
										<p class="muted small">{m.integ_no_deliveries()}</p>
									{:else}
										<ul class="list">
											{#each deliveries as delivery (delivery.id)}
												<li class="mini">
													<span class="mini__text">
														<span class="mini__title"><code>{delivery.event_type}</code> <span class="state" data-state={delivery.status}>{delivery.status}</span></span>
														<span class="row__meta">{when(delivery.created_at)} · {m.integ_attempts({ count: delivery.attempts })}{delivery.last_status_code ? ` · HTTP ${delivery.last_status_code}` : ''}{delivery.last_error ? ` · ${delivery.last_error}` : ''}</span>
													</span>
													{#if canWrite && delivery.status !== 'pending'}
														<Button variant="text" size="s" disabled={busy === delivery.id} onclick={() => redeliver(delivery)}>{m.integ_redeliver()}</Button>
													{/if}
												</li>
											{/each}
										</ul>
									{/if}
								</div>
							{/if}
						</li>
					{/each}
				</ul>
			{/if}
		{:else}
			<p class="muted">{m.common_loading()}</p>
		{/if}
	{:else if tab === 'orgs'}
		{#if orgs}
			{#if orgs.length === 0}
				<p class="muted">{m.integ_no_orgs()}</p>
			{:else}
				<ul class="list">
					{#each orgs as org (org.id)}
						<li class="row">
							<div class="row__main">
								<div class="row__text">
									<span class="row__title">{org.name}</span>
									<span class="row__meta">{org.synced_at ? m.integ_synced({ at: when(org.synced_at) }) : m.integ_not_synced()} · {m.integ_published_of({ published: org.published, total: org.projects })}</span>
								</div>
								<div class="actions">
									{#if canWrite}<Button variant="tonal" size="s" disabled={busy === org.id} onclick={() => syncOrg(org)}>{org.synced_at ? m.integ_sync() : m.integ_create_there()}</Button>{/if}
									{#if canManage && org.synced_at}<Button variant="text" size="s" disabled={busy === org.id} onclick={() => removeOrg(org)}>{m.integ_remove()}</Button>{/if}
								</div>
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		{:else}
			<p class="muted">{m.common_loading()}</p>
		{/if}
	{:else if tab === 'apps'}
		{#if overview}
			{#if orphans > 0}<p class="notice">{m.shipyard_orphans({ count: orphans })}</p>{/if}
			{#each overview.errors as e (e)}<p class="error">{e}</p>{/each}
			{#if overview.apps.length === 0}
				<p class="muted">{m.shipyard_no_apps()}</p>
			{:else}
				<ul class="list">
					{#each overview.apps as app (app.key)}
						<li class="row">
							<div class="row__main">
								<div class="row__text">
									<span class="row__title">{app.name} <code>{app.key}</code></span>
									<span class="row__meta">{app.org_name} · {app.environments.join(', ') || m.shipyard_no_envs()}</span>
								</div>
								<div class="actions">
									{#if app.project}
										<a class="link" href="/projects/session/{app.project.session_id}" target="_blank" rel="noopener">{app.project.name}</a>
									{:else if canWrite}
										<select aria-label={m.shipyard_link_to({ app: app.name })} bind:value={choice[app.key]}>
											<option value="">{m.shipyard_new_project()}</option>
											{#each candidatesFor(app.org_id) as p (p.id)}<option value={p.id}>{p.name}</option>{/each}
										</select>
										<Button variant="filled" size="s" disabled={busy === app.key} onclick={() => link(app)}>{m.shipyard_reconnect()}</Button>
									{/if}
									<Button variant="tonal" size="s" onclick={() => openApp(app)}>{m.integ_open_app()}</Button>
								</div>
							</div>
						</li>
					{/each}
				</ul>
			{/if}
		{:else}
			<p class="muted">{m.common_loading()}</p>
		{/if}
	{:else if tab === 'events'}
		{#if events}
			{#if events.events.length === 0}
				<p class="muted">{m.integ_no_events()}</p>
			{:else}
				<ul class="list">
					{#each events.events as event (event.id)}
						<li class="mini">
							<span class="mini__text">
								<span class="mini__title"><code>{event.type}</code></span>
								<span class="row__meta">{[event.project_name, event.environment, event.service].filter(Boolean).join(' · ') || '—'} · {when(event.created_at)}</span>
							</span>
						</li>
					{/each}
				</ul>
			{/if}
		{:else}
			<p class="muted">{m.common_loading()}</p>
		{/if}
	{/if}
</section>

<BottomSheet bind:open={sheetOpen} label={current?.name}>
	{#if current}
		<h2 class="sheet-title">{current.name} <code>{current.key}</code></h2>
		<p class="muted">{current.org_name}</p>
		{#if error && sheetOpen}<p class="error" role="alert">{error}</p>{/if}
		{#if notice && sheetOpen}<p class="notice" role="status">{notice}</p>{/if}
		{#if !detail}
			<p class="muted">{m.common_loading()}</p>
		{:else}
			<section class="block">
				<h3 class="block__title">{m.integ_environments()}</h3>
				{#if detail.environments.length === 0}
					<p class="muted small">{m.shipyard_no_envs()}</p>
				{:else}
					<ul class="list">
						{#each detail.environments as env (env.environment)}
							<li class="mini">
								<span class="mini__text">
									<span class="mini__title">{env.environment}{#if env.last_publish}<span class="state" data-state={env.last_publish.status}>{env.last_publish.status}</span>{/if}</span>
									<span class="row__meta">
										{env.services.map((s) => `${s.name} ${s.running}/${s.replicas}`).join(' · ') || m.integ_no_services()}
									</span>
									{#each env.services.flatMap((s) => s.urls) as url (url)}<a class="link small" href={url} target="_blank" rel="noopener">{url}</a>{/each}
								</span>
								{#if canManage}
									<Button variant="text" size="s" disabled={busy === `env-${env.environment}`} onclick={() => deleteEnv(env.environment)}>{m.integ_take_down()}</Button>
								{/if}
							</li>
						{/each}
					</ul>
				{/if}
				{#if canWrite && detail.environments.length > 0}
					<form
						class="form"
						onsubmit={(e) => {
							e.preventDefault();
							void duplicate();
						}}
					>
						<span class="form__title">{m.integ_duplicate()}</span>
						<div class="form__row">
							<label class="field">
								<span>{m.integ_from()}</span>
								<select bind:value={dupFrom}>
									{#each detail.environments as env (env.environment)}<option value={env.environment}>{env.environment}</option>{/each}
								</select>
							</label>
							<label class="field">
								<span>{m.integ_to()}</span>
								<input bind:value={dupTo} pattern="[a-z0-9][a-z0-9-]*" maxlength="40" required />
							</label>
						</div>
						<label class="check"><input type="checkbox" bind:checked={dupCopy} /> {m.integ_copy_secrets()}</label>
						<div><Button type="submit" variant="tonal" size="s" disabled={busy === 'duplicate' || !dupTo.trim()}>{m.integ_duplicate_go()}</Button></div>
					</form>
				{/if}
			</section>

			<section class="block">
				<h3 class="block__title">{m.integ_secrets()}</h3>
				<p class="muted small">{m.integ_secrets_hint()}</p>
				{#if secrets.length === 0}
					<p class="muted small">{m.integ_no_secrets()}</p>
				{:else}
					<ul class="list">
						{#each secrets as secret (secret.ref)}
							<li class="mini">
								<span class="mini__text">
									<span class="mini__title"><code>{secret.name}</code> <span class="tag">{secret.environment ?? m.integ_all_envs()}</span></span>
									<span class="row__meta">{secret.kind} · {secret.used_by.length ? secret.used_by.join(', ') : m.integ_unused()} · {when(secret.updated_at)}</span>
								</span>
								{#if canManage && secret.kind === 'user'}
									<Button variant="text" size="s" disabled={busy === secret.ref} onclick={() => deleteSecret(secret)}>{m.common_delete()}</Button>
								{/if}
							</li>
						{/each}
					</ul>
				{/if}
				{#if canManage}
					<form
						class="form"
						onsubmit={(e) => {
							e.preventDefault();
							void saveSecret();
						}}
					>
						<span class="form__title">{m.integ_set_secret()}</span>
						<div class="form__row">
							<label class="field">
								<span>{m.integ_secret_name()}</span>
								<input bind:value={secretName} placeholder="STRIPE_SECRET_KEY" autocomplete="off" spellcheck="false" required />
							</label>
							<label class="field">
								<span>{m.integ_environment()}</span>
								<select bind:value={secretEnv}>
									<option value="">{m.integ_all_envs()}</option>
									{#each detail.environments as env (env.environment)}<option value={env.environment}>{env.environment}</option>{/each}
								</select>
							</label>
						</div>
						{#if !secretGenerate}
							<label class="field">
								<span>{m.integ_secret_value()}</span>
								<input type="password" bind:value={secretValue} autocomplete="new-password" required />
							</label>
						{/if}
						<label class="check"><input type="checkbox" bind:checked={secretGenerate} /> {m.integ_generate()}</label>
						{#if secretName && !SECRET_NAME.test(secretName)}<p class="error small">{m.integ_secret_name_rule()}</p>{/if}
						<div>
							<Button type="submit" variant="tonal" size="s" disabled={busy === 'secret' || !SECRET_NAME.test(secretName) || (!secretGenerate && !secretValue)}>{m.integ_save_secret()}</Button>
						</div>
					</form>
				{/if}
			</section>

			{#if canManage}
				<section class="block">
					<h3 class="block__title">{m.integ_danger()}</h3>
					<p class="muted small">{m.integ_terminate_hint()}</p>
					<div><Button variant="outlined" size="s" disabled={busy === 'terminate'} onclick={terminateApp}>{m.integ_terminate()}</Button></div>
				</section>
			{/if}
		{/if}
	{/if}
</BottomSheet>

<Dialog bind:open={confirmOpen} headline={m.integ_are_you_sure()}>
	<p>{confirmText}</p>
	{#snippet actions()}
		<Button variant="text" onclick={() => (confirmOpen = false)}>{m.common_cancel()}</Button>
		<Button
			variant="filled"
			onclick={async () => {
				confirmOpen = false;
				await confirmAction?.();
			}}>{m.integ_confirm()}</Button
		>
	{/snippet}
</Dialog>

<BottomSheet bind:open={addOpen} label={m.integ_add_endpoint()}>
	<form
		class="add"
		onsubmit={(e) => {
			e.preventDefault();
			void addEndpoint();
		}}
	>
		<h2 class="add__title">{m.integ_add_endpoint()}</h2>
		<TextField name="webhook_url" label={m.integ_add_url()} type="url" inputmode="url" autocomplete="off" bind:value={addUrl} supportingText={m.integ_add_url_hint()} />
		{#if status?.webhook_target && addUrl.trim() === status.webhook_target}
			<p class="muted small">{m.integ_add_own()}</p>
		{/if}
		<div class="add__group">
			<span class="add__label">{m.integ_add_events()}</span>
			<ButtonGroup
				options={[{ value: 'all', label: m.integ_all_events() }, { value: 'some', label: m.integ_add_choose() }]}
				bind:value={addAll}
				aria-label={m.integ_add_events()}
			/>
			{#if addAll === 'some'}
				{#if eventChoices.length === 0}
					<p class="muted small">{busy === 'events' ? m.integ_loading() : m.integ_add_no_types()}</p>
				{:else}
					<div class="add__chips">
						{#each eventChoices as event (event)}
							<Chip variant="filter" selected={addEvents.includes(event)} onclick={() => toggleEvent(event)}><code>{event}</code></Chip>
						{/each}
					</div>
				{/if}
			{/if}
		</div>
		<Select name="webhook_org" label={m.integ_add_org()} options={orgChoices} bind:value={addOrg} />
		{#if addError}<p class="error" role="alert">{addError}</p>{/if}
		<div class="add__actions">
			<Button variant="text" type="button" onclick={() => (addOpen = false)}>{m.common_cancel()}</Button>
			<Button variant="filled" type="submit" disabled={busy === 'add' || !addUrl.trim() || (addAll === 'some' && addEvents.length === 0)}>{m.integ_add_submit()}</Button>
		</div>
	</form>
</BottomSheet>

<style>
	.integration {
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.setup {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 10px 16px;
		padding: 12px 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-high);
	}
	.break {
		overflow-wrap: anywhere;
	}
	.setup__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: min(100%, 240px);
		flex: 1;
	}
	.integration__head {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 4px 12px;
	}
	.integration__name {
		font-family: var(--md-sys-typescale-title-medium-font);
		font-size: 1.125rem;
		font-weight: 800;
		color: var(--md-sys-color-on-surface);
	}
	.integration__what {
		flex: 1;
		min-width: 180px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.chip {
		padding: 2px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.chip[data-on='true'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.tabs {
		display: flex;
		gap: 4px;
		overflow-x: auto;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		scrollbar-width: none;
	}
	.tab {
		flex: none;
		min-height: 44px;
		padding: 0 14px;
		border: none;
		border-bottom: 3px solid transparent;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.tab[aria-selected='true'] {
		border-bottom-color: var(--md-sys-color-primary);
		color: var(--md-sys-color-primary);
	}
	.tab:hover:not(:disabled) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
	}
	.tab:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.tab:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}
	.facts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 12px;
		margin: 0;
	}
	.facts dt {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.facts dd {
		margin: 2px 0 0;
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
		white-space: pre-line;
	}
	.notice {
		margin: 0;
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.small {
		font-size: 0.8125rem;
	}
	.add {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 0 4px 12px;
	}
	.add__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 800;
		color: var(--md-sys-color-on-surface);
	}
	.add__group {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.add__label {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		font-weight: 600;
	}
	.add__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		max-height: 220px;
		overflow-y: auto;
	}
	.add__chips code {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
	}
	.add__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}
	.secret-once {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.secret-once code {
		font-family: var(--md-ref-typeface-mono);
		overflow-wrap: anywhere;
	}
	.list {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 12px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.row:last-child {
		border-bottom: none;
	}
	.row__main {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 16px;
	}
	.row__text,
	.mini__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: min(100%, 220px);
	}
	.row__title,
	.mini__title {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.row__title code,
	.mini__title code,
	.sheet-title code {
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		font-weight: 400;
	}
	.row__meta {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		overflow-wrap: anywhere;
	}
	.tag {
		padding: 1px 8px;
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.6875rem;
		font-weight: 600;
	}
	.state {
		padding: 1px 8px;
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-highest);
		font-size: 0.6875rem;
		font-weight: 600;
	}
	.state[data-state='delivered'],
	.state[data-state='succeeded'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.state[data-state='failed'] {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.mini {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px 12px;
		padding: 8px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.mini:last-child {
		border-bottom: none;
	}
	.sub {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 10px 12px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.filter {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.link {
		color: var(--md-sys-color-primary);
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	select,
	input:not([type='checkbox']) {
		min-height: 44px;
		padding: 0 10px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		min-width: 0;
	}
	select:focus-visible,
	input:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.sheet-title {
		margin: 0;
		font-family: var(--md-sys-typescale-headline-small-font);
		font-size: 1.375rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.block {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-top: 20px;
	}
	.block__title {
		margin: 0;
		font-size: 1rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.form {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-top: 6px;
		padding: 12px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.form__title {
		font-weight: 700;
		font-size: 0.875rem;
	}
	.form__row {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
		gap: 10px;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.check {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 44px;
		font-size: 0.875rem;
	}
	.check input {
		width: 18px;
		height: 18px;
		accent-color: var(--md-sys-color-primary);
	}
</style>
