<script lang="ts">
	// Admin → Configuration → one section: what used to be set in the server's environment (attachments, file
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
	import { SECTIONS, type Field, type Section } from '../sections';
	import type { PageData } from './$types';

	let { data, params }: { data: PageData; params: { section: string } } = $props();
	const canManage = $derived(can(data.access, 'configuration', 'manage'));

	let sections = $state<Section[] | null>(null);
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

	const current = $derived(sections?.find((s) => s.name === params.section) ?? null);
	const changes = $derived.by(() => {
		if (!current) return {};
		const changed: Record<string, string> = {};
		for (const field of current.fields) {
			const k = key(current.name, field.name);
			// A number input's bind:value hands back a number; the backend takes every value as text.
			const value = String(values[k] ?? '');
			if (value !== initial[k]) changed[field.name] = value;
		}
		return changed;
	});
	const dirty = $derived(Object.keys(changes).length > 0);

	let testingStorage = $state(false);
	let storageTestResult = $state<{ success: boolean; message: string } | null>(null);

	async function testStorageConnection() {
		if (!current || current.name !== 'storage') return;
		testingStorage = true;
		storageTestResult = null;
		try {
			const res = await adminApi<{ success: boolean; message: string }>('configuration/storage/test', {
				method: 'POST',
				body: { values: changes }
			});
			storageTestResult = res;
		} catch (e) {
			storageTestResult = { success: false, message: (e as Error).message };
		} finally {
			testingStorage = false;
		}
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
		if (field.source === 'environment' && field.env) return m.config_source_env({ env: field.env });
		return field.env ? m.config_source_none({ env: field.env }) : m.config_source_unset();
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
	<a class="back" href="/admin/configuration">{m.config_back()}</a>

	{#if !current || !SECTIONS[current.name]}
		<p class="error" role="alert">{m.config_unknown_section()}</p>
	{:else}
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
						{#if storageTestResult}
							<p class={storageTestResult.success ? 'success small' : 'error small'} role="alert">
								{storageTestResult.message}
							</p>
						{/if}
						{#if saveError}<p class="error" role="alert">{saveError}</p>{/if}
						{#if saved && !dirty}<p class="muted small" role="status">{m.config_saved()}</p>{/if}
						{#if current.name === 'storage'}
							<Button variant="outlined" type="button" disabled={testingStorage} onclick={testStorageConnection}>
								{testingStorage ? 'Testing connection...' : 'Test connection'}
							</Button>
						{/if}
						<Button variant="filled" type="submit" disabled={saving || !dirty}>{saving ? m.config_saving() : m.config_save()}</Button>
					{/if}
				</div>
			</form>
		</section>
	{/if}
{/if}

<style>
	.back {
		display: inline-flex;
		align-items: center;
		min-height: 44px;
		margin-bottom: 8px;
		color: var(--md-sys-color-primary);
		font-weight: 600;
		text-decoration: none;
	}
	.back:hover {
		text-decoration: underline;
	}
	.back:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
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
	.success {
		margin: 0;
		color: var(--md-sys-color-primary);
		overflow-wrap: anywhere;
	}
</style>
