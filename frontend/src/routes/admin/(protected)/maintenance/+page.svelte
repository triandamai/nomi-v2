<script lang="ts">
	// Admin → Maintenance: while it's on, everyone outside the admin area sees the maintenance
	// page (it notices by itself when it's off again).
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { adminApi } from '$lib/adminApi';

	interface State { enabled: boolean; message: string; until: string | null; updated_at: string }

	let enabled = $state(false);
	let message = $state('');
	let until = $state('');
	let saving = $state(false);
	let saved = $state(false);
	let error = $state<string | null>(null);

	function toLocalInput(iso: string | null): string {
		if (!iso) return '';
		const d = new Date(iso);
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
	}

	$effect(() => {
		adminApi<State>('maintenance')
			.then((s) => {
				enabled = s.enabled;
				message = s.message;
				until = toLocalInput(s.until);
			})
			.catch((e) => (error = (e as Error).message));
	});

	async function save(event: SubmitEvent) {
		event.preventDefault();
		saving = true;
		saved = false;
		error = null;
		try {
			await adminApi('maintenance', { method: 'PUT', body: { enabled, message, until: until ? new Date(until).toISOString() : null } });
			saved = true;
		} catch (e) {
			error = (e as Error).message;
		} finally {
			saving = false;
		}
	}
</script>

<PageHeader title={m.admin_maintenance()} lede={m.maintenance_admin_lede()} agent="supervisor" />

<form class="panel" onsubmit={save}>
	<label class="toggle">
		<span><strong>{m.maintenance_mode()}</strong><span class="hint">{enabled ? m.maintenance_on_hint() : m.maintenance_off_hint()}</span></span>
		<Switch bind:checked={enabled} aria-label={m.maintenance_mode()} />
	</label>
	<TextField name="message" label={m.maintenance_message()} bind:value={message} maxlength={500} />
	<label class="field">
		<span class="field__label">{m.maintenance_until()}</span>
		<input type="datetime-local" bind:value={until} />
	</label>
	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#if saved}<p class="ok" role="status">{m.maintenance_saved()}</p>{/if}
	<div><Button type="submit" variant="filled" disabled={saving}>{m.support_save()}</Button></div>
	<p class="hint"><a href="/maintenance" target="_blank" rel="noopener">{m.maintenance_preview()}</a></p>
</form>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
		max-width: 640px;
		padding: 18px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.toggle {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
	}
	.toggle > span {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.hint {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.hint a {
		color: var(--md-sys-color-primary);
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.field__label {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.field input {
		min-height: 44px;
		padding: 0 12px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: 8px;
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font: inherit;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.ok {
		margin: 0;
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
</style>
