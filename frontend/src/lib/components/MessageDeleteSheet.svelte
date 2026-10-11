<script lang="ts">
	// Asks before deleting a message, listing what goes with it: the replies to it, and the
	// transactions, reminders, plans, scheduled tasks and files it led to. Nothing can be undone.
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';
	import BottomSheet from './m3/BottomSheet.svelte';
	import Button from './m3/Button.svelte';
	import { formatAmount } from '$lib/money';
	import type { MessageDeletion, RelatedItem } from '$lib/types';

	let {
		open = $bindable(false),
		messageId,
		ondeleted,
	}: { open?: boolean; messageId: string; ondeleted?: (ids: string[]) => void } = $props();

	let preview = $state<MessageDeletion | null>(null);
	let loading = $state(false);
	let deleting = $state(false);
	let error = $state<string | null>(null);

	const url = $derived(`${page.url.pathname}/message/${messageId}/delete`);

	$effect(() => {
		if (!open) return;
		preview = null;
		error = null;
		loading = true;
		fetch(url)
			.then(async (response) => {
				if (response.status === 403) throw new Error(m.msg_delete_not_yours());
				if (!response.ok) throw new Error(m.msg_delete_failed());
				preview = (await response.json()) as MessageDeletion;
			})
			.catch((e: Error) => (error = e.message))
			.finally(() => (loading = false));
	});

	const groups = $derived(
		preview
			? ([
					{ title: m.msg_delete_transactions(), items: preview.transactions, money: true },
					{ title: m.msg_delete_reminders(), items: preview.reminders, money: false },
					{ title: m.msg_delete_plans(), items: preview.plans, money: false },
					{ title: m.msg_delete_tasks(), items: preview.tasks, money: false },
					{ title: m.msg_delete_files(), items: preview.files, money: false },
				] as { title: string; items: RelatedItem[]; money: boolean }[]).filter((group) => group.items.length > 0)
			: [],
	);
	const replies = $derived(preview ? preview.message_ids.length - 1 : 0);

	async function confirm() {
		deleting = true;
		error = null;
		try {
			const response = await fetch(url, { method: 'DELETE' });
			if (!response.ok) throw new Error();
			const done = (await response.json()) as MessageDeletion;
			open = false;
			ondeleted?.(done.message_ids);
		} catch {
			error = m.msg_delete_failed();
		} finally {
			deleting = false;
		}
	}
</script>

<BottomSheet bind:open>
	<div class="sheet">
		<h2 class="sheet__title">{groups.length > 0 ? m.msg_delete_title_related() : m.msg_delete_title()}</h2>
		{#if loading}
			<p class="sheet__lede">{m.common_loading()}</p>
		{:else if preview}
			<p class="sheet__lede">
				{#if replies > 0}{replies === 1 ? m.msg_delete_with_reply() : m.msg_delete_with_replies({ count: replies })}{/if}
				{groups.length > 0 ? m.msg_delete_related_lede() : m.msg_delete_lede()}
			</p>
			{#each groups as group (group.title)}
				<section class="group">
					<h3 class="group__title">{group.title}</h3>
					<ul class="group__list">
						{#each group.items as item (item.id)}
							<li class="group__item">
								<span class="group__label">{item.label}</span>
								{#if group.money && item.amount_cents !== null}
									<span class="group__amount" class:group__amount--income={item.kind === 'income'}>
										{item.kind === 'income' ? '+' : '−'}{formatAmount(item.amount_cents)}
									</span>
								{/if}
							</li>
						{/each}
					</ul>
				</section>
			{/each}
			<p class="sheet__warning">{m.msg_delete_cannot_undo()}</p>
		{/if}
		{#if error}<p class="sheet__error" role="alert">{error}</p>{/if}
		<div class="sheet__actions">
			<Button variant="text" type="button" onclick={() => (open = false)} disabled={deleting}>{m.common_cancel()}</Button>
			<Button variant="filled" type="button" class="sheet__danger" onclick={confirm} disabled={deleting || !preview}>
				{deleting ? m.common_deleting() : groups.length > 0 ? m.msg_delete_all() : m.common_delete()}
			</Button>
		</div>
	</div>
</BottomSheet>

<style>
	.sheet {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.sheet__title {
		margin: 0;
		font-family: var(--md-sys-typescale-title-large-font);
		font-size: var(--md-sys-typescale-title-large-size);
		color: var(--md-sys-color-on-surface);
	}
	.sheet__lede {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.group {
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		overflow: hidden;
	}
	.group__title {
		margin: 0;
		padding: 8px 12px;
		background: var(--md-sys-color-surface-container);
		font-size: 0.75rem;
		font-weight: 700;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--md-sys-color-on-surface-variant);
	}
	.group__list {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.group__item {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		padding: 8px 12px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface);
	}
	.group__item:first-child {
		border-top: none;
	}
	.group__label {
		min-width: 0;
		overflow-wrap: anywhere;
	}
	.group__amount {
		flex: none;
		font-variant-numeric: tabular-nums;
		font-weight: 600;
	}
	.group__amount--income {
		color: var(--md-sys-color-primary);
	}
	.sheet__warning {
		margin: 0;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--md-sys-color-error);
	}
	.sheet__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.sheet__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
		margin-top: 8px;
	}
	.sheet__actions :global(.sheet__danger) {
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
	}
</style>
