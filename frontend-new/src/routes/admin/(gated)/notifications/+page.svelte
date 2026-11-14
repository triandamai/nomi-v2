<script lang="ts">
	import { BellOff, Send } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import type { Broadcast } from '$lib/admin/api';
	import { dateTime, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import TextArea from '$lib/ds/admin/TextArea.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';

	// Notifications (notifications: view): with write, a note to everyone's inbox or to the people
	// on one plan, sent after a confirm that names who gets it; then what went out, newest first,
	// with how many got it and read it.
	let { data, form } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'notifications', 'write'));
	const plans = $derived(data.plans?.ok ? data.plans.data : []);
	const audiences = $derived([{ value: 'all', label: m.admin_notify_everyone() }, ...plans.map((plan) => ({ value: plan.id, label: m.admin_notify_plan({ plan: plan.name }) }))]);

	let audience = $state('all');
	let confirming = $state(false);
	let sendForm: HTMLFormElement | undefined = $state();
	const audienceLabel = $derived(audiences.find((option) => option.value === audience)?.label ?? m.admin_notify_everyone());
	const sentTo = (value: string) => (value === 'all' ? m.admin_notify_everyone() : (plans.find((plan) => plan.id === value)?.name ?? m.admin_notify_a_plan()));

	const columns: TableColumn<Broadcast>[] = [
		{ key: 'title', label: m.admin_col_title() },
		{ key: 'audience', label: m.admin_col_audience(), phone: false },
		{ key: 'recipients', label: m.admin_col_subscribers(), align: 'end', sort: (row) => row.recipients },
		{ key: 'read', label: m.admin_col_read(), align: 'end', sort: (row) => row.read, phone: false },
		{ key: 'sent', label: m.admin_col_sent(), sort: (row) => row.created_at },
	];
	const sent = $derived(form?.action === 'send' && form.ok ? (form.data as { recipients: number }) : null);
</script>

<PageHeader title={m.admin_page_notifications()} intro={m.admin_notify_intro()} />

{#if canWrite}
	{#snippet footer()}<Button label={m.admin_notify_send()} variant="inverse" iconEnd={Send} onclick={() => sendForm?.reportValidity() && (confirming = true)} />{/snippet}
	<form bind:this={sendForm} method="POST" action="?/send" use:enhance={() => ({ update, result }) => update({ reset: result.type === 'success' })}>
		<FormSection title={m.admin_notify_compose()} description={m.admin_notify_compose_desc()} {footer}>
			<div class="grid gap-4 sm:grid-cols-2">
				<div class="sm:col-span-2"><TextField label={m.admin_notify_title()} name="title" required autocomplete="off" /></div>
				<div class="sm:col-span-2"><TextArea label={m.admin_notify_body()} name="body" required /></div>
				<TextField label={m.admin_notify_link()} name="link" helper={m.admin_notify_link_helper()} placeholder="/settings/plan" mono autocomplete="off" />
				<Select label={m.admin_notify_audience()} name="audience" bind:value={audience} options={audiences} />
			</div>
			{#if sent}
				<Notice kind="success" text={sent.recipients === 1 ? m.admin_notify_sent_one() : m.admin_notify_sent({ count: full(sent.recipients, language) })} />
			{:else if form?.action === 'send' && !form.ok}
				<Notice kind="error" text={m.admin_notify_failed()} detail={form.problem?.message || undefined} />
			{/if}
		</FormSection>
	</form>
	<ConfirmDialog
		bind:open={confirming}
		title={m.admin_notify_confirm_title({ audience: audienceLabel })}
		body={m.admin_notify_confirm_body()}
		confirmLabel={m.admin_notify_send()}
		cancelLabel={m.admin_cancel()}
		onconfirm={() => {
			confirming = false;
			sendForm?.requestSubmit();
		}}
	/>
{/if}

<section aria-label={m.admin_notify_history()} class="flex flex-col gap-3">
	<SectionLabel label={m.admin_notify_history()} />
	{#if data.broadcasts.ok}
		{#if data.broadcasts.data.length === 0}
			<EmptyState icon={BellOff} title={m.admin_notify_empty_title()} body={m.admin_notify_empty_body()} />
		{:else}
			<AdminTable label={m.admin_notify_history()} {columns} rows={data.broadcasts.data} rowKey={(row) => row.id}>
				{#snippet cell(row, column)}
					{#if column.key === 'title'}
						<span class="flex flex-col gap-0.5">
							<span class="font-medium text-text">{row.title}</span>
							<span class="line-clamp-2 text-caption text-text-3">{row.body}</span>
						</span>
					{:else if column.key === 'audience'}
						{sentTo(row.audience)}
					{:else if column.key === 'recipients'}
						<span class="font-mono">{full(row.recipients, language)}</span>
					{:else if column.key === 'read'}
						<span class="font-mono">{full(row.read, language)}</span>
					{:else if column.key === 'sent'}
						<span class="font-mono text-text-2">{dateTime(row.created_at, language)}</span>
					{/if}
				{/snippet}
			</AdminTable>
		{/if}
	{:else}
		<LoadFailed title={m.admin_notify_history_failed()} problem={data.broadcasts.problem} />
	{/if}
</section>
