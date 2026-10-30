<script lang="ts">
	import { Eye } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Build together → Follow bar: across the top while you follow a teammate, in their colour,
	// saying where they are, with a way to stop.
	type Props = { name: string; peer: 1 | 2; where: string; onstop: () => void };
	let { name, peer, where, onstop }: Props = $props();
</script>

<div role="status" class={['flex h-8 w-full items-center gap-2 px-2.5 text-bg', peer === 1 ? 'bg-peer-1' : 'bg-peer-2']}>
	<Icon icon={Eye} size={14} />
	<span class="min-w-0 flex-1 truncate text-caption font-medium">{m.follow_label({ name, where })}</span>
	<button type="button" class="inline-flex h-5.5 cursor-pointer items-center rounded-xs bg-shade px-2 text-caption outline-offset-1" onclick={onstop}>{m.follow_stop()}</button>
</div>
