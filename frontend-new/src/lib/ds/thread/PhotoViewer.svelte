<script lang="ts">
	import { Download, Image, X, ZoomIn, ZoomOut } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Chat / File Viewer / Image (rCBA7) and Mobile / File Viewer / Image (LiYjQ):
	// a photo of the chat, large. On a desktop a 900 lightbox: the header (the photo's name in mono
	// over its meta, zoom out, the zoom in mono, zoom in, Download, Close), the photo on surface-2
	// padded 32, and the footer saying which photo it is with "esc closes". On a phone the whole
	// screen, Close first, and "swipe down closes". Zoom goes 50% to 200% in quarters.
	// A native modal dialog: focus starts on Close, Escape closes, and the page gives focus back.
	type Props = {
		name: string;
		/** "JPG · 1,8 MB · your attachment". */
		meta: string;
		src: string;
		/** "1 of 1 photo in this message". */
		where: string;
		phone?: boolean;
		downloadHref: string;
		onclose: () => void;
	};
	let { name, meta, src, where, phone = false, downloadHref, onclose }: Props = $props();

	const STEPS = [0.5, 0.75, 1, 1.25, 1.5, 1.75, 2];
	let step = $state(2);
	const zoom = $derived(STEPS[step]);

	function modal(dialog: HTMLDialogElement) {
		dialog.showModal();
	}

	let startY: number | null = null;
	const ontouchstart = (event: TouchEvent) => (startY = event.touches[0]?.clientY ?? null);
	function ontouchend(event: TouchEvent) {
		const endY = event.changedTouches[0]?.clientY;
		if (phone && startY !== null && endY !== undefined && endY - startY > 80) onclose();
		startY = null;
	}
</script>

<dialog
	use:modal
	aria-label={name}
	oncancel={(event) => {
		event.preventDefault();
		onclose();
	}}
	{ontouchstart}
	{ontouchend}
	class={[
		'flex flex-col overflow-hidden bg-bg p-0 text-text backdrop:bg-overlay',
		phone ? 'fixed inset-0 m-0 h-dvh max-h-none w-full max-w-none' : 'm-auto h-190 max-h-dvh w-225 max-w-full rounded-md shadow-float outline -outline-offset-1 outline-border-strong',
	]}
>
	<header class={['flex shrink-0 items-center border-b border-border', phone ? 'h-14 gap-3 pr-3 pl-4' : 'h-13 gap-2.5 pr-2.5 pl-4']}>
		{#if phone}
			<!-- svelte-ignore a11y_autofocus -->
			<button type="button" autofocus aria-label={m.action_close()} class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-surface-2 text-text-2 outline-offset-1 pointer-coarse:size-11" onclick={onclose}><Icon icon={X} /></button>
		{:else}
			<span class="inline-flex shrink-0 text-text-2"><Icon icon={Image} size={14} /></span>
		{/if}
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class={['truncate font-mono text-text', phone ? 'text-ui' : 'text-caption']}>{name}</span>
			<span class="truncate font-mono text-label text-text-3">{meta}</span>
		</span>
		{#if !phone}
			<button type="button" aria-label={m.files_zoom_out()} disabled={step === 0} class="inline-flex size-7.5 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:enabled:bg-surface-2 hover:enabled:text-text disabled:cursor-default" onclick={() => (step = Math.max(0, step - 1))}><Icon icon={ZoomOut} /></button>
			<span aria-live="polite" class="font-mono text-label text-text-2 tabular-nums">{Math.round(zoom * 100)}%</span>
			<button type="button" aria-label={m.files_zoom_in()} disabled={step === STEPS.length - 1} class="inline-flex size-7.5 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:enabled:bg-surface-2 hover:enabled:text-text disabled:cursor-default" onclick={() => (step = Math.min(STEPS.length - 1, step + 1))}><Icon icon={ZoomIn} /></button>
		{/if}
		<a href={downloadHref} download aria-label={m.files_download_name({ name })} class="inline-flex size-7.5 shrink-0 items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:size-11"><Icon icon={Download} /></a>
		{#if !phone}
			<!-- svelte-ignore a11y_autofocus -->
			<button type="button" autofocus aria-label={m.action_close()} class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text" onclick={onclose}><Icon icon={X} /></button>
		{/if}
	</header>
	<div class={['flex min-h-0 flex-1 items-center justify-center overflow-auto bg-surface-2', phone ? 'p-4' : 'p-8']}>
		<img {src} alt={name} style="transform: scale({zoom})" class="max-h-full max-w-full rounded-sm object-contain outline -outline-offset-1 outline-border transition-transform motion-reduce:transition-none" />
	</div>
	<footer class={['flex shrink-0 items-center gap-2 border-t border-border px-4', phone ? 'py-3' : 'py-2.5']}>
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text-2">{where}</span>
		<span class="font-mono text-label text-text-3">{phone ? m.files_swipe_closes() : m.files_esc_closes()}</span>
	</footer>
</dialog>
