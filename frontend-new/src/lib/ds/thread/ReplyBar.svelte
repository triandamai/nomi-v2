<script lang="ts" module>
	import type { DislikeReason } from '../chat/DislikeFeedback.svelte';

	export type RegenerateStyle = 'again' | 'shorter' | 'longer' | 'list';
	export type Dislike = { reasons: DislikeReason[]; note: string; includeChat: boolean };
</script>

<script lang="ts">
	import { tick } from 'svelte';
	import ResponseActions from '../chat/ResponseActions.svelte';
	import RegenerateMenu from '../chat/RegenerateMenu.svelte';
	import DislikeFeedback from '../chat/DislikeFeedback.svelte';
	import FeedbackSent from '../chat/FeedbackSent.svelte';

	// Flow / Chat — Like & dislike (HKYo1) and Message controls (Cf6GO), steps 6–7: the quiet bar
	// under a finished reply (Response Actions) and what opens from it. Like is one tap and taps off
	// again; dislike opens the reasons under the reply, which close into a thank-you once sent;
	// regenerate opens its menu under its button (try again, shorter, more detail, as a list).
	// "With another model" isn't offered: the API can't regenerate on another model yet
	// (BACKEND_TODO.md). Each callback answers whether the server took it; nothing changes on screen
	// until it has, except the like, which is put back when it fails.
	type Props = {
		/** The reply as plain text, for Copy. */
		text: string;
		meta?: string;
		feedback?: 'liked' | 'disliked' | null;
		onfeedback: (rating: 'up' | 'down' | null) => Promise<boolean>;
		ondislike: (dislike: Dislike) => Promise<boolean>;
		onregenerate?: (style: RegenerateStyle) => void;
		onshare?: () => void;
		ondelete?: () => void;
	};
	let { text, meta, feedback = null, onfeedback, ondislike, onregenerate, onshare, ondelete }: Props = $props();

	const initial = () => feedback;
	let rated = $state(initial());
	let panel = $state<'dislike' | 'sent' | null>(null);
	let sentWithChat = $state(false);
	let regenerating = $state(false);
	let menu: HTMLElement | undefined = $state();

	async function rate(next: 'liked' | 'disliked') {
		const before = rated;
		const value = rated === next ? null : next;
		rated = value;
		const ok = await onfeedback(value === 'liked' ? 'up' : value === 'disliked' ? 'down' : null);
		if (!ok) {
			rated = before;
			return;
		}
		panel = value === 'disliked' ? 'dislike' : null;
	}

	async function sendDislike(dislike: Dislike) {
		if (!(await ondislike(dislike))) return;
		sentWithChat = dislike.includeChat;
		panel = 'sent';
	}

	async function openMenu() {
		regenerating = !regenerating;
		if (!regenerating) return;
		await tick();
		menu?.querySelector<HTMLElement>('[role="menuitem"]')?.focus();
	}

	function pick(choice: { style: RegenerateStyle } | { model: string }) {
		regenerating = false;
		if ('style' in choice) onregenerate?.(choice.style);
	}

	function onkeydown(event: KeyboardEvent) {
		const items = [...(menu?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])];
		const at = items.indexOf(document.activeElement as HTMLElement);
		const next = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: items.length - 1 }[event.key];
		if (next !== undefined) {
			event.preventDefault();
			items[(next + items.length) % items.length]?.focus();
		} else if (event.key === 'Escape' || event.key === 'Tab') {
			regenerating = false;
		}
	}

	$effect(() => {
		if (!regenerating) return;
		const away = (event: PointerEvent) => {
			if (!menu?.parentElement?.contains(event.target as Node)) regenerating = false;
		};
		document.addEventListener('pointerdown', away);
		return () => document.removeEventListener('pointerdown', away);
	});
</script>

<div class="relative flex w-full flex-col gap-3">
	<ResponseActions
		{text}
		{meta}
		feedback={rated}
		onlike={() => rate('liked')}
		ondislike={() => rate('disliked')}
		{onshare}
		onregenerate={onregenerate ? openMenu : undefined}
		{ondelete}
	/>
	{#if regenerating}
		<!-- Under its button: the bar's buttons are 28 wide, 2 apart, Regenerate the fifth. -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div bind:this={menu} class="absolute top-8 left-28 z-10" {onkeydown}>
			<RegenerateMenu inline onpick={pick} />
		</div>
	{/if}
	{#if panel === 'dislike'}
		<DislikeFeedback onsend={sendDislike} oncancel={() => (panel = null)} />
	{:else if panel === 'sent'}
		<FeedbackSent withChat={sentWithChat} />
	{/if}
</div>
