<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// A quiet line about a message's files, its 12 icon 6 before it in text-3: the note of what was
	// left out (Attachments step 14 JVUHc, "Left Out": mono, `mono`), the composer's "… removed · 1
	// file ready" (Errors D5 yhuBX), and a shared link's "kept the files private" (Link D5 RCgbo,
	// "Files Note": sans; 13 on a phone).
	type Props = { icon?: Component<LucideProps>; text: string; mono?: boolean; phone?: boolean; align?: 'start' | 'end'; id?: string };
	let { icon, text, mono = false, phone = false, align = 'start', id }: Props = $props();
</script>

<p {id} class={['flex items-center gap-1.5 text-text-3', align === 'end' && 'justify-end text-right']}>
	{#if icon}<span class="inline-flex shrink-0"><Icon {icon} size={12} /></span>{/if}
	<span class={[mono ? 'font-mono text-label' : phone ? 'text-ui' : 'text-caption']}>{text}</span>
</p>
