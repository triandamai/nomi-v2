<script lang="ts">
	// Admin → Shipyard: whether publishing is connected (and its webhook), and every app Shipyard
	// holds for Nomi's organizations. An app no project points at can be reconnected to one, or
	// given a new project.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { adminApi } from '$lib/adminApi';

	interface Status { configured: boolean; can_reveal: boolean; webhook_url: string | null; webhook_from_env: boolean; registered_at: string | null; last_event_at: string | null }
	interface App { org_id: string; org_name: string; key: string; name: string; environments: string[]; project: { id: string; name: string; session_id: string } | null }
	interface Overview { apps: App[]; candidates: { org_id: string; projects: { id: string; name: string }[] }[]; errors: string[] }

	let status = $state<Status | null>(null);
	let overview = $state<Overview | null>(null);
	let error = $state<string | null>(null);
	let busy = $state<string | null>(null);
	let choice = $state<Record<string, string>>({});

	const when = (at: string | null) => (at ? new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(at)) : '—');

	async function load() {
		error = null;
		try {
			status = await adminApi<Status>('shipyard');
			if (status.configured) overview = await adminApi<Overview>('shipyard/apps');
		} catch (e) {
			error = (e as Error).message;
		}
	}
	$effect(() => {
		void load();
	});

	async function registerWebhook() {
		busy = 'webhook';
		try {
			status = await adminApi<Status>('shipyard/webhook', { method: 'POST' });
		} catch (e) {
			error = (e as Error).message;
		} finally {
			busy = null;
		}
	}

	async function link(app: App) {
		busy = app.key;
		error = null;
		const picked = choice[app.key] ?? '';
		try {
			await adminApi('shipyard/apps/link', { method: 'POST', body: { org_id: app.org_id, app: app.key, project_id: picked || null } });
			await load();
		} catch (e) {
			error = (e as Error).message;
		} finally {
			busy = null;
		}
	}

	const candidatesFor = (orgId: string) => overview?.candidates.find((c) => c.org_id === orgId)?.projects ?? [];
	const orphans = $derived(overview?.apps.filter((a) => !a.project).length ?? 0);
</script>

<PageHeader title={m.admin_shipyard()} lede={m.shipyard_lede()} agent="coding" />

{#if error}<p class="error" role="alert">{error}</p>{/if}

{#if status}
	<section class="panel">
		<dl class="facts">
			<div><dt>{m.shipyard_connection()}</dt><dd>{status.configured ? m.shipyard_connected() : m.shipyard_not_configured()}</dd></div>
			<div><dt>{m.shipyard_webhook()}</dt><dd>{status.webhook_from_env ? m.shipyard_webhook_env() : (status.webhook_url ?? m.shipyard_webhook_none())}</dd></div>
			<div><dt>{m.shipyard_last_event()}</dt><dd>{when(status.last_event_at)}</dd></div>
			<div><dt>{m.shipyard_db_access()}</dt><dd>{status.can_reveal ? m.shipyard_db_on() : m.shipyard_db_off()}</dd></div>
		</dl>
		{#if status.configured && !status.webhook_from_env}
			<div><Button variant="tonal" size="s" disabled={busy === 'webhook'} onclick={registerWebhook}>{status.webhook_url ? m.shipyard_reregister() : m.shipyard_register()}</Button></div>
		{/if}
	</section>
{/if}

{#if overview}
	<h2 class="title">{m.shipyard_apps()}{#if orphans > 0}<span class="count">{m.shipyard_orphans({ count: orphans })}</span>{/if}</h2>
	{#each overview.errors as e (e)}<p class="error">{e}</p>{/each}
	{#if overview.apps.length === 0}
		<p class="muted">{m.shipyard_no_apps()}</p>
	{:else}
		<ul class="list">
			{#each overview.apps as app (app.key)}
				<li class="row" class:row--orphan={!app.project}>
					<div class="row__text">
						<span class="row__title">{app.name} <code>{app.key}</code></span>
						<span class="row__meta">{app.org_name} · {app.environments.join(', ') || m.shipyard_no_envs()}</span>
					</div>
					{#if app.project}
						<a class="row__project" href="/projects/session/{app.project.session_id}" target="_blank" rel="noopener">{app.project.name}</a>
					{:else}
						<div class="row__link">
							<select aria-label={m.shipyard_link_to({ app: app.name })} bind:value={choice[app.key]}>
								<option value="">{m.shipyard_new_project()}</option>
								{#each candidatesFor(app.org_id) as p (p.id)}<option value={p.id}>{p.name}</option>{/each}
							</select>
							<Button variant="filled" size="s" disabled={busy === app.key} onclick={() => link(app)}>{m.shipyard_reconnect()}</Button>
						</div>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
{/if}

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin-bottom: 20px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
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
	.title {
		display: flex;
		align-items: center;
		gap: 10px;
		margin: 0 0 8px;
		font-size: 1rem;
		font-weight: 700;
	}
	.count {
		padding: 1px 8px;
		border-radius: 4px;
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		font-size: 0.75rem;
	}
	.error {
		margin: 0 0 12px;
		color: var(--md-sys-color-error);
	}
	.muted {
		color: var(--md-sys-color-on-surface-variant);
	}
	.list {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 16px;
		padding: 12px 4px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__title {
		font-weight: 600;
	}
	.row__title code {
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
	}
	.row__meta {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.row__project {
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.row__link {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.row__link select {
		min-height: 40px;
		padding: 0 10px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font: inherit;
	}
	.row__link select:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
</style>
