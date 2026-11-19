<script lang="ts">
	import { Eye } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Build — Invite & build together (evgdu) → Follow Bar: on top of a phone screen
	// while you follow someone, in their colour, 40 tall [0,16]: an eye, "Following Sari · Code" at
	// 14 / 500, and Stop in a 16-round shade. The teammate's colour is the page's text on it.
	type Props = { name: string; peer: 1 | 2; where: string; onstop: () => void };
	let { name, peer, where, onstop }: Props = $props();
</script>

<div role="status" class={['flex h-10 w-full items-center gap-2 px-4 text-bg', peer === 1 ? 'bg-peer-1' : 'bg-peer-2']}>
	<Icon icon={Eye} size={16} />
	<span class="min-w-0 flex-1 truncate text-phone-sm font-medium">{m.follow_label({ name, where })}</span>
	<button type="button" aria-label={m.follow_stop()} class="inline-flex h-8 shrink-0 cursor-pointer items-center rounded-2xl bg-shade px-2.5 text-ui font-medium outline-offset-1 pointer-coarse:min-h-11" onclick={onstop}>{m.pb_stop()}</button>
</div>
