<script lang="ts">
	import { CircleAlert, Ellipsis, FileX, TriangleAlert, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import AttachmentChip from '../chat/AttachmentChip.svelte';
	import AttachmentImage from '../chat/AttachmentImage.svelte';

	// Chat responses → Attachment / Failed (BD98f), Rejected (xpYoV) and Model Warning (rrjYM), with
	// Uploading and Ready (the design system's AttachmentChip, a picture's AttachmentImage): one file
	// in the composer, whatever became of it (UX contract audit-chat-files, G3 · G4 · G6).
	// - Failed: the reason in danger and Retry · Remove; on a phone (`phone`) the ⋯ that opens the
	//   file's sheet (Flow / Mobile Chat — Attachments, step 10), since two words don't fit a thumb.
	// - Rejected: why it can't be read, and Remove.
	// - Warning: the picture's 28 thumb and which model can't see it, in the warning colour.
	// Each chip is one stop for the keyboard (its actions inside it): Backspace or Delete on it
	// removes the file, as the contract asks.
	type Props = {
		name: string;
		status: 'uploading' | 'ready' | 'failed' | 'rejected' | 'warning';
		/** Uploading: "42% · 2,1 of 5 MB"; ready: "PDF · 4,2 MB"; otherwise the reason, already written. */
		detail: string;
		/** 0 to 1 while uploading. */
		progress?: number;
		/** A picture's own preview (a ready picture is drawn as its 44 square). */
		thumb?: string;
		phone?: boolean;
		onretry?: () => void;
		onremove?: () => void;
		/** On a phone, a failed file's sheet (Retry, Remove). */
		onmore?: () => void;
	};
	let { name, status, detail, progress, thumb, phone = false, onretry, onremove, onmore }: Props = $props();

	function onkeydown(event: KeyboardEvent) {
		if ((event.key === 'Backspace' || event.key === 'Delete') && onremove) {
			event.preventDefault();
			onremove();
		}
	}
</script>

<span role="group" aria-label={name} tabindex="0" data-file-chip={status} class="inline-flex max-w-full rounded-sm outline-offset-2" {onkeydown}>
	{#if status === 'uploading' || (status === 'ready' && !thumb)}
		<AttachmentChip {name} {detail} progress={status === 'uploading' ? (progress ?? 0) : undefined} {onremove} />
	{:else if status === 'ready' && thumb}
		<AttachmentImage src={thumb} alt={name} {onremove} />
	{:else}
		<span
			class={[
				'flex max-w-full items-center gap-2.5 rounded-sm py-1.5 pr-2 pl-2.5 outline -outline-offset-1',
				status === 'warning' ? 'bg-warning-bg outline-border-strong' : 'bg-danger-bg outline-danger-border',
			]}
		>
			{#if status === 'warning' && thumb}
				<img src={thumb} alt="" class="size-7 shrink-0 rounded-xs object-cover outline -outline-offset-1 outline-border-strong" />
			{:else}
				<span class="inline-flex shrink-0 text-danger"><Icon icon={status === 'rejected' ? FileX : CircleAlert} /></span>
			{/if}
			<span class="flex min-w-0 flex-col gap-px">
				<span class="truncate font-mono text-caption text-text">{name}</span>
				{#if status === 'warning'}
					<span class="flex min-w-0 items-center gap-1 text-warning">
						<Icon icon={TriangleAlert} size={11} />
						<span class="truncate font-mono text-label text-warning">{detail}</span>
					</span>
				{:else}
					<span class="truncate font-mono text-label text-danger">{detail}</span>
				{/if}
			</span>
			{#if status === 'warning'}
				{#if onremove}
					<button type="button" aria-label={m.attach_remove({ name })} class="inline-flex shrink-0 cursor-pointer rounded-xs p-0.5 text-text-2 outline-offset-1 hover:text-text pointer-coarse:size-11 pointer-coarse:items-center pointer-coarse:justify-center" onclick={onremove}>
						<Icon icon={X} size={14} />
					</button>
				{/if}
			{:else if phone && onmore}
				<button type="button" aria-label={m.files_more({ name })} class="inline-flex size-6 shrink-0 cursor-pointer items-center justify-center rounded-xs text-text-2 outline-offset-1 pointer-coarse:size-11" onclick={onmore}>
					<Icon icon={Ellipsis} />
				</button>
			{:else}
				<span class="flex shrink-0 items-center gap-3 pr-0.5 pl-1.5">
					{#if status === 'failed' && onretry}
						<button type="button" aria-label={m.files_retry_name({ name })} class="cursor-pointer rounded-xs text-caption font-medium text-text outline-offset-2 hover:underline pointer-coarse:min-h-11" onclick={onretry}>{m.files_retry()}</button>
					{/if}
					{#if onremove}
						<button type="button" aria-label={m.attach_remove({ name })} class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-2 hover:text-text pointer-coarse:min-h-11" onclick={onremove}>{m.files_remove()}</button>
					{/if}
				</span>
			{/if}
		</span>
	{/if}
</span>
