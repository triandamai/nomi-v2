<script lang="ts">
	// Admin → Payment: the default provider, and one provider's keys under a tab per provider.
	// Secrets never come back from the server: a field only says whether it is set and where from.
	import Button from '$lib/components/m3/Button.svelte';
	import Radio from '$lib/components/m3/Radio.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { adminApi } from '$lib/adminApi';
	import { can, type Level } from '$lib/permissions';
	import { m } from '$lib/paraglide/messages';

	type ProviderName = 'xendit' | 'doku';

	let { access, provider }: { access: Record<string, Level>; provider: ProviderName } = $props();
	const canManage = $derived(can(access, 'configuration', 'manage'));

	interface Field {
		name: string;
		env: string | null;
		kind: 'text' | 'number' | 'secret' | 'choice' | 'domain';
		value: string | null;
		set: boolean;
		source: 'saved' | 'environment' | 'none';
	}

	interface ProviderConfig {
		provider: ProviderName;
		fields: Field[];
		can_manage: boolean;
	}

	interface Overview {
		chosen: ProviderName | null;
		source: 'saved' | 'environment' | 'none';
		in_use: ProviderName | null;
		providers: { name: ProviderName; configured: boolean }[];
		can_manage: boolean;
	}

	interface TestResponse {
		success: boolean;
		message: string;
	}

	/** The radio value for "no provider chosen: the first one set up". */
	const AUTOMATIC = '';
	/** Brand names, the same in every language. */
	const PROVIDER_LABEL: Record<ProviderName, string> = { xendit: 'Xendit', doku: 'DOKU' };
	const PROVIDERS: ProviderName[] = ['xendit', 'doku'];

	const COPY: Record<ProviderName, { title: () => string; lede: () => string; fields: Record<string, { label: () => string; hint: () => string }> }> = {
		xendit: {
			title: m.pay_xendit_title,
			lede: m.pay_xendit_lede,
			fields: {
				secret_key: { label: m.pay_xendit_secret_key, hint: m.pay_xendit_secret_key_hint },
				webhook_token: { label: m.pay_xendit_webhook_token, hint: m.pay_xendit_webhook_token_hint },
				public_key: { label: m.pay_xendit_public_key, hint: m.pay_xendit_public_key_hint },
				base_url: { label: m.pay_xendit_base_url, hint: m.pay_xendit_base_url_hint },
				usd_to_idr: { label: m.pay_xendit_usd_to_idr, hint: m.pay_xendit_usd_to_idr_hint },
			},
		},
		doku: {
			title: m.pay_doku_title,
			lede: m.pay_doku_lede,
			fields: {
				client_id: { label: m.pay_doku_client_id, hint: m.pay_doku_client_id_hint },
				secret_key: { label: m.pay_doku_secret_key, hint: m.pay_doku_secret_key_hint },
				base_url: { label: m.pay_doku_base_url, hint: m.pay_doku_base_url_hint },
				notification_path: { label: m.pay_doku_notification_path, hint: m.pay_doku_notification_path_hint },
			},
		},
	};
	const copy = $derived(COPY[provider]);

	let overview = $state<Overview | null>(null);
	let overviewError = $state<string | null>(null);
	let choice = $state(AUTOMATIC);
	let choosing = $state(false);
	let chooseError = $state<string | null>(null);
	let chose = $state(false);

	let fields = $state<Field[] | null>(null);
	let loadError = $state<string | null>(null);
	let values = $state<Record<string, string>>({});
	let initial = $state<Record<string, string>>({});
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let saved = $state(false);

	let testing = $state(false);
	let testResult = $state<TestResponse | null>(null);

	function showOverview(next: Overview) {
		overview = next;
		choice = next.chosen ?? AUTOMATIC;
	}

	function populate(next: Field[]) {
		fields = next;
		for (const f of next) {
			const val = f.kind === 'secret' ? '' : (f.value ?? '');
			values[f.name] = val;
			initial[f.name] = val;
		}
	}

	async function loadOverview() {
		try {
			showOverview(await adminApi<Overview>('payment'));
			overviewError = null;
		} catch (e) {
			overviewError = (e as Error).message;
		}
	}

	async function loadProvider(name: ProviderName) {
		try {
			populate((await adminApi<ProviderConfig>(`payment/${name}`)).fields);
		} catch (e) {
			loadError = (e as Error).message;
		}
	}

	$effect(() => {
		void loadOverview();
	});

	$effect(() => {
		void loadProvider(provider);
	});

	const changes = $derived.by(() => {
		if (!fields) return {};
		const diff: Record<string, string> = {};
		for (const f of fields) {
			const current = values[f.name] ?? '';
			if (current !== initial[f.name]) diff[f.name] = current;
		}
		return diff;
	});
	const dirty = $derived(Object.keys(changes).length > 0);
	const choiceDirty = $derived(overview !== null && choice !== (overview.chosen ?? AUTOMATIC));
	const configured = (name: ProviderName) => overview?.providers.find((p) => p.name === name)?.configured ?? false;

	/** What the default-provider card says new payments do now. */
	const inUseText = $derived.by(() => {
		if (!overview) return '';
		if (overview.in_use) return m.pay_in_use({ provider: PROVIDER_LABEL[overview.in_use] });
		if (overview.chosen) return m.pay_chosen_missing({ provider: PROVIDER_LABEL[overview.chosen] });
		return m.pay_in_use_none();
	});

	function sourceText(field: Field): string {
		if (field.source === 'saved') return m.config_source_saved();
		if (field.source === 'environment') return m.config_source_env({ env: field.env ?? '' });
		return m.config_source_none({ env: field.env ?? '' });
	}

	function supportingText(field: Field): string {
		const hint = copy.fields[field.name]?.hint() ?? '';
		const keep = field.kind === 'secret' && field.set ? m.pay_keep_secret() : '';
		return [hint, keep, sourceText(field)].filter(Boolean).join(' ');
	}

	async function chooseProvider() {
		if (!choiceDirty) return;
		choosing = true;
		chooseError = null;
		chose = false;
		try {
			showOverview(await adminApi<Overview>('payment', { method: 'PUT', body: { provider: choice } }));
			chose = true;
		} catch (e) {
			chooseError = (e as Error).message;
		} finally {
			choosing = false;
		}
	}

	async function save() {
		if (!dirty || !fields) return;
		saving = true;
		saveError = null;
		saved = false;
		testResult = null;
		try {
			populate((await adminApi<ProviderConfig>(`payment/${provider}`, { method: 'PUT', body: { values: changes } })).fields);
			saved = true;
			// Saving keys can set a provider up, or stop it being set up.
			void loadOverview();
		} catch (e) {
			saveError = (e as Error).message;
		} finally {
			saving = false;
		}
	}

	async function testKeys() {
		testing = true;
		testResult = null;
		try {
			testResult = await adminApi<TestResponse>(`payment/${provider}/test`, { method: 'POST' });
		} catch (e) {
			testResult = { success: false, message: (e as Error).message };
		} finally {
			testing = false;
		}
	}
</script>

<PageHeader title={m.pay_title()} lede={m.pay_lede()} agent="money" />

<section class="card" aria-labelledby="pay-default-title">
	<h2 id="pay-default-title" class="card__title">{m.pay_default_title()}</h2>
	<p class="card__lede">{m.pay_default_lede()}</p>

	{#if overviewError}
		<p class="error" role="alert">{overviewError}</p>
	{:else if !overview}
		<p class="muted">{m.pj_loading()}</p>
	{:else}
		<form
			class="fields"
			onsubmit={(event) => {
				event.preventDefault();
				void chooseProvider();
			}}
		>
			<fieldset class="choices" disabled={!canManage}>
				<legend class="sr-only">{m.pay_default_title()}</legend>
				<div class="choice-row">
					<Radio name="payment-provider" value={AUTOMATIC} bind:group={choice} label={m.pay_default_auto()} disabled={!canManage} />
				</div>
				{#each PROVIDERS as name (name)}
					<div class="choice-row">
						<Radio name="payment-provider" value={name} bind:group={choice} label={PROVIDER_LABEL[name]} disabled={!canManage} />
						<span class="status" class:status--ready={configured(name)}>
							{configured(name) ? m.pay_status_ready() : m.pay_status_missing()}
						</span>
					</div>
				{/each}
			</fieldset>

			<p class="in-use" class:in-use--warn={!overview.in_use} role="status">{inUseText}</p>
			{#if overview.source === 'environment'}
				<p class="muted small">{m.pay_default_from_env()}</p>
			{/if}

			{#if canManage}
				<div class="actions">
					{#if chooseError}<p class="error" role="alert">{chooseError}</p>{/if}
					{#if chose && !choiceDirty}<p class="muted small" role="status">{m.pay_default_saved()}</p>{/if}
					<div class="button-group">
						<Button variant="filled" type="submit" disabled={choosing || !choiceDirty}>
							{choosing ? m.config_saving() : m.config_save()}
						</Button>
					</div>
				</div>
			{/if}
		</form>
	{/if}
</section>

<nav class="tabs-nav" aria-label={m.pay_tabs_label()}>
	{#each PROVIDERS as name (name)}
		<a href="/admin/payment/{name}" class="tab-link" class:active={name === provider} aria-current={name === provider ? 'page' : undefined}>
			{PROVIDER_LABEL[name]}
			{#if overview}
				<span class="tab-status">· {configured(name) ? m.pay_status_ready() : m.pay_status_missing()}</span>
			{/if}
		</a>
	{/each}
</nav>

<section class="card" aria-labelledby="pay-provider-title">
	<h2 id="pay-provider-title" class="card__title">{copy.title()}</h2>
	<p class="card__lede">{copy.lede()}</p>

	{#if loadError}
		<p class="error" role="alert">{loadError}</p>
	{:else if !fields}
		<p class="muted">{m.pj_loading()}</p>
	{:else}
		<form
			class="fields"
			onsubmit={(event) => {
				event.preventDefault();
				void save();
			}}
		>
			{#each fields as field (field.name)}
				<TextField
					label={copy.fields[field.name]?.label() ?? field.name}
					bind:value={values[field.name]}
					type={field.kind === 'secret' ? 'password' : 'text'}
					autocomplete="off"
					disabled={!canManage}
					supportingText={supportingText(field)}
				/>
			{/each}

			<div class="actions">
				{#if !canManage}
					<p class="muted small">{m.config_view_only()}</p>
				{:else}
					{#if saveError}<p class="error" role="alert">{saveError}</p>{/if}
					{#if saved && !dirty}<p class="muted small" role="status">{m.config_saved()}</p>{/if}
					<div class="button-group">
						<Button variant="tonal" type="button" disabled={testing || dirty} onclick={testKeys}>
							{testing ? m.pay_testing() : m.pay_test()}
						</Button>
						<Button variant="filled" type="submit" disabled={saving || !dirty}>
							{saving ? m.config_saving() : m.config_save()}
						</Button>
					</div>
				{/if}
			</div>

			{#if testResult}
				<div class="test-banner" class:success={testResult.success} class:fail={!testResult.success} role="status">
					<strong>{testResult.success ? m.pay_test_ok() : m.pay_test_failed()}:</strong>
					{testResult.message}
				</div>
			{/if}
		</form>
	{/if}
</section>

<style>
	.tabs-nav {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		max-width: 680px;
		margin: 28px 0 20px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		padding-bottom: 4px;
	}

	.tab-link {
		padding: 8px 16px;
		font-weight: 600;
		text-decoration: none;
		color: var(--md-sys-color-on-surface-variant);
		border-bottom: 2px solid transparent;
		transition:
			color 0.15s,
			border-color 0.15s;
	}

	.tab-link.active {
		color: var(--md-sys-color-primary);
		border-bottom-color: var(--md-sys-color-primary);
	}

	.tab-status {
		font-weight: 400;
		font-size: 0.8125rem;
	}

	.card {
		max-width: 680px;
		padding: 24px clamp(16px, 3vw, 32px);
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
		border: 1px solid var(--md-sys-color-outline-variant);
	}

	.card__title {
		margin: 0 0 6px;
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}

	.card__lede {
		margin: 0 0 24px;
		font-size: 0.875rem;
		line-height: 1.45;
		color: var(--md-sys-color-on-surface-variant);
	}

	.fields {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.choices {
		display: flex;
		flex-direction: column;
		gap: 12px;
		margin: 0;
		padding: 0;
		border: 0;
	}

	.choice-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}

	.status {
		padding: 2px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		font-size: 0.75rem;
		font-weight: 600;
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}

	.status--ready {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}

	.in-use {
		margin: 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
	}

	.in-use--warn {
		color: var(--md-sys-color-error);
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding-top: 12px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}

	.button-group {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
		margin-left: auto;
	}

	.test-banner {
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-medium);
		font-size: 0.875rem;
	}

	.test-banner.success {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}

	.test-banner.fail {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}

	.sr-only {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
		white-space: nowrap;
	}

	.error {
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}

	.muted {
		color: var(--md-sys-color-on-surface-variant);
	}

	.small {
		font-size: 0.8125rem;
	}
</style>
