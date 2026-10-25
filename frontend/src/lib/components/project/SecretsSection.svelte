<script lang="ts">
	// The project's secrets: API keys and the like its published app reads from its environment.
	// A value goes straight to Shipyard and never comes back: the list shows names only, and
	// "Replace" asks for a new value rather than showing the old one.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { ago, projectApi } from '$lib/projectApi';

	let { projectId }: { projectId: string } = $props();

	interface Secret { name: string; updated_at: string; updated_by_name: string | null }
	interface Secrets { secrets: Secret[]; can_edit: boolean }

	/** What a name must look like: the backend checks it too. */
	const NAME = /^[A-Z_][A-Z0-9_]{0,63}$/;
	const RESERVED = ['DATABASE_URL', 'PORT', 'NODE_ENV', 'HOST'];

	let data = $state<Secrets | null>(null);
	let loadError = $state<string | null>(null);

	async function load() {
		try {
			data = await projectApi<Secrets>(projectId, 'secrets');
			loadError = null;
		} catch (e) {
			loadError = (e as Error).message;
		}
	}
	$effect(() => {
		void projectId;
		void load();
	});

	// ─── Add or replace ──────────────────────────────────────────
	let open = $state(false);
	/** The secret being replaced; null when adding one. */
	let replacing = $state<string | null>(null);
	let name = $state('');
	let value = $state('');
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let done = $state<string | null>(null);

	const typedName = $derived(name.trim().toUpperCase());
	const nameProblem = $derived(
		!typedName
			? null
			: RESERVED.includes(typedName)
				? m.secrets_reserved({ name: typedName })
				: NAME.test(typedName)
					? null
					: m.secrets_name_rule(),
	);
	const exists = $derived(!replacing && !!data?.secrets.some((s) => s.name === typedName));

	function openSheet(secret: string | null) {
		replacing = secret;
		name = secret ?? '';
		value = '';
		saveError = null;
		open = true;
	}

	async function save() {
		saving = true;
		saveError = null;
		try {
			await projectApi(projectId, `secrets/${encodeURIComponent(typedName)}`, { method: 'PUT', body: { value } });
			open = false;
			value = '';
			done = m.secrets_saved({ name: typedName });
			await load();
		} catch (e) {
			saveError = (e as Error).message;
		} finally {
			saving = false;
		}
	}

	// ─── Remove ──────────────────────────────────────────────────
	let removing = $state<string | null>(null);
	let removeOpen = $state(false);
	let removeError = $state<string | null>(null);

	async function remove() {
		const target = removing;
		if (!target) return;
		removeOpen = false;
		try {
			await projectApi(projectId, `secrets/${encodeURIComponent(target)}`, { method: 'DELETE' });
			done = m.secrets_removed({ name: target });
			removeError = null;
			await load();
		} catch (e) {
			removeError = (e as Error).message;
		}
	}
</script>

<section class="block">
	<div class="block__row">
		<div class="block__text">
			<h3 class="block__title">{m.secrets_title()}</h3>
			<p class="block__hint">{m.secrets_hint()}</p>
		</div>
		{#if data?.can_edit}
			<Button variant="tonal" size="s" onclick={() => openSheet(null)}>{m.secrets_add()}</Button>
		{/if}
	</div>

	{#if loadError}
		<p class="error" role="alert">{loadError}</p>
	{:else if !data}
		<p class="block__hint">{m.common_loading()}</p>
	{:else if data.secrets.length === 0}
		<p class="block__hint">{m.secrets_empty()}</p>
	{:else}
		<ul class="secrets">
			{#each data.secrets as secret (secret.name)}
				<li class="secret">
					<div class="secret__text">
						<span class="secret__name">{secret.name}</span>
						<span class="block__hint">
							{secret.updated_by_name
								? m.secrets_updated_by({ when: ago(secret.updated_at, getLocale()), who: secret.updated_by_name })
								: m.secrets_updated({ when: ago(secret.updated_at, getLocale()) })}
						</span>
					</div>
					{#if data.can_edit}
						<div class="secret__actions">
							<Button variant="text" size="s" onclick={() => openSheet(secret.name)}>{m.secrets_replace()}</Button>
							<Button variant="text" size="s" onclick={() => { removing = secret.name; removeOpen = true; }}>{m.secrets_remove()}</Button>
						</div>
					{/if}
				</li>
			{/each}
		</ul>
		<p class="block__hint">{m.secrets_next_publish()}</p>
	{/if}
	{#if data && !data.can_edit}<p class="block__hint">{m.pj_admins_only()}</p>{/if}
	{#if removeError}<p class="error" role="alert">{removeError}</p>{/if}
	{#if done}<p class="ok" role="status">{done}</p>{/if}
</section>

<BottomSheet bind:open label={replacing ? m.secrets_replace_title({ name: replacing }) : m.secrets_add_title()}>
	<form
		class="sheet"
		onsubmit={(e) => {
			e.preventDefault();
			if (typedName && !nameProblem && value) void save();
		}}
	>
		<h2 class="sheet__title">{replacing ? m.secrets_replace_title({ name: replacing }) : m.secrets_add_title()}</h2>
		<p class="block__hint">{m.secrets_sheet_lede()}</p>
		{#if !replacing}
			<TextField
				name="secret_name"
				label={m.secrets_name()}
				bind:value={name}
				placeholder="STRIPE_SECRET_KEY"
				autocomplete="off"
				autocapitalize="characters"
				spellcheck={false}
				maxlength={64}
				class="mono"
				error={!!nameProblem}
				supportingText={nameProblem ?? (exists ? m.secrets_exists() : m.secrets_name_rule())}
			/>
		{/if}
		<TextField
			name="secret_value"
			label={m.secrets_value()}
			type="password"
			bind:value
			autocomplete="off"
			spellcheck={false}
			maxlength={8192}
		/>
		{#if saveError}<p class="error" role="alert">{saveError}</p>{/if}
		<div class="sheet__actions">
			<Button variant="text" type="button" onclick={() => (open = false)}>{m.common_cancel()}</Button>
			<Button variant="filled" type="submit" disabled={saving || !typedName || !!nameProblem || !value}>{m.secrets_save()}</Button>
		</div>
	</form>
</BottomSheet>

<Dialog bind:open={removeOpen} headline={m.secrets_remove()}>
	<p>{m.secrets_remove_confirm({ name: removing ?? '' })}</p>
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (removeOpen = false)}>{m.common_cancel()}</Button>
		<Button variant="filled" type="button" onclick={remove}>{m.secrets_remove()}</Button>
	{/snippet}
</Dialog>

<style>
	.block {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 20px;
		padding: 16px 18px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
	}
	.block__row {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-start;
		justify-content: space-between;
		gap: 8px 16px;
	}
	.block__text {
		display: flex;
		flex: 1 1 220px;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.block__title {
		margin: 0;
		font-size: 0.9375rem;
		font-weight: 700;
	}
	.block__hint {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.secrets {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.secret {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 4px 12px;
		padding: 8px 0;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.secret:first-child {
		border-top: none;
	}
	.secret__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.secret__name {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.9375rem;
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.secret__actions {
		display: flex;
		gap: 4px;
	}
	.sheet {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 8px 16px 16px;
	}
	.sheet__title {
		margin: 0;
		font-size: 1.375rem;
		font-weight: 700;
	}
	.sheet :global(.mono input) {
		font-family: var(--md-ref-typeface-mono);
		text-transform: uppercase;
	}
	.sheet__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 8px;
	}
	.ok {
		margin: 0;
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
</style>
