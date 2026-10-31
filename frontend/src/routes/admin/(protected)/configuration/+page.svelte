<script lang="ts">
	// Admin → Configuration: what used to be set in the server's environment (attachments, file
	// storage, Google, email and the agent loop), saved in Nomi instead. The environment still
	// fills anything not saved here, and each field says where its value comes from. Secrets are
	// never shown back. Seeing the page takes the configuration area; saving takes manage.
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { adminApi } from '$lib/adminApi';
	import { can } from '$lib/permissions';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const canManage = $derived(can(data.access, 'configuration', 'manage'));

	interface Field {
		name: string;
		env: string;
		kind: 'text' | 'number' | 'secret' | 'choice';
		choices?: string[];
		value: string | null;
		set: boolean;
		source: 'saved' | 'environment' | 'none';
	}
	interface Section {
		name: string;
		fields: Field[];
	}

	type Copy = { label: () => string; hint?: () => string };
	const SECTIONS: Record<string, { title: () => string; lede: () => string; note?: () => string; fields: Record<string, Copy> }> = {
		attachments: {
			title: m.config_attachments,
			lede: m.config_attachments_lede,
			fields: {
				max_file_mb: { label: m.config_attachments_max_file_mb },
				max_video_mb: { label: m.config_attachments_max_video_mb },
				max_message_mb: { label: m.config_attachments_max_message_mb },
				max_files: { label: m.config_attachments_max_files },
				quota_mb: { label: m.config_attachments_quota_mb, hint: m.config_attachments_quota_mb_hint },
			},
		},
		storage: {
			title: m.config_storage,
			lede: m.config_storage_lede,
			note: m.config_storage_restart,
			fields: {
				bucket: { label: m.config_storage_bucket },
				region: { label: m.config_storage_region, hint: m.config_storage_region_hint },
				access_key_id: { label: m.config_storage_access_key_id },
				secret_access_key: { label: m.config_storage_secret_access_key },
				endpoint_url: { label: m.config_storage_endpoint_url, hint: m.config_storage_endpoint_url_hint },
				public_url_base: { label: m.config_storage_public_url_base, hint: m.config_storage_public_url_base_hint },
			},
		},
		google: {
			title: m.config_google,
			lede: m.config_google_lede,
			fields: {
				client_id: { label: m.config_google_client_id },
				client_secret: { label: m.config_google_client_secret },
				redirect_uri: { label: m.config_google_redirect_uri, hint: m.config_google_redirect_uri_hint },
				signin_redirect_uri: { label: m.config_google_signin_redirect_uri, hint: m.config_google_signin_redirect_uri_hint },
			},
		},
		github: {
			title: m.config_github,
			lede: m.config_github_lede,
			fields: {
				client_id: { label: m.config_github_client_id },
				client_secret: { label: m.config_github_client_secret },
				redirect_uri: { label: m.config_github_redirect_uri, hint: m.config_github_redirect_uri_hint },
			},
		},
		smtp: {
			title: m.config_smtp,
			lede: m.config_smtp_lede,
			fields: {
				host: { label: m.config_smtp_host },
				port: { label: m.config_smtp_port, hint: m.config_smtp_port_hint },
				username: { label: m.config_smtp_username },
				password: { label: m.config_smtp_password },
				security: { label: m.config_smtp_security },
				from: { label: m.config_smtp_from, hint: m.config_smtp_from_hint },
			},
		},
		agent_loop: {
			title: m.config_agent_loop,
			lede: m.config_agent_loop_lede,
			fields: {
				turn_concurrency: { label: m.config_agent_loop_turn_concurrency, hint: m.config_agent_loop_turn_concurrency_hint },
				turn_time_limit_secs: { label: m.config_agent_loop_turn_time_limit_secs, hint: m.config_agent_loop_turn_time_limit_secs_hint },
				delegation_time_limit_secs: { label: m.config_agent_loop_delegation_time_limit_secs, hint: m.config_agent_loop_delegation_time_limit_secs_hint },
			},
		},
		relay: {
			title: m.config_relay,
			lede: m.config_relay_lede,
			fields: {
				relay_image: { label: m.config_relay_relay_image, hint: m.config_relay_relay_image_hint },
				functions_image: { label: m.config_relay_functions_image, hint: m.config_relay_functions_image_hint },
				postgrest_image: { label: m.config_relay_postgrest_image, hint: m.config_relay_postgrest_image_hint },
				version: { label: m.config_relay_version, hint: m.config_relay_version_hint },
			sdk_package: { label: m.config_relay_sdk_package, hint: m.config_relay_sdk_package_hint },
			sdk_version: { label: m.config_relay_sdk_version, hint: m.config_relay_sdk_version_hint },
			cli_package: { label: m.config_relay_cli_package, hint: m.config_relay_cli_package_hint },
			cli_version: { label: m.config_relay_cli_version, hint: m.config_relay_cli_version_hint },
			},
		},
	};
	const ORDER = ['attachments', 'storage', 'google', 'smtp', 'agent_loop', 'relay'];

	let sections = $state<Section[] | null>(null);
	let tab = $state('attachments');
	let loadError = $state<string | null>(null);
	// What's in each field now, and what it held when loaded (only changes are sent).
	let values = $state<Record<string, string>>({});
	let initial = $state<Record<string, string>>({});
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let saved = $state(false);

	const key = (section: string, field: string) => `${section}.${field}`;

	function take(section: Section) {
		for (const field of section.fields) {
			const value = field.kind === 'secret' ? '' : (field.value ?? '');
			values[key(section.name, field.name)] = value;
			initial[key(section.name, field.name)] = value;
		}
	}

	async function load() {
		try {
			const answer = await adminApi<{ sections: Section[] }>('configuration');
			sections = answer.sections;
			for (const section of answer.sections) take(section);
		} catch (e) {
			loadError = (e as Error).message;
		}
	}

	$effect(() => {
		void load();
	});

	const current = $derived(sections?.find((s) => s.name === tab) ?? null);
	const changes = $derived.by(() => {
		if (!current) return {};
		const changed: Record<string, string> = {};
		for (const field of current.fields) {
			const k = key(current.name, field.name);
			if (values[k] !== initial[k]) changed[field.name] = values[k];
		}
		return changed;
	});
	const dirty = $derived(Object.keys(changes).length > 0);

	function select(name: string) {
		tab = name;
		saveError = null;
		saved = false;
	}

	async function save() {
		if (!current || !dirty) return;
		saving = true;
		saveError = null;
		saved = false;
		try {
			const section = await adminApi<Section>(`configuration/${current.name}`, { method: 'PUT', body: { values: changes } });
			sections = (sections ?? []).map((s) => (s.name === section.name ? section : s));
			take(section);
			saved = true;
		} catch (e) {
			saveError = (e as Error).message;
		} finally {
			saving = false;
		}
	}

	function source(field: Field): string {
		if (field.source === 'saved') return m.config_source_saved();
		if (field.source === 'environment') return m.config_source_env({ env: field.env });
		return m.config_source_none({ env: field.env });
	}

	function hint(section: string, field: Field): string {
		const own = SECTIONS[section]?.fields[field.name]?.hint?.();
		const keep = field.kind === 'secret' && field.set ? m.relay_admin_key_keep() : null;
		return [own, keep, source(field)].filter(Boolean).join(' ');
	}
</script>

<PageHeader title={m.admin_configuration()} lede={m.config_lede()} agent="coding" />

{#if loadError}
	<p class="error" role="alert">{loadError}</p>
{:else if !sections}
	<p class="muted">{m.pj_loading()}</p>
{:else}
	<div class="tabs" role="tablist" aria-label={m.admin_configuration()}>
		{#each ORDER as name (name)}
			<button type="button" role="tab" class="tab" aria-selected={tab === name} onclick={() => select(name)}>{SECTIONS[name].title()}</button>
		{/each}
	</div>

	{#if current}
		{@const copy = SECTIONS[current.name]}
		<section class="card" aria-labelledby="config-title">
			<h2 id="config-title" class="card__title">{copy.title()}</h2>
			<p class="card__lede">{copy.lede()}</p>
			{#if copy.note}<p class="note">{copy.note()}</p>{/if}

			<form
				class="fields"
				onsubmit={(event) => {
					event.preventDefault();
					void save();
				}}
			>
				{#each current.fields as field (field.name)}
					{@const label = copy.fields[field.name]?.label() ?? field.name}
					{#if field.kind === 'choice' && canManage}
						<div class="choice">
							<Select
								{label}
								name={field.name}
								bind:value={values[key(current.name, field.name)]}
								options={[{ value: '', label: m.config_choice_default() }, ...(field.choices ?? []).map((c) => ({ value: c, label: c }))]}
							/>
							<p class="muted small">{hint(current.name, field)}</p>
						</div>
					{:else}
						<TextField
							{label}
							bind:value={values[key(current.name, field.name)]}
							type={field.kind === 'secret' ? 'password' : field.kind === 'number' ? 'number' : 'text'}
							min={field.kind === 'number' ? 1 : undefined}
							autocomplete="off"
							disabled={!canManage}
							supportingText={hint(current.name, field)}
						/>
					{/if}
				{/each}

				<div class="actions">
					{#if !canManage}
						<p class="muted small">{m.config_view_only()}</p>
					{:else}
						{#if saveError}<p class="error" role="alert">{saveError}</p>{/if}
						{#if saved && !dirty}<p class="muted small" role="status">{m.config_saved()}</p>{/if}
						<Button variant="filled" type="submit" disabled={saving || !dirty}>{saving ? m.config_saving() : m.config_save()}</Button>
					{/if}
				</div>
			</form>
		</section>
	{/if}
{/if}

<style>
	.tabs {
		display: flex;
		gap: 4px;
		margin-bottom: 16px;
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
	.tab:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
	}
	.tab:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.card {
		max-width: 720px;
		padding: 20px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.card__title {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 700;
	}
	.card__lede {
		margin: 4px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.note {
		margin: 12px 0 0;
		padding: 10px 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
	}
	.fields {
		display: flex;
		flex-direction: column;
		gap: 14px;
		margin-top: 18px;
	}
	.choice {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: flex-end;
		gap: 12px;
	}
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.small {
		font-size: 0.875rem;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
		overflow-wrap: anywhere;
	}
</style>
