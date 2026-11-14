<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import { dateTime, localParts } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import TextArea from '$lib/ds/admin/TextArea.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Maintenance (maintenance: view; manage changes it): whether everyone but this console is held
	// on the maintenance page, its message, and when Nomi is back (UTC). Turning it on asks first,
	// since everyone is moved there at once; turning it off doesn't.
	let { data, form } = $props();

	const language = getLocale();
	const MAX = 500;
	const canManage = $derived(can(data.access, 'maintenance', 'manage'));
	const initial = () => (data.maintenance.ok ? data.maintenance.data.enabled : false);
	let enabled = $state(initial());
	let confirming = $state(false);
	let saveForm: HTMLFormElement | undefined = $state();

	function save() {
		const wasOn = data.maintenance.ok && data.maintenance.data.enabled;
		if (enabled && !wasOn) confirming = true;
		else saveForm?.requestSubmit();
	}
</script>

<PageHeader title={m.admin_page_maintenance()} intro={m.admin_maintenance_intro()} />

{#if data.maintenance.ok}
	{@const current = data.maintenance.data}
	{@const until = localParts(current.until, 'UTC')}
	{#snippet footer()}<Button label={m.admin_save()} variant="inverse" onclick={save} />{/snippet}
	<form bind:this={saveForm} method="POST" action="?/save" use:enhance={() => ({ update }) => update({ reset: false })}>
		<FormSection title={m.admin_maintenance_section()} footer={canManage ? footer : undefined}>
			{#snippet aside()}
				<Status kind={current.enabled ? 'attention' : 'live'} label={current.enabled ? m.admin_maintenance_on() : m.admin_maintenance_off()} />
			{/snippet}
			<div class="flex flex-col gap-4">
				<Toggle label={m.admin_maintenance_switch()} bind:checked={enabled} disabled={!canManage} />
				<input type="hidden" name="enabled" value={enabled ? 'on' : ''} />
				<TextArea label={m.admin_maintenance_message()} name="message" value={current.message} helper={m.admin_maintenance_message_helper({ max: String(MAX) })} disabled={!canManage} />
				<div class="grid gap-4 sm:grid-cols-2">
					<TextField label={m.admin_maintenance_until_date()} name="until_date" value={until.date} placeholder="2026-11-12" helper={m.admin_maintenance_until_helper()} mono disabled={!canManage} autocomplete="off" />
					<TextField label={m.admin_maintenance_until_time()} name="until_time" value={until.time} placeholder="03:30" mono disabled={!canManage} autocomplete="off" />
				</div>
				<p class="font-mono text-label text-text-3">{m.admin_maintenance_updated({ when: dateTime(current.updated_at, language) })}</p>
			</div>
			{#if form?.action === 'save' && form.ok}
				<Notice kind="success" text={current.enabled ? m.admin_maintenance_saved_on() : m.admin_maintenance_saved_off()} />
			{:else if form?.action === 'save' && !form.ok}
				<Notice kind="error" text={m.admin_maintenance_save_failed()} detail={form.problem?.message || undefined} />
			{/if}
		</FormSection>
	</form>
	<ConfirmDialog
		bind:open={confirming}
		title={m.admin_maintenance_confirm_title()}
		body={m.admin_maintenance_confirm_body()}
		confirmLabel={m.admin_maintenance_confirm()}
		cancelLabel={m.admin_cancel()}
		onconfirm={() => {
			confirming = false;
			saveForm?.requestSubmit();
		}}
	/>
{:else}
	<LoadFailed title={m.admin_maintenance_failed()} problem={data.maintenance.problem} />
{/if}
