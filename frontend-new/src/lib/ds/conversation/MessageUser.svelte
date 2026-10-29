<script lang="ts">
	import { CircleAlert, Copy, Check, FileText, Hourglass, LoaderCircle, Pencil } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import VersionSwitcher from '../chat/VersionSwitcher.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// What the person said: on the right, in a surface-2 bubble, with any files above it. Until it's
	// sent it's a little faded and says where it is: sending, waiting in the outbox (offline), or
	// not sent with a way to retry or edit it. Once sent, hovering shows copy, edit and the time;
	// an edited message says so and steps between its versions.
	type Props = {
		text: string;
		status?: 'sent' | 'sending' | 'queued' | 'failed';
		/** When it was sent, e.g. "14:02". */
		time?: string;
		edited?: boolean;
		version?: { current: number; total: number; onprev: () => void; onnext: () => void };
		files?: { name: string; detail?: string }[];
		images?: { src: string; alt: string }[];
		oncancel?: () => void;
		onretry?: () => void;
		onedit?: () => void;
		/** Shows the hover actions without a pointer (the gallery). */
		'data-hover'?: boolean;
	};
	let { text, status = 'sent', time, edited = false, version, files = [], images = [], oncancel, onretry, onedit, 'data-hover': hover }: Props = $props();

	const copier = createCopier();
</script>

<div class="group flex w-full flex-col items-end gap-1.5" data-hover={hover || undefined}>
	{#if files.length || images.length}
		<ul aria-label={m.msg_files()} class="flex max-w-full flex-wrap items-end justify-end gap-2">
			{#each files as file (file.name)}
				<li class="flex w-50 max-w-full items-center gap-2.5 rounded-sm border border-border bg-surface px-2.5 py-2">
					<span class="inline-flex size-7 shrink-0 items-center justify-center rounded-xs bg-surface-2 text-text-2"><Icon icon={FileText} size={14} /></span>
					<span class="flex min-w-0 flex-1 flex-col gap-px">
						<span class="truncate font-mono text-caption text-text">{file.name}</span>
						{#if file.detail}<span class="truncate font-mono text-label text-text-3">{file.detail}</span>{/if}
					</span>
				</li>
			{/each}
			{#each images as image (image.src)}
				<li><img src={image.src} alt={image.alt} class="h-13 w-22 rounded-sm border border-border-strong bg-surface-2 object-cover" /></li>
			{/each}
		</ul>
	{/if}
	<p
		class={[
			'max-w-xl rounded-md bg-surface-2 px-3.5 py-2 text-body whitespace-pre-wrap text-text',
			status !== 'sent' && 'opacity-85',
			status === 'failed' && 'border border-danger-border',
		]}
	>
		{text}
	</p>
	{#if status !== 'sent'}
		<p class="flex flex-wrap items-center justify-end gap-1.5">
			{#if status === 'sending'}
				<span class="inline-flex items-center gap-1.5 font-mono text-label text-text-3">
					<span class="inline-flex animate-spin motion-reduce:animate-none"><Icon icon={LoaderCircle} size={12} /></span>{m.msg_sending()}
				</span>
			{:else if status === 'queued'}
				<span class="inline-flex items-center gap-1.5 font-mono text-label text-text-3"><Icon icon={Hourglass} size={12} />{m.msg_queued()}</span>
				{#if oncancel}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={oncancel}>{m.action_cancel()}</button>{/if}
			{:else}
				<span class="inline-flex items-center gap-1.5 font-mono text-label text-danger"><Icon icon={CircleAlert} size={12} />{m.msg_not_sent()}</span>
				{#if onretry}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onretry}>{m.action_retry()}</button>{/if}
				{#if onedit}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onedit}>{m.action_edit()}</button>{/if}
			{/if}
		</p>
	{:else if edited || version}
		<p class="flex items-center gap-2">
			{#if edited && time}<span class="font-mono text-label text-text-3">{m.msg_edited({ time })}</span>{/if}
			{#if version}<VersionSwitcher {...version} />{/if}
		</p>
	{:else if time || onedit}
		<p class="flex items-center gap-0.5 opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100 group-data-hover:opacity-100 pointer-coarse:opacity-100">
			<button
				type="button"
				aria-label={copier.copied ? m.copy_done() : m.msg_copy()}
				class="inline-flex size-6.5 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:size-11"
				onclick={() => copier.copy(text)}><Icon icon={copier.copied ? Check : Copy} size={13} /></button
			>
			{#if onedit}
				<button
					type="button"
					aria-label={m.msg_edit()}
					class="inline-flex size-6.5 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:size-11"
					onclick={onedit}><Icon icon={Pencil} size={13} /></button
				>
			{/if}
			{#if time}<span class="pl-1 font-mono text-label text-text-3">{time}</span>{/if}
		</p>
	{/if}
</div>
