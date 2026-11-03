<script lang="ts">
	// Deleting one of the crew's replies. A reply that only said something gets the design system's
	// dialog (Chat responses → Delete Dialog: the reply goes, your message stays); one that also
	// did things (a transaction, a reminder, files) gets the sheet listing exactly what goes with it.
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import DeleteReplyDialog from '$lib/components/ds/chat/DeleteReplyDialog.svelte';
	import MessageDeleteSheet from '$lib/components/MessageDeleteSheet.svelte';
	import type { MessageDeletion } from '$lib/types';

	let {
		open = $bindable(false),
		messageId,
		name,
		ondeleted,
	}: { open?: boolean; messageId: string; name: string; ondeleted?: (ids: string[]) => void } = $props();

	const url = $derived(`${page.url.pathname}/message/${messageId}/delete`);
	let simple = $state(false);
	let sheet = $state(false);
	let error = $state<string | null>(null);

	$effect(() => {
		if (!open) return;
		open = false;
		error = null;
		fetch(url)
			.then(async (response) => {
				const preview = response.ok ? ((await response.json()) as MessageDeletion) : null;
				const related = preview ? preview.transactions.length + preview.reminders.length + preview.plans.length + preview.tasks.length + preview.files.length : 1;
				if (preview && related === 0 && preview.message_ids.length === 1) simple = true;
				else sheet = true;
			})
			.catch(() => (sheet = true));
	});

	async function remove() {
		const response = await fetch(url, { method: 'DELETE' }).catch(() => null);
		if (!response?.ok) {
			error = m.msg_delete_failed();
			return;
		}
		const done = (await response.json()) as MessageDeletion;
		ondeleted?.(done.message_ids);
	}
</script>

<DeleteReplyDialog bind:open={simple} {name} ondelete={remove} />
<MessageDeleteSheet bind:open={sheet} {messageId} {ondeleted} />
{#if error}<p class="delete-error" role="alert">{error}</p>{/if}

<style>
	.delete-error {
		margin: 4px 0 0;
		color: var(--md-sys-color-error);
		font-size: 0.8125rem;
	}
</style>
