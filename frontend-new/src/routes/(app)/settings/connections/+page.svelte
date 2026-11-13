<script lang="ts">
	import { onMount, type Component } from 'svelte';
	import { Dumbbell, Mail, type LucideProps } from '@lucide/svelte';
	import { goto, invalidate } from '$app/navigation';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { pageAction } from '$lib/pageAction';
	import { ago } from '$lib/adapters/time';
	import { GOOGLE_SERVICES, appsToAdd, connectedApps, serviceName, type AppId } from '$lib/settings/connections';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import GithubMark from '$lib/ds/foundations/GithubMark.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import ConnectionCard from '$lib/ds/settings/ConnectionCard.svelte';
	import AppCard from '$lib/ds/settings/AppCard.svelte';

	// Settings › Connections (App / Settings / Connections OBmgS): the apps the crew works in, as the
	// backend connects them. Connected cards (what each holds and may do, who uses it, Manage to
	// disconnect or, for Google, change what Tara may use) and the apps still to add. Google and
	// GitHub go off to sign in and come back through /connections/*/callback; a GitHub token and a
	// Hevy key are checked as they connect. Left out (AGENTS.md §9, BACKEND_TODO.md): the app
	// catalog with its categories and search, MCP servers, per-service access levels and "Needs you".
	let { data } = $props();

	const PATH = '/settings/connections';
	const ICONS: Record<AppId, Component<LucideProps>> = { google: Mail, github: GithubMark, hevy: Dumbbell };
	const NAMES: Record<AppId, () => string> = { google: m.connections_name_google, github: m.connections_name_github, hevy: m.connections_name_hevy };

	const connected = $derived(connectedApps(data.overview, data.github));
	const toAdd = $derived(appsToAdd(data.overview, data.github));

	// Which dialog is open: adding an app, or managing a connected one.
	let open = $state<{ kind: 'add' | 'manage'; app: AppId } | null>(null);
	const dialogOpen = (kind: 'add' | 'manage', app: AppId) => open?.kind === kind && open.app === app;
	const close = () => (open = null);

	let services = $state<string[]>([...GOOGLE_SERVICES]);
	let token = $state('');
	let key = $state('');
	let busy = $state(false);
	let failed = $state<string>();

	function add(app: AppId) {
		failed = undefined;
		token = '';
		key = '';
		services = app === 'google' && data.overview?.google.services.length ? [...data.overview.google.services] : [...GOOGLE_SERVICES];
		open = { kind: 'add', app };
	}

	async function connectWith(app: 'github' | 'hevy') {
		busy = true;
		failed = undefined;
		const result = await pageAction(PATH, app === 'github' ? 'githubToken' : 'hevy', app === 'github' ? { token } : { api_key: key });
		busy = false;
		if (!result.ok) return void (failed = result.message || undefined);
		close();
		toasts.show({ kind: 'success', title: m.connections_now_connected({ name: NAMES[app]() }) });
		await invalidate('app:connections');
	}

	async function disconnect(app: AppId) {
		busy = true;
		const result = await pageAction(PATH, 'disconnect', { app });
		busy = false;
		if (!result.ok) return void (result.message && toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message }));
		close();
		toasts.show({ kind: 'success', title: m.connections_disconnected({ name: NAMES[app]() }) });
		await invalidate('app:connections');
	}

	// Back from Google or GitHub: say how it went, then take the word off the address.
	onMount(() => {
		const params = page.url.searchParams;
		const came = params.get('connected') as AppId | null;
		const error = params.get('error');
		if (!came && !error) return;
		if (came && NAMES[came]) toasts.show({ kind: 'success', title: m.connections_now_connected({ name: NAMES[came]() }) });
		if (error) toasts.show({ kind: 'error', title: error === 'declined' ? m.connections_declined() : m.connections_failed() });
		void goto(PATH, { replaceState: true, noScroll: true, keepFocus: true });
	});
</script>

<svelte:head><title>{m.settings_connections()} · {m.app_name()}</title></svelte:head>

<div class="h-full overflow-y-auto">
	<div class="flex max-w-316 flex-col gap-8 px-4 pt-7 pb-10 lg:px-10">
		<header class="flex max-w-215 flex-col gap-1.5">
			<h1 class="text-display font-medium tracking-display text-text">{m.settings_connections()}</h1>
			<p class="text-body text-text-2">{m.connections_intro()}</p>
		</header>

		<section aria-labelledby="connected-title" class="flex flex-col gap-3">
			<h2 id="connected-title" class="flex items-end gap-2">
				<span class="text-body font-medium text-text">{m.connections_connected()}</span>
				<span class="font-mono text-label text-text-3">{connected.length}</span>
			</h2>
			{#if connected.length}
				<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
					{#each connected as app (app.id)}
						<ConnectionCard
							icon={ICONS[app.id]}
							name={NAMES[app.id]()}
							account={app.account}
							status={m.connections_on()}
							caption={app.caption}
							canLabel={m.connections_can_do()}
							can={app.can}
							usedBy={app.usedBy}
							when={app.lastUsed ? ago(app.lastUsed, new Date()) : undefined}
							manage={m.connections_manage()}
							onmanage={() => (open = { kind: 'manage', app: app.id })}
						/>
					{/each}
				</div>
			{:else}
				<p class="text-caption text-text-3">{m.connections_none()}</p>
			{/if}
		</section>

		<section aria-labelledby="add-title" class="flex flex-col gap-3">
			<h2 id="add-title" class="text-body font-medium text-text">{m.connections_add_more()}</h2>
			{#if toAdd.length}
				<div class="grid gap-x-4 gap-y-2.5 md:grid-cols-2 xl:grid-cols-3">
					{#each toAdd as app (app.id)}
						<AppCard
							icon={ICONS[app.id]}
							name={NAMES[app.id]()}
							caption={app.caption}
							agent={app.for.agent}
							forLabel={m.connections_for({ name: app.for.name })}
							access={app.access}
							add={m.connections_add({ name: NAMES[app.id]() })}
							unavailable={app.available ? undefined : m.connections_not_set_up()}
							onadd={() => add(app.id)}
						/>
					{/each}
				</div>
			{:else}
				<p class="text-caption text-text-3">{m.connections_all_added()}</p>
			{/if}
		</section>
	</div>
</div>

<!-- Google: off to sign in with the services chosen (a plain post: the answer sends the browser to Google). -->
<form method="POST" action="?/google">
	<input type="hidden" name="resume" value={data.resume ?? ''} />
	{#each services as service (service)}<input type="hidden" name="service" value={service} />{/each}
	<Dialog bind:open={() => dialogOpen('add', 'google'), (value) => !value && close()} title={m.connections_google_title()} description={m.connections_google_body()}>
		<fieldset class="flex flex-col gap-2.5 pt-2">
			<legend class="mb-2 font-mono text-label tracking-label text-text-3 uppercase">{m.connections_google_services()}</legend>
			{#each GOOGLE_SERVICES as service (service)}
				<Checkbox
					label={serviceName(service)}
					checked={services.includes(service)}
					onchange={(checked) => (services = checked ? [...services, service] : services.filter((one) => one !== service))}
				/>
			{/each}
		</fieldset>
		{#snippet actions()}
			<Button variant="secondary" label={m.action_cancel()} onclick={close} />
			<Button type="submit" variant="inverse" label={m.connections_google_go()} disabled={!services.length} />
		{/snippet}
	</Dialog>
</form>

<form method="POST" action="?/github">
	<input type="hidden" name="resume" value={data.resume ?? ''} />
	<Dialog bind:open={() => dialogOpen('add', 'github'), (value) => !value && close()} title={m.connections_github_title()} description={m.connections_github_body()}>
		<div class="flex flex-col gap-3 pt-2">
			{#if data.github?.oauth}
				<Button type="submit" variant="inverse" iconStart={GithubMark} label={m.connections_github_go()} />
				<span class="text-center font-mono text-label text-text-3">{m.connections_or()}</span>
			{/if}
			<TextField label={m.connections_github_token_label()} type="password" mono autocomplete="off" bind:value={token} error={failed} />
		</div>
		{#snippet actions()}
			<Button variant="secondary" label={m.action_cancel()} onclick={close} />
			<Button variant="secondary" label={m.connections_github_token_go()} loading={busy} disabled={!token.trim()} onclick={() => connectWith('github')} />
		{/snippet}
	</Dialog>
</form>

<Dialog bind:open={() => dialogOpen('add', 'hevy'), (value) => !value && close()} title={m.connections_hevy_title()} description={m.connections_hevy_body()}>
	<div class="pt-2">
		<TextField label={m.connections_hevy_key_label()} type="password" mono autocomplete="off" bind:value={key} error={failed} />
	</div>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={close} />
		<Button variant="inverse" label={m.connections_check_connect()} loading={busy} disabled={!key.trim()} onclick={() => connectWith('hevy')} />
	{/snippet}
</Dialog>

{#each connected as app (app.id)}
	<Dialog bind:open={() => dialogOpen('manage', app.id), (value) => !value && close()} title={NAMES[app.id]()} description={app.caption}>
		<dl class="flex flex-col gap-3 pt-2">
			<div class="flex flex-col gap-1">
				<dt class="font-mono text-label tracking-label text-text-3 uppercase">{m.connections_manage_account()}</dt>
				<dd class="font-mono text-ui text-text">{app.account}</dd>
			</div>
			<div class="flex flex-col gap-1.5">
				<dt class="font-mono text-label tracking-label text-text-3 uppercase">{m.connections_can_do()}</dt>
				<dd class="flex flex-wrap gap-1">
					{#each app.can as thing (thing)}<span class="inline-flex h-5.5 items-center rounded-xs bg-surface-2 px-1.75 font-mono text-label text-text-2">{thing}</span>{/each}
				</dd>
			</div>
		</dl>
		{#snippet actions()}
			<Button variant="danger" label={m.general_disconnect()} loading={busy} onclick={() => disconnect(app.id)} />
			{#if app.id === 'google'}<Button variant="secondary" label={m.connections_change_access()} onclick={() => add('google')} />{/if}
			<Button variant="secondary" label={m.action_close()} onclick={close} />
		{/snippet}
	</Dialog>
{/each}
