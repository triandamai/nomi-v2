<script lang="ts">
	// Share a reply, or its whole chat, by link (ShareDialog): opening it makes the link (this
	// reply, masked, anyone can view, 7 days); each setting changed saves to the same link.
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import ShareDialog from '$lib/components/ds/chat/ShareDialog.svelte';
	import type { ShareSettings } from '$lib/components/ds/chat/SharePanel.svelte';
	import { CREW } from '$lib/components/ds/agents/crew';
	import { crewId } from '$lib/crewId';
	import { expiryLabel, nextExpiry, plainText, shareUrl, type ExpiryDays } from '$lib/shareLink';

	type Share = { id: string; token: string; preview: string; chat_messages: number };
	let {
		open = $bindable(false),
		messageId,
		agent = null,
		time,
	}: { open?: boolean; messageId: string; agent?: string | null; time: string } = $props();

	let share = $state<Share | null>(null);
	let settings = $state<ShareSettings>({ scope: 'reply', mask: true, anyone: true });
	let expiry = $state<ExpiryDays>(7);
	let failed = $state(false);

	const sessionId = $derived(page.params.sessionId ?? '');
	const who = $derived(crewId(agent));
	const body = () => ({ ...settings, message_id: messageId, expires_in_days: expiry });

	async function save(request: Promise<Response>) {
		const response = await request.catch(() => null);
		failed = !response?.ok;
		if (response?.ok) share = (await response.json()) as Share;
	}

	$effect(() => {
		if (open && !share && sessionId) {
			void save(fetch(`/chat/${sessionId}/shares`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body()) }));
		}
	});

	function update() {
		if (!share) return;
		void save(fetch(`/chat/${sessionId}/shares/${share.id}`, { method: 'PUT', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body()) }));
	}
</script>

{#if open && share}
	<ShareDialog
		bind:open
		preview={{ agent: who, name: agent ?? CREW.find((member) => member.id === who)?.name ?? 'Nomi', time, excerpt: plainText(share.preview) }}
		chatMessages={share.chat_messages}
		url={shareUrl(page.url.origin, share.token)}
		expiry={expiryLabel(expiry)}
		{settings}
		onchange={(next) => {
			settings = next;
			update();
		}}
		onexpiry={() => {
			expiry = nextExpiry(expiry);
			update();
		}}
	/>
{/if}
{#if open && failed}<p class="share-error" role="alert">{m.share_failed()}</p>{/if}

<style>
	.share-error {
		margin: 6px 0 0;
		color: var(--md-sys-color-error);
		font-size: var(--md-sys-typescale-body-small-size);
	}
</style>
