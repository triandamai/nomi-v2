<script lang="ts">
	// Admin → Integrations → Relay: how Nomi reaches Relay (kept encrypted in Nomi), and Relay's
	// own settings (kept by Relay, checked before it saves them). Database passwords are never
	// shown back: their fields start empty and stay as stored unless something is typed.
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import { adminApi } from '$lib/adminApi';
	import { DEFAULT_SMTP_PORT, readRefusal, settingsBody, type RelayCheck, type RelayOverview, type RelaySettingsForm } from '$lib/relaySettings';

	let { canManage }: { canManage: boolean } = $props();

	let overview = $state<RelayOverview | null>(null);
	let error = $state<string | null>(null);
	let notice = $state<string | null>(null);
	let checks = $state<RelayCheck[]>([]);
	let busy = $state<'connection' | 'settings' | null>(null);

	let controlUrl = $state('');
	let controlKey = $state('');
	let form = $state<RelaySettingsForm>({
		public_domain: '',
		shared_dev_url: '',
		shared_production_url: '',
		postgrest_dev_url: '',
		postgrest_production_url: '',
		postgrest_role: 'authenticator',
		preview_origins: '',
		smtp_host: '',
		smtp_port: String(DEFAULT_SMTP_PORT),
		smtp_security: 'starttls',
		smtp_user: '',
		smtp_password: '',
		smtp_from: '',
	});
	const securityOptions = [
		{ value: 'starttls', label: 'STARTTLS' },
		{ value: 'tls', label: 'TLS' },
		{ value: 'none', label: m.relay_admin_smtp_none() },
	];

	const connected = $derived(!!overview && overview.connection.source !== 'none');
	const status = $derived.by(() => {
		if (!overview) return null;
		if (overview.relay_error) return { label: m.relay_admin_unreachable(), on: false };
		if (overview.relay?.configured) return { label: m.relay_admin_ready(), on: true };
		return { label: m.relay_admin_not_set_up(), on: false };
	});

	function fill(next: RelayOverview) {
		overview = next;
		controlUrl = next.connection.control_url;
		controlKey = '';
		const relay = next.relay;
		form = {
			public_domain: relay?.public_domain ?? '',
			shared_dev_url: '',
			shared_production_url: '',
			postgrest_dev_url: relay?.postgrest_dev_url ?? '',
			postgrest_production_url: relay?.postgrest_production_url ?? '',
			postgrest_role: relay?.postgrest_role || 'authenticator',
			preview_origins: (relay?.preview_origins.length ? relay.preview_origins : next.default_preview_origins).join('\n'),
			smtp_host: relay?.shared_smtp?.host ?? '',
			smtp_port: String(relay?.shared_smtp?.port ?? DEFAULT_SMTP_PORT),
			smtp_security: relay?.shared_smtp?.security || 'starttls',
			smtp_user: relay?.shared_smtp?.user ?? '',
			smtp_password: '',
			smtp_from: relay?.shared_smtp?.from ?? '',
		};
		checks = relay?.checks ?? [];
	}

	async function load() {
		try {
			fill(await adminApi<RelayOverview>('relay'));
		} catch (e) {
			error = readRefusal(e).message;
		}
	}

	async function saveConnection(event: SubmitEvent) {
		event.preventDefault();
		busy = 'connection';
		error = notice = null;
		try {
			await adminApi('relay/connection', { method: 'PUT', body: { control_url: controlUrl, control_key: controlKey } });
			await load();
			notice = m.relay_admin_connection_saved();
		} catch (e) {
			error = readRefusal(e).message;
		} finally {
			busy = null;
		}
	}

	async function saveSettings(event: SubmitEvent) {
		event.preventDefault();
		busy = 'settings';
		error = notice = null;
		try {
			await adminApi('relay/settings', { method: 'PUT', body: settingsBody(form, !!overview?.relay?.shared_smtp) });
			await load();
			notice = m.relay_admin_saved();
		} catch (e) {
			const refusal = readRefusal(e);
			error = refusal.message;
			if (refusal.checks.length) checks = refusal.checks;
		} finally {
			busy = null;
		}
	}

	onMount(load);
</script>

<section class="integration" aria-labelledby="relay-title">
	<header class="integration__head">
		<span class="integration__name" id="relay-title">Relay</span>
		<span class="integration__what">{m.relay_admin_what()}</span>
		{#if status}<span class="chip" data-on={status.on}>{status.label}</span>{/if}
	</header>

	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#if notice}<p class="notice" role="status">{notice}</p>{/if}
	{#if !canManage}<p class="muted small">{m.relay_admin_view_only()}</p>{/if}

	{#if overview}
		<form class="block" onsubmit={saveConnection}>
			<h3 class="block__title">{m.relay_admin_connection_title()}</h3>
			{#if overview.connection.source === 'environment'}<p class="muted small">{m.relay_admin_from_env()}</p>{/if}
			<div class="grid">
				<TextField label={m.relay_admin_control_url()} bind:value={controlUrl} type="url" required placeholder="https://relay.example.com" disabled={!canManage} />
				<TextField
					label={m.relay_admin_control_key()}
					bind:value={controlKey}
					type="password"
					autocomplete="off"
					required={!overview.connection.key_set}
					supportingText={overview.connection.key_set ? m.relay_admin_key_keep() : m.relay_admin_key_hint()}
					disabled={!canManage}
				/>
			</div>
			{#if canManage}
				<div class="actions"><Button variant="tonal" size="s" type="submit" disabled={busy !== null}>{m.relay_admin_save_connection()}</Button></div>
			{/if}
		</form>

		<form class="block" onsubmit={saveSettings}>
			<h3 class="block__title">{m.relay_admin_settings_title()}</h3>
			{#if !connected}
				<p class="muted small">{m.relay_admin_need_connection()}</p>
			{:else}
				<p class="muted small">{m.relay_admin_settings_hint()}</p>
				<div class="grid">
					<TextField label={m.relay_admin_public_domain()} bind:value={form.public_domain} required placeholder="api.relay.example.com" supportingText={m.relay_admin_public_domain_hint()} disabled={!canManage} />
					<TextField label={m.relay_admin_pgrst_role()} bind:value={form.postgrest_role} required disabled={!canManage} />
					<TextField
						label={m.relay_admin_dev_db()}
						bind:value={form.shared_dev_url}
						type="password"
						autocomplete="off"
						required={!overview.relay?.shared_dev_url}
						supportingText={overview.relay?.shared_dev_url ? m.relay_admin_db_keep({ url: overview.relay.shared_dev_url }) : m.relay_admin_db_hint()}
						disabled={!canManage}
					/>
					<TextField
						label={m.relay_admin_prod_db()}
						bind:value={form.shared_production_url}
						type="password"
						autocomplete="off"
						required={!overview.relay?.shared_production_url}
						supportingText={overview.relay?.shared_production_url ? m.relay_admin_db_keep({ url: overview.relay.shared_production_url }) : m.relay_admin_db_hint()}
						disabled={!canManage}
					/>
					<TextField label={m.relay_admin_pgrst_dev()} bind:value={form.postgrest_dev_url} type="url" required placeholder="http://postgrest-dev:3000" disabled={!canManage} />
					<TextField label={m.relay_admin_pgrst_prod()} bind:value={form.postgrest_production_url} type="url" required placeholder="http://postgrest-production:3000" disabled={!canManage} />
				</div>
				<label class="area">
					<span class="area__label">{m.relay_admin_origins()}</span>
					<textarea bind:value={form.preview_origins} rows="3" disabled={!canManage}></textarea>
					<span class="area__hint">{m.relay_admin_origins_hint()}</span>
				</label>

				<fieldset class="sender">
					<legend class="sender__title">{m.relay_admin_sender_title()}</legend>
					<p class="muted small">{m.relay_admin_sender_hint()}</p>
					<div class="grid">
						<TextField label={m.relay_admin_smtp_host()} bind:value={form.smtp_host} placeholder="smtp.resend.com" disabled={!canManage} />
						<TextField label={m.relay_admin_smtp_port()} bind:value={form.smtp_port} type="number" inputmode="numeric" min="1" max="65535" disabled={!canManage} />
						<Select label={m.relay_admin_smtp_security()} name="smtp_security" options={securityOptions} bind:value={form.smtp_security} />
						<TextField label={m.relay_admin_smtp_from()} bind:value={form.smtp_from} placeholder="Nomi <codes@your-domain.com>" disabled={!canManage} />
						<TextField label={m.relay_admin_smtp_user()} bind:value={form.smtp_user} autocomplete="off" disabled={!canManage} />
						<TextField
							label={m.relay_admin_smtp_password()}
							bind:value={form.smtp_password}
							type="password"
							autocomplete="off"
							supportingText={overview.relay?.shared_smtp ? m.relay_admin_key_keep() : undefined}
							disabled={!canManage}
						/>
					</div>
				</fieldset>

				{#if checks.length}
					<div class="checks">
						<h4 class="checks__title">{m.relay_admin_checks()}</h4>
						<ul class="checks__list">
							{#each checks as check (check.name)}
								<li class="check" data-ok={check.ok}>
									<span class="check__mark" aria-hidden="true">{check.ok ? '✓' : '✕'}</span>
									<span class="check__name">{check.name}</span>
									<span class="check__message">{check.ok ? m.relay_admin_check_ok() : check.message}</span>
								</li>
							{/each}
						</ul>
					</div>
				{/if}

				{#if canManage}
					<div class="actions"><Button variant="filled" size="s" type="submit" disabled={busy !== null}>{m.relay_admin_save_settings()}</Button></div>
				{/if}
			{/if}
		</form>
	{/if}
</section>

<style>
	.integration {
		display: flex;
		flex-direction: column;
		gap: 14px;
		margin-top: 16px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
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
	.block {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.block__title {
		margin: 0;
		font-size: 0.9375rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 260px), 1fr));
		gap: 12px 16px;
	}
	.area {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.area__label,
	.area__hint {
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.area textarea {
		box-sizing: border-box;
		min-height: 48px;
		padding: 12px 14px;
		border: 1.5px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		resize: vertical;
	}
	.area textarea:hover:not(:disabled) {
		border-color: var(--md-sys-color-on-surface);
	}
	.area textarea:focus-visible {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 11.5px 13.5px;
	}
	.sender {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
		margin: 0;
		padding: 12px 0 0;
		border: none;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.sender__title {
		padding: 0;
		font-size: 0.875rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.checks {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.checks__title {
		margin: 0;
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.checks__list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.check {
		display: grid;
		grid-template-columns: 20px minmax(110px, auto) 1fr;
		align-items: baseline;
		gap: 8px;
		font-size: 0.875rem;
	}
	.check__mark {
		font-weight: 700;
		color: var(--md-sys-color-primary);
	}
	.check[data-ok='false'] .check__mark,
	.check[data-ok='false'] .check__message {
		color: var(--md-sys-color-error);
	}
	.check__name {
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.check__message {
		color: var(--md-sys-color-on-surface-variant);
		overflow-wrap: anywhere;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
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
	@media (max-width: 480px) {
		.check {
			grid-template-columns: 20px 1fr;
		}
		.check__message {
			grid-column: 2;
		}
	}
</style>
