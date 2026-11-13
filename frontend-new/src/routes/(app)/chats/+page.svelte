<script lang="ts">
	import { MessageSquare, SearchX, Trash2 } from '@lucide/svelte';
	import { invalidate } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { chatListGroups, type ChatListRow as Row } from '$lib/adapters/chat';
	import { clock } from '$lib/clock';
	import { callJson } from '$lib/chat/client';
	import { chatHeader } from '$lib/chat/header.svelte';
	import { later } from '$lib/chat/later';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Search from '$lib/ds/inputs/Search.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import ChatListRow from '$lib/ds/thread/ChatListRow.svelte';

	// The Chats page. The pen draws none, so it's built from the design system in the list style of
	// the Inbox (App / Inbox BFoiZ): the page's name and line, a search, then the chats in groups
	// (pinned first, then by the day they last moved), each a ChatListRow with pin and delete. A
	// delete has ten seconds to undo, as in a chat; leaving the app sends it at once (the layout).
	let { data } = $props();

	const time = clock();
	const header = chatHeader();
	const language = getLocale();
	let query = $state('');

	const gone = $derived(new Set(header.leaving));
	const groups = $derived(
		chatListGroups(
			data.sessions.filter((session) => !gone.has(session.id)),
			{ now: time.now, timeZone: time.timeZone, language, query },
		),
	);
	const none = $derived(data.sessions.every((session) => session.project_id || gone.has(session.id)));

	async function pin(row: Row) {
		const result = await callJson(`/chat/${row.id}/api/session`, { method: 'PATCH', body: { pinned: !row.pinned } });
		if (!result.ok) return void toasts.show({ kind: 'error', title: m.chat_change_failed(), detail: result.message || undefined });
		await invalidate('app:shell');
		toasts.show({ kind: 'success', title: row.pinned ? m.chat_unpinned_toast() : m.chat_pinned_toast() });
	}

	function remove(row: Row) {
		const id = row.id;
		header.leaving = [...header.leaving, id];
		later.schedule(
			id,
			() => {
				void fetch(`/chat/${id}/api/session`, { method: 'DELETE', keepalive: true })
					.then((response) => {
						if (!response.ok) throw new Error(String(response.status));
						return invalidate('app:shell');
					})
					.catch(() => toasts.show({ kind: 'error', title: m.chat_delete_failed() }))
					.finally(() => (header.leaving = header.leaving.filter((one) => one !== id)));
			},
			10_000,
		);
		toasts.show({
			kind: 'info',
			icon: Trash2,
			title: m.chat_deleted(),
			duration: 10_000,
			action: {
				label: m.action_undo(),
				onclick: () => {
					if (later.undo(id)) header.leaving = header.leaving.filter((one) => one !== id);
				},
			},
		});
	}
</script>

<svelte:head><title>{m.chats_title()} · {m.app_name()}</title></svelte:head>

<div class="mx-auto flex w-full max-w-180 flex-col gap-6">
	<header class="flex flex-col gap-1.5">
		<h1 class="text-display font-medium tracking-display text-text">{m.chats_title()}</h1>
		<p class="text-body text-text-2">{m.chats_intro()}</p>
	</header>
	{#if none}
		<EmptyState icon={MessageSquare} title={m.chats_empty_title()} body={m.chats_empty_body()} action={{ label: m.nav_new_chat(), href: '/chat/new' }} />
	{:else}
		<Search placeholder={m.chats_search()} bind:value={query} />
		{#if groups.length === 0}
			<EmptyState icon={SearchX} title={m.chats_none_title()} body={m.chats_none_body({ query: query.trim() })} />
		{:else}
			{#each groups as group (group.key)}
				<section class="flex flex-col gap-2" aria-label={group.label}>
					<SectionLabel label={group.label} count={group.rows.length} />
					<div class="flex flex-col gap-0.5">
						{#each group.rows as row (row.id)}
							<ChatListRow href={row.href} title={row.title} preview={row.preview || undefined} time={row.time} pinned={row.pinned} working={row.working} onpin={() => pin(row)} ondelete={() => remove(row)} />
						{/each}
					</div>
				</section>
			{/each}
		{/if}
	{/if}
</div>
