<script lang="ts" module>
	export type SentFileItem = {
		name: string;
		/** "PDF · 4,2 MB", or "PDF · private" on a shared link. */
		meta: string;
		/** A picture: its source, drawn as the pen's photo (88 × 52). */
		image?: string;
		/** A file viewers can't open (a shared link): the lock in place of the file's icon. */
		locked?: boolean;
		/** Opens it (the viewer, the photo, or a download); without it the file is only named. */
		onopen?: () => void;
	};
</script>

<script lang="ts">
	import { FileText, Image, Lock } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Chat / User / With Files (ENNIK), and the Shared link's private files (Link D5
	// RCgbo · M5 YWmux): the files a message went with, over its bubble, right-aligned 8 apart. Each
	// file a 200 card [8,10] in the border stroke with its 28 tile (the file's icon, or the lock when
	// viewers can't open it), the name in mono over its meta; a picture its photo. On a phone (`phone`)
	// a column, 6 apart, of 240 cards [10,12] with the 14 corners and a 32 tile.
	type Props = { files: SentFileItem[]; phone?: boolean };
	let { files, phone = false }: Props = $props();
</script>

{#snippet card(file: SentFileItem)}
	<span class={['inline-flex shrink-0 items-center justify-center bg-surface-2 text-text-2', phone ? 'size-8 rounded-inner' : 'size-7 rounded-xs']}>
		<Icon icon={file.locked ? Lock : file.image ? Image : FileText} size={phone ? 16 : 14} />
	</span>
	<span class="flex min-w-0 flex-1 flex-col gap-px text-left">
		<span class={['truncate font-mono text-text', phone ? 'text-ui' : 'text-caption']}>{file.name}</span>
		<span class="truncate font-mono text-label text-text-3">{file.meta}</span>
	</span>
{/snippet}

<ul aria-label={m.files_sent_files()} class={['flex', phone ? 'flex-col items-end gap-1.5' : 'flex-wrap items-end justify-end gap-2']}>
	{#each files as file, index (index)}
		<li class="flex max-w-full">
			{#if file.image && !file.locked}
				{#if file.onopen}
					<button type="button" aria-label={m.files_open_name({ name: file.name })} class="inline-flex cursor-pointer rounded-sm outline-offset-2" onclick={file.onopen}>
						<img src={file.image} alt={file.name} class="h-13 w-22 rounded-sm bg-surface-2 object-cover outline -outline-offset-1 outline-border" />
					</button>
				{:else}
					<img src={file.image} alt={file.name} class="h-13 w-22 rounded-sm bg-surface-2 object-cover outline -outline-offset-1 outline-border" />
				{/if}
			{:else if file.onopen}
				<button type="button" aria-label={m.files_open_name({ name: file.name })} class={['flex max-w-full cursor-pointer items-center gap-2.5 bg-surface outline -outline-offset-1 outline-border hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus', phone ? 'w-60 rounded-xl px-3 py-2.5' : 'w-50 rounded-sm px-2.5 py-2']} onclick={file.onopen}>
					{@render card(file)}
				</button>
			{:else}
				<span class={['flex max-w-full items-center gap-2.5 bg-surface outline -outline-offset-1 outline-border', phone ? 'w-60 rounded-xl px-3 py-2.5' : 'w-50 rounded-sm px-2.5 py-2']}>{@render card(file)}</span>
			{/if}
		</li>
	{/each}
</ul>
