<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { CREW } from '$lib/ds/agents/crew';
	import { crewOf } from '$lib/adapters/shell';
	import PhoneToast from '$lib/ds/mobile/PhoneToast.svelte';
	import { topLayer } from '$lib/ds/overlays/topLayer';
	import { newNeedsYou, type NeedsYou } from './needsYou';

	// On a phone, when a crew member in another chat stops for the person's OK while they're elsewhere
	// in the app (Flow / Mobile Chat — Approvals & background, step 5's interim, UX contract D2 / G2):
	// Mobile / Toast at the top, "{chat}: {Agent} needs your OK" with Open. It shows for 6 s and
	// waits while it has focus. Home lists the same under needs you. Looked for whenever the person's
	// chats move on (`changes`), never on a timer.
	type Props = { changes: number; openChat: string | null; titleOf: (sessionId: string) => string; onopen: (sessionId: string) => void };
	let { changes, openChat, titleOf, onopen }: Props = $props();

	const seen = new Set<string>();
	let shown = $state<NeedsYou | null>(null);
	let focused = $state(false);
	let ready = false;

	async function look(announce: boolean) {
		const response = await fetch('/chat/needs-you').catch(() => null);
		if (!response?.ok) return;
		const items = (await response.json()) as NeedsYou[];
		const fresh = newNeedsYou(seen, items, openChat);
		for (const item of items) seen.add(item.key);
		if (announce && fresh[0]) shown = fresh[0];
	}

	onMount(() => {
		void look(false).then(() => (ready = true));
	});
	$effect(() => {
		void changes;
		untrack(() => ready && void look(true));
	});
	$effect(() => {
		if (!shown || focused) return;
		const timer = setTimeout(() => (shown = null), 6000);
		return () => clearTimeout(timer);
	});

	const agentName = (type: string) => CREW.find((member) => member.id === crewOf(type))?.name ?? type.charAt(0).toUpperCase() + type.slice(1);
</script>

{#if shown}
	{@const item = shown}
	<div class="contents">
		<div {@attach topLayer} popover="manual" class="fixed inset-x-2 top-2 bottom-auto m-0 w-auto overflow-visible border-0 bg-transparent p-0" onfocusin={() => (focused = true)} onfocusout={() => (focused = false)}>
			<PhoneToast
				kind="info"
				title={m.phone_notify_toast({ chat: titleOf(item.sessionId), agent: agentName(item.agent) })}
				action={{
					label: m.notification_open(),
					onclick: () => {
						// Read before clearing: `item` follows `shown`, and is gone once it's null.
						const sessionId = item.sessionId;
						shown = null;
						onopen(sessionId);
					},
				}}
			/>
		</div>
	</div>
{/if}
