<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { goto, invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import { DEFAULT_SERVICES, WORKSPACE_SERVICE_ORDER, WORKSPACE_SERVICES, isWorkspaceService } from '$lib/workspace';
	import { timeAgo } from '$lib/i18n';
	import type { WorkspaceService } from '$lib/types';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const google = $derived(data.google);
	const connection = $derived(google?.connection ?? null);

	let sheetOpen = $state(false);
	let confirmOpen = $state(false);
	let starting = $state(false);
	let picked = $state<Record<WorkspaceService, boolean>>({ gmail: true, sheets: true, docs: true, drive: true, calendar: false });
	let resume = $state<string | null>(null);
	let snackbar = $state(false);
	let snackbarMessage = $state('');

	function openSheet(extra: WorkspaceService[] = []) {
		const base = connection ? connection.services : DEFAULT_SERVICES;
		const on = new Set<WorkspaceService>([...base, ...extra]);
		picked = Object.fromEntries(WORKSPACE_SERVICE_ORDER.map((s) => [s, on.has(s)])) as Record<WorkspaceService, boolean>;
		sheetOpen = true;
	}

	// Arriving from a chat's Connect card (?connect=1&services=…&resume=…), back from Google
	// (?connected=1), or from a declined or failed sign-in (?error=…).
	$effect(() => {
		const params = page.url.searchParams;
		if (params.get('connect') === '1' && google?.configured) {
			resume = params.get('resume');
			openSheet((params.get('services') ?? '').split(',').filter(isWorkspaceService));
		}
		if (params.get('connected') === '1') {
			snackbarMessage = m.conn_connected_snack();
			snackbar = true;
		}
		if (params.size > 0) goto('/connections', { replaceState: true, noScroll: true, keepFocus: true });
	});

	// Just signed up with Google: offer Workspace with that same account (Google pre-selects it).
	const welcome = page.url.searchParams.get('welcome') === '1';
	const errorParam = page.url.searchParams.get('error');
	const signInError = errorParam === 'declined' ? m.conn_signin_cancelled() : errorParam;

	const missing = $derived(connection ? WORKSPACE_SERVICE_ORDER.filter((s) => !connection.services.includes(s)) : []);

</script>

{#snippet serviceIcon(service: WorkspaceService, size = 20)}
	<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
		<path d={WORKSPACE_SERVICES[service].icon} />
	</svg>
{/snippet}

<div class="connections">
	<div class="connections__inner">
		<header class="connections__head">
			<h1 class="md-display-small connections__title">{m.conn_title()}</h1>
			<p class="md-body-large connections__lede">{m.conn_lede()}</p>
		</header>

		{#if signInError}
			<p class="notice" role="alert">{signInError}</p>
		{/if}
		{#if form?.error}
			<p class="notice" role="alert">{form.error}</p>
		{/if}

		{#if !google}
			<p class="notice" role="alert">{m.conn_load_failed()}</p>
		{:else if welcome && !connection}
			<section class="welcome" aria-labelledby="welcome-title">
				<AgentShape agent="nomi" size={56} />
				<div class="welcome__text">
					<h2 id="welcome-title" class="welcome__title">{m.conn_welcome_title()}</h2>
					<p class="welcome__body">
						{m.conn_welcome_body()}
					</p>
				</div>
				<div class="welcome__actions">
					{#if google.configured}
						<Button variant="gradient" size="m" onclick={() => openSheet()}>{m.conn_connect_workspace()}</Button>
					{/if}
					<Button variant="text" size="m" href="/">{m.conn_maybe_later()}</Button>
				</div>
			</section>
		{/if}

		{#if !google}
			<!-- shown above -->
		{:else if connection}
			<section class="card card--connected" aria-labelledby="google-title">
				<div class="card__top">
					<AgentShape agent="workspace" size={52} />
					<div class="card__titles">
						<h2 id="google-title" class="card__title">Google Workspace</h2>
						<span class="card__sub">{connection.email}</span>
					</div>
				</div>
				<span class="live"><span class="live__dot"></span>{m.conn_connected_only_you()}</span>
				<ul class="chips" aria-label={m.conn_allowed_services()}>
					{#each connection.services as service (service)}
						<li class="chip">{WORKSPACE_SERVICES[service].label}</li>
					{/each}
					{#each missing as service (service)}
						<li><button type="button" class="chip chip--add" onclick={() => openSheet([service])}>+ {WORKSPACE_SERVICES[service].label}</button></li>
					{/each}
				</ul>
			</section>

			<h2 class="section-label">{m.conn_recent()}</h2>
			{#if google.activity.length === 0}
				<p class="empty">{m.conn_recent_empty()}</p>
			{:else}
				<ul class="rows">
					{#each google.activity as item, i (i)}
						<li class="row">
							<span class="row__icon">{@render serviceIcon(item.service, 18)}</span>
							<span class="row__text">
								<span class="row__headline">{item.summary}</span>
								<span class="row__supporting">{WORKSPACE_SERVICES[item.service]?.label ?? item.service} · {timeAgo(item.created_at)}</span>
							</span>
							{#if item.link}
								<a class="row__link" href={item.link} target="_blank" rel="noreferrer">{m.common_open()}</a>
							{/if}
						</li>
					{/each}
				</ul>
			{/if}

			<h2 class="section-label">{m.conn_always_ask()}</h2>
			<ul class="rows">
				<li class="row row--plain"><span class="row__headline">{m.conn_sending_email()}</span><span class="always">{m.conn_always()}</span></li>
				<li class="row row--plain"><span class="row__headline">{m.conn_inviting()}</span><span class="always">{m.conn_always()}</span></li>
			</ul>

			<div class="connections__actions">
				<Button variant="tonal" size="m" onclick={() => openSheet()}>{m.conn_change_access()}</Button>
				<Button variant="outlined" size="m" class="danger" onclick={() => (confirmOpen = true)}>{m.conn_disconnect()}</Button>
			</div>
		{:else}
			<section class="card" aria-labelledby="google-title">
				<div class="card__top">
					<AgentShape agent="workspace" size={52} />
					<div class="card__titles">
						<h2 id="google-title" class="card__title">Google Workspace</h2>
						<span class="card__sub">{m.conn_for_workspace()}</span>
						<span class="status">{m.conn_not_connected()}</span>
					</div>
				</div>
				<ul class="tiles" aria-label={m.conn_what_it_covers()}>
					{#each WORKSPACE_SERVICE_ORDER as service (service)}
						<li class="tile">{@render serviceIcon(service)}<span>{WORKSPACE_SERVICES[service].label}</span></li>
					{/each}
				</ul>
				{#if google.configured}
					<Button variant="filled" size="m" class="card__cta" onclick={() => openSheet()}>{m.conn_connect_google()}</Button>
				{:else}
					<p class="unconfigured">
						{m.conn_unconfigured()} <code>GOOGLE_CLIENT_ID</code>,
						<code>GOOGLE_CLIENT_SECRET</code> {m.conn_and()} <code>GOOGLE_REDIRECT_URI</code>.
					</p>
				{/if}
			</section>

			<h2 class="section-label">{m.conn_how()}</h2>
			<ol class="steps">
				<li><span class="steps__n">1</span>{m.conn_step1()}</li>
				<li><span class="steps__n">2</span>{m.conn_step2()}</li>
				<li><span class="steps__n">3</span>{m.conn_step3()}</li>
			</ol>
		{/if}
	</div>
</div>

<BottomSheet bind:open={sheetOpen}>
	{#snippet children()}
		<form
			method="POST"
			action="?/start"
			class="sheet"
			use:enhance={() => {
				starting = true;
				return async ({ result, update }) => {
					if (result.type === 'success' && typeof result.data?.url === 'string') {
						window.location.assign(result.data.url);
						return;
					}
					starting = false;
					await update();
				};
			}}
		>
			<h2 class="sheet__title">{m.conn_sheet_title()}</h2>
			<p class="sheet__lede">{m.conn_sheet_lede()}</p>
			<ul class="rows">
				{#each WORKSPACE_SERVICE_ORDER as service (service)}
					<li class="row">
						<label class="row__text" for="svc-{service}">
							<span class="row__headline">{WORKSPACE_SERVICES[service].label}</span>
							<span class="row__supporting">{WORKSPACE_SERVICES[service].detail}</span>
						</label>
						<Switch id="svc-{service}" bind:checked={picked[service]} aria-label={m.conn_allow({ service: WORKSPACE_SERVICES[service].label })} />
						{#if picked[service]}<input type="hidden" name="service" value={service} />{/if}
					</li>
				{/each}
			</ul>
			{#if resume}<input type="hidden" name="resume" value={resume} />{/if}
			<p class="sheet__guard">
				<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 3l8 4v5c0 5-3.5 8-8 9-4.5-1-8-4-8-9V7z" /></svg>
				{m.conn_guard()}
			</p>
			<div class="sheet__actions">
				<Button variant="filled" size="m" type="submit" disabled={starting || !Object.values(picked).some(Boolean)}>
					{starting ? m.conn_opening_google() : m.conn_continue_google()}
				</Button>
				<Button variant="text" size="m" onclick={() => (sheetOpen = false)}>{m.common_not_now()}</Button>
			</div>
		</form>
	{/snippet}
</BottomSheet>

<Dialog bind:open={confirmOpen} headline={m.conn_disconnect_title()}>
	{#snippet children()}
		<p class="md-body-medium">{m.conn_disconnect_body({ account: connection?.email ?? m.conn_your_account() })}</p>
	{/snippet}
	{#snippet actions()}
		<Button variant="text" size="s" onclick={() => (confirmOpen = false)}>{m.common_cancel()}</Button>
		<form
			method="POST"
			action="?/disconnect"
			use:enhance={() => async ({ update }) => {
				confirmOpen = false;
				await update();
				await invalidateAll();
				snackbarMessage = m.conn_disconnected_snack();
				snackbar = true;
			}}
		>
			<Button variant="filled" size="s" type="submit">{m.conn_disconnect()}</Button>
		</form>
	{/snippet}
</Dialog>

<Snackbar bind:open={snackbar} message={snackbarMessage} />

<style>
	.connections {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.connections__inner {
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-width: 720px;
		margin: 0 auto;
	}
	.connections__head {
		margin-bottom: 8px;
	}
	.connections__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.connections__lede {
		margin: 8px 0 0;
		max-width: 60ch;
		color: var(--md-sys-color-on-surface-variant);
	}
	.notice {
		margin: 0;
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}

	.welcome {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 16px 20px;
		padding: 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.welcome__text {
		display: flex;
		flex: 1 1 280px;
		flex-direction: column;
		gap: 6px;
	}
	.welcome__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.5rem;
		font-weight: 700;
	}
	.welcome__body {
		margin: 0;
		font-size: 0.9375rem;
		line-height: 1.5;
	}
	.welcome__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.card--connected {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.card__top {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.card__titles {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.card__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.375rem;
		font-weight: 700;
	}
	.card__sub {
		overflow: hidden;
		font-size: 0.875rem;
		text-overflow: ellipsis;
		opacity: 0.85;
	}
	.card :global(.card__cta) {
		width: 100%;
	}
	.status,
	.live,
	.section-label,
	.always {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.status {
		align-self: flex-start;
		margin-top: 6px;
		padding: 4px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface-variant);
	}
	.live {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		color: var(--md-sys-color-primary);
	}
	.live__dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: currentColor;
	}
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		min-height: 32px;
		padding: 0 12px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: color-mix(in srgb, var(--md-sys-color-surface-container-lowest) 60%, transparent);
		color: inherit;
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
	}
	.chip--add {
		border: 1px dashed currentColor;
		background: transparent;
		cursor: pointer;
	}
	.tiles {
		display: grid;
		grid-template-columns: repeat(5, minmax(0, 1fr));
		gap: 8px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.tile {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 10px 0;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-primary);
		font-size: 0.6875rem;
		font-weight: 600;
	}
	.tile span {
		color: var(--md-sys-color-on-surface);
	}
	.unconfigured {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.5;
	}
	.unconfigured code {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
	}

	.section-label {
		margin: 12px 8px 0;
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 400;
	}
	.empty {
		margin: 0 8px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.rows {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin: 0;
		padding: 0;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
		list-style: none;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 14px;
		min-height: 56px;
		padding: 10px 16px;
		background: var(--md-sys-color-surface-container-lowest);
	}
	.row--plain {
		justify-content: space-between;
	}
	.row__icon {
		display: flex;
		flex-shrink: 0;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-primary);
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__headline {
		overflow-wrap: anywhere;
		color: var(--md-sys-color-on-surface);
		font-size: 1rem;
		font-weight: 600;
	}
	.row__supporting {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.4;
	}
	.row__link {
		color: var(--md-sys-color-primary);
		font-size: 0.875rem;
		font-weight: 600;
		text-decoration: none;
	}
	.always {
		color: var(--md-sys-color-primary);
	}
	.connections__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 8px;
	}
	.connections__actions :global(.danger) {
		color: var(--md-sys-color-error);
	}
	.steps {
		display: flex;
		flex-direction: column;
		gap: 14px;
		margin: 0;
		padding: 0 8px;
		list-style: none;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		line-height: 1.45;
	}
	.steps li {
		display: flex;
		align-items: flex-start;
		gap: 14px;
	}
	.steps__n {
		display: flex;
		flex-shrink: 0;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-size: 0.8125rem;
		font-weight: 700;
	}

	.sheet {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.sheet__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.625rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.sheet__lede {
		margin: 0 0 4px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.sheet__guard {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		margin: 4px 0 0;
		padding: 14px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		font-size: 0.8125rem;
		line-height: 1.45;
	}
	.sheet__guard svg {
		flex-shrink: 0;
	}
	.sheet__actions {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin-top: 8px;
	}
</style>
