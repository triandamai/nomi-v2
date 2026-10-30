<script lang="ts">
	import { X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Attachment / Image: a picture attached in the composer, as a 44px square.
	// Its remove button shows on hover or focus (always on touch screens).
	type Props = { src: string; alt: string; onremove?: () => void };
	let { src, alt, onremove }: Props = $props();
</script>

<span class="group relative inline-flex size-11 shrink-0">
	<img {src} {alt} class="size-11 rounded-sm bg-surface-2 object-cover outline -outline-offset-1 outline-border-strong" />
	{#if onremove}
		<button
			type="button"
			aria-label={m.attach_remove({ name: alt })}
			class="absolute -top-1.5 -right-1.5 inline-flex size-5 cursor-pointer items-center justify-center rounded-full border border-border-strong bg-surface text-text-2 opacity-0 outline-offset-1 group-focus-within:opacity-100 group-hover:opacity-100 hover:text-text pointer-coarse:opacity-100"
			onclick={onremove}><Icon icon={X} size={12} /></button
		>
	{/if}
</span>
