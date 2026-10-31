<script lang="ts">
	import { MousePointer2 } from '@lucide/svelte';

	// Build together → Peer cursor: where a teammate's pointer is, with their name, in their colour.
	type Props = { name: string; peer: 1 | 2 };
	let { name, peer }: Props = $props();
</script>

<span aria-hidden="true" class={['inline-flex flex-col', peer === 1 ? 'text-peer-1' : 'text-peer-2']}>
	<MousePointer2 size={18} strokeWidth={1.5} />
	<span class="pl-3"><span class={['inline-flex rounded-full px-2 py-0.5 text-caption font-medium text-bg', peer === 1 ? 'bg-peer-1' : 'bg-peer-2']}>{name}</span></span>
</span>
