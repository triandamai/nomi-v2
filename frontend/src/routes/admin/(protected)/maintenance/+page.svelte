<script lang="ts">
	// Admin → Maintenance: while it's on, everyone outside the admin area sees the maintenance
	// page (it notices by itself when it's off again).
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import DatePicker from '$lib/components/m3/DatePicker.svelte';
	import TimePicker from '$lib/components/m3/TimePicker.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { adminApi } from '$lib/adminApi';

	interface State { enabled: boolean; message: string; until: string | null; updated_at: string }

	let enabled = $state(false);
	let message = $state('');
	// When it's expected back: a day and (optionally) a time. No day means "back soon".
	let untilDate = $state('');
	let untilTime = $state('');
	let saving = $state(false);
	let saved = $state(false);
	let error = $state<string | null>(null);

	const pad = (n: number) => String(n).padStart(2, '0');

	function fill(iso: string | null) {
		if (!iso) {
			untilDate = '';
			untilTime = '';
			return;
		}
		const d = new Date(iso);
		untilDate = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
		untilTime = `${pad(d.getHours())}:${pad(d.getMinutes())}`;
	}

	/** The picked day and time in the admin's own timezone, or null for "back soon". */
	function untilIso(): string | null {
		if (!untilDate) return null;
		const [y, mo, d] = untilDate.split('-').map(Number);
		const [h, mi] = (untilTime || '00:00').split(':').map(Number);
		return new Date(y, mo - 1, d, h, mi).toISOString();
	}

	$effect(() => {
		adminApi<State>('maintenance')
			.then((s) => {
				enabled = s.enabled;
				message = s.message;
				fill(s.until);
			})
			.catch((e) => (error = (e as Error).message));
	});

	async function save(event: SubmitEvent) {
		event.preventDefault();
		saving = true;
		saved = false;
		error = null;
		try {
			await adminApi('maintenance', { method: 'PUT', body: { enabled, message, until: untilIso() } });
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
	<div class="when">
		<DatePicker id="maintenance-day" label={m.maintenance_until()} bind:value={untilDate} supportingText={m.maintenance_until_hint()} />
		{#if untilDate}
			<TimePicker id="maintenance-time" label={m.maintenance_until_time()} bind:value={untilTime} step={15} />
		{/if}
	</div>
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
	.when {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 200px), 1fr));
		gap: 12px;
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
