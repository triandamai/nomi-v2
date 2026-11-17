<script lang="ts">
	import type { Snippet } from 'svelte';
	import { CircleAlert, Download, FileText, LoaderCircle, RotateCcw, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';

	// Chat responses → Chat / File Viewer (G534E), Loading (HVM9y) and Error (K1phVf); on a phone
	// Mobile / File Viewer (GbEIf), Loading (b3lX47) and Error (nQ4Iq): a file of the chat, opened
	// beside it. On a desktop the 440 panel on the bg with its left stroke: the header (the file's
	// icon, name in mono over its meta, Download, Close), then the page on surface-2 padded 16. On a
	// phone the whole screen: Close first in the 56 top bar, padded 12, swipe down to go back.
	// - Loading: the real step and the time it's taken ("Opening page 1 of 12 · 0:02") over a
	//   page-shaped skeleton, and the footer's note.
	// - Error: what happened, the code in mono, Try again and Download.
	// - The cite bar and the highlight footer (G534E) come with citations, which the backend doesn't
	//   give yet (BACKEND_TODO.md): opened from a file it shows the page alone, as the contract asks.
	// Focus goes to Close as it opens; Escape closes; the page gives focus back to what opened it.
	type Props = {
		name: string;
		/** "PDF · 12 pages · your attachment". */
		meta: string;
		status: 'loading' | 'ready' | 'error';
		/** While loading: "Opening page 1 of 12", and how long it's taken, "0:02". */
		step?: string;
		elapsed?: string;
		/** Under the skeleton while loading. */
		note?: string;
		error?: { body: string; code: string };
		phone?: boolean;
		downloadHref: string;
		onretry?: () => void;
		onclose: () => void;
		/** The page itself, once it's there. */
		children?: Snippet;
	};
	let { name, meta, status, step, elapsed, note, error, phone = false, downloadHref, onretry, onclose, children }: Props = $props();

	let close: HTMLButtonElement | undefined = $state();
	$effect(() => close?.focus());

	function onkeydown(event: KeyboardEvent) {
		if (event.key !== 'Escape' || event.defaultPrevented) return;
		event.preventDefault();
		onclose();
	}

	// A phone's swipe down closes it, as the pen's note says.
	let startY: number | null = null;
	const ontouchstart = (event: TouchEvent) => (startY = event.touches[0]?.clientY ?? null);
	function ontouchend(event: TouchEvent) {
		const endY = event.changedTouches[0]?.clientY;
		if (phone && startY !== null && endY !== undefined && endY - startY > 80) onclose();
		startY = null;
	}
</script>

<svelte:window {onkeydown} />

<section
	role="dialog"
	aria-label={name}
	class={['flex flex-col bg-bg text-text', phone ? 'fixed inset-0 z-40 h-dvh w-full' : 'h-full w-110 shrink-0 border-l border-border']}
	{ontouchstart}
	{ontouchend}
>
	{#if phone}
		<header class="flex h-14 shrink-0 items-center gap-3 border-b border-border pr-3 pl-4">
			<button bind:this={close} type="button" aria-label={m.action_close()} class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-surface-2 text-text-2 outline-offset-1 pointer-coarse:size-11" onclick={onclose}><Icon icon={X} /></button>
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate font-mono text-ui text-text">{name}</span>
				<span class="truncate font-mono text-label text-text-3">{meta}</span>
			</span>
			<a href={downloadHref} download aria-label={m.files_download_name({ name })} class="inline-flex size-7.5 shrink-0 items-center justify-center rounded-sm text-text-3 outline-offset-1 pointer-coarse:size-11"><Icon icon={Download} /></a>
		</header>
	{:else}
		<header class="flex h-13 shrink-0 items-center gap-2.5 border-b border-border pr-2.5 pl-4">
			<span class="inline-flex shrink-0 text-text-2"><Icon icon={FileText} size={14} /></span>
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate font-mono text-caption text-text">{name}</span>
				<span class="truncate font-mono text-label text-text-3">{meta}</span>
			</span>
			<a href={downloadHref} download aria-label={m.files_download_name({ name })} class="inline-flex size-7.5 shrink-0 items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text"><Icon icon={Download} /></a>
			<button bind:this={close} type="button" aria-label={m.action_close()} class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text" onclick={onclose}><Icon icon={X} /></button>
		</header>
	{/if}

	{#if status === 'error' && error}
		<div class={['flex min-h-0 flex-1 flex-col justify-center gap-3', phone ? 'p-6' : 'p-8']}>
			<span class="inline-flex text-danger"><Icon icon={CircleAlert} size={phone ? 20 : 18} /></span>
			<p class={phone ? 'text-phone-title font-semibold text-text' : 'text-ui font-medium text-text'}>{m.files_viewer_error_title()}</p>
			<p class={phone ? 'text-body text-text-2' : 'text-caption leading-normal text-text-2'}>{error.body}</p>
			<p class="font-mono text-label text-text-3">{error.code}</p>
			{#if phone}
				<span class="flex flex-col gap-2 pt-2">
					{#if onretry}<button type="button" class="flex h-11.5 cursor-pointer items-center justify-center gap-2 rounded-lg bg-accent text-phone font-medium text-accent-ink outline-offset-2" onclick={onretry}><Icon icon={RotateCcw} />{m.files_try_again()}</button>{/if}
					<a href={downloadHref} download class="flex h-11.5 items-center justify-center gap-2 rounded-lg border border-border-strong text-phone font-medium text-text outline-offset-2"><Icon icon={Download} />{m.files_download()}</a>
				</span>
			{:else}
				<span class="flex gap-2 pt-1">
					{#if onretry}<Button label={m.files_try_again()} variant="secondary" iconStart={RotateCcw} onclick={onretry} />{/if}
					<Button label={m.files_download()} variant="ghost" iconStart={Download} href={downloadHref} />
				</span>
			{/if}
		</div>
	{:else}
		{#if status === 'loading'}
			<p role="status" class="flex h-10.5 shrink-0 items-center gap-2 border-b border-border bg-surface px-4">
				<span class="inline-flex text-text-2 motion-safe:animate-spin"><Icon icon={LoaderCircle} size={12} /></span>
				<span class="font-mono text-label text-text-2">{step}</span>
				{#if elapsed}<span class="font-mono text-label text-text-3">· {elapsed}</span>{/if}
			</p>
		{/if}
		<div class={['flex min-h-0 flex-1 flex-col bg-surface-2', phone ? 'p-3' : 'p-4']}>
			{#if status === 'loading'}
				<div aria-hidden="true" class={['flex flex-1 flex-col rounded-sm bg-surface outline -outline-offset-1 outline-border', phone ? 'gap-3.5 p-4' : 'gap-3 p-5']}>
					<span class="h-2.5 w-40 rounded-xs bg-surface-2"></span>
					<span class="h-2.5 w-full rounded-xs bg-surface-2"></span>
					{#each ['w-55', 'w-full', 'w-45', 'w-full', 'w-full', 'w-50', 'w-full', 'w-35', 'w-full', 'w-full'] as width, index (index)}
						<span class={['h-2 rounded-xs bg-surface-2', width]}></span>
					{/each}
				</div>
			{:else}
				{@render children?.()}
			{/if}
		</div>
		{#if status === 'loading' && note}
			<p class="shrink-0 border-t border-border bg-surface py-2 pr-2.5 pl-4 font-mono text-label text-text-3">{note}</p>
		{/if}
	{/if}
</section>
