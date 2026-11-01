<script lang="ts">
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { adminApi } from '$lib/adminApi';
	import { can } from '$lib/permissions';
	import { m } from '$lib/paraglide/messages';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	const canManage = $derived(can(data.access, 'configuration', 'manage'));

	interface Field {
		name: string;
		env: string | null;
		kind: 'text' | 'number' | 'secret' | 'choice' | 'domain';
		value: string | null;
		set: boolean;
		source: 'saved' | 'environment' | 'none';
	}

	interface PaymentConfigResponse {
		provider: string;
		fields: Field[];
		can_manage: boolean;
	}

	interface TestResponse {
		success: boolean;
		message: string;
	}

	const FIELD_METADATA: Record<string, { label: string; hint: string }> = {
		secret_key: {
			label: 'Secret API Key',
			hint: 'Starts with xnd_development_ or xnd_production_. Used to authenticate API calls to Xendit.',
		},
		webhook_token: {
			label: 'Webhook Verification Token',
			hint: 'The verification token configured in your Xendit Dashboard -> Settings -> Callbacks (sent in x-callback-token header).',
		},
		public_key: {
			label: 'Public API Key (optional)',
			hint: 'Your Xendit public key, if needed for frontend client-side tokenization.',
		},
		base_url: {
			label: 'Base API URL (optional)',
			hint: 'Default: https://api.xendit.co. Useful when routing through a mock server or proxy.',
		},
	};

	let fields = $state<Field[] | null>(null);
	let loadError = $state<string | null>(null);
	let values = $state<Record<string, string>>({});
	let initial = $state<Record<string, string>>({});
	let saving = $state(false);
	let saveError = $state<string | null>(null);
	let saved = $state(false);

	let testing = $state(false);
	let testResult = $state<TestResponse | null>(null);

	function populate(dataFields: Field[]) {
		for (const f of dataFields) {
			const val = f.kind === 'secret' ? '' : (f.value ?? '');
			values[f.name] = val;
			initial[f.name] = val;
		}
	}

	async function load() {
		try {
			const res = await adminApi<PaymentConfigResponse>('payment/xendit');
			fields = res.fields;
			populate(res.fields);
		} catch (e) {
			loadError = (e as Error).message;
		}
	}

	$effect(() => {
		void load();
	});

	const changes = $derived.by(() => {
		if (!fields) return {};
		const diff: Record<string, string> = {};
		for (const f of fields) {
			const currentVal = values[f.name] ?? '';
			if (currentVal !== initial[f.name]) {
				diff[f.name] = currentVal;
			}
		}
		return diff;
	});

	const dirty = $derived(Object.keys(changes).length > 0);

	function sourceText(field: Field): string {
		if (field.source === 'saved') return 'Source: Saved in database.';
		if (field.source === 'environment') return `Source: From environment (${field.env}).`;
		return field.env ? `Source: Not set (can also use env ${field.env}).` : 'Source: Not set.';
	}

	function fieldSupportingText(field: Field): string {
		const meta = FIELD_METADATA[field.name]?.hint ?? '';
		const keepSecret = field.kind === 'secret' && field.set ? 'Leave empty to keep existing secret.' : '';
		return [meta, keepSecret, sourceText(field)].filter(Boolean).join(' ');
	}

	async function save() {
		if (!dirty || !fields) return;
		saving = true;
		saveError = null;
		saved = false;
		testResult = null;
		try {
			const res = await adminApi<PaymentConfigResponse>('payment/xendit', {
				method: 'PUT',
				body: { values: changes },
			});
			fields = res.fields;
			populate(res.fields);
			saved = true;
		} catch (e) {
			saveError = (e as Error).message;
		} finally {
			saving = false;
		}
	}

	async function testConnection() {
		testing = true;
		testResult = null;
		try {
			const res = await adminApi<TestResponse>('payment/xendit/test', {
				method: 'POST',
			});
			testResult = res;
		} catch (e) {
			testResult = {
				success: false,
				message: (e as Error).message,
			};
		} finally {
			testing = false;
		}
	}
</script>

<PageHeader
	title="Payment: Xendit"
	lede="Configure your Xendit payment gateway integration to create invoices and receive webhook payment updates."
	agent="money"
/>

{#if loadError}
	<p class="error" role="alert">{loadError}</p>
{:else if !fields}
	<p class="muted">{m.pj_loading()}</p>
{:else}
	<div class="tabs-nav">
		<a href="/admin/payment/xendit" class="tab-link active">Xendit</a>
	</div>

	<section class="card" aria-labelledby="xendit-config-title">
		<h2 id="xendit-config-title" class="card__title">Xendit API Configuration</h2>
		<p class="card__lede">
			Credentials can be saved securely in Nomi's database or read from environment variables (<code>XENDIT_SECRET_KEY</code>, <code>XENDIT_WEBHOOK_TOKEN</code>).
		</p>

		<form
			class="fields"
			onsubmit={(event) => {
				event.preventDefault();
				void save();
			}}
		>
			{#each fields as field (field.name)}
				{@const meta = FIELD_METADATA[field.name]}
				<TextField
					label={meta?.label ?? field.name}
					bind:value={values[field.name]}
					type={field.kind === 'secret' ? 'password' : 'text'}
					autocomplete="off"
					disabled={!canManage}
					supportingText={fieldSupportingText(field)}
				/>
			{/each}

			<div class="actions">
				{#if !canManage}
					<p class="muted small">{m.config_view_only()}</p>
				{:else}
					{#if saveError}<p class="error" role="alert">{saveError}</p>{/if}
					{#if saved && !dirty}<p class="muted small" role="status">Settings saved successfully.</p>{/if}
					<div class="button-group">
						<Button variant="tonal" type="button" disabled={testing || dirty} onclick={testConnection}>
							{testing ? 'Testing…' : 'Test Client Config'}
						</Button>
						<Button variant="filled" type="submit" disabled={saving || !dirty}>
							{saving ? m.config_saving() : m.config_save()}
						</Button>
					</div>
				{/if}
			</div>

			{#if testResult}
				<div class="test-banner" class:success={testResult.success} class:fail={!testResult.success}>
					<strong>{testResult.success ? 'Success' : 'Failed'}:</strong> {testResult.message}
				</div>
			{/if}
		</form>
	</section>
{/if}

<style>
	.tabs-nav {
		display: flex;
		gap: 8px;
		margin-bottom: 20px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		padding-bottom: 4px;
	}

	.tab-link {
		padding: 8px 16px;
		font-weight: 600;
		text-decoration: none;
		color: var(--md-sys-color-on-surface-variant);
		border-bottom: 2px solid transparent;
		transition: color 0.15s, border-color 0.15s;
	}

	.tab-link.active {
		color: var(--md-sys-color-primary);
		border-bottom-color: var(--md-sys-color-primary);
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

	.actions {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		padding-top: 12px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}

	.button-group {
		display: flex;
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
