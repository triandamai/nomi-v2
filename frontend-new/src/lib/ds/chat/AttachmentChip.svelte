<script lang="ts">
	import { FileText, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Attachment / Uploading and Ready, in the composer before you send. Uploading
	// shows the real progress (bytes in, never a guess); ready shows the type and size.
	type Props = {
		name: string;
		/** e.g. "62% · 3,1 of 5 MB" while uploading, "PDF · 4,2 MB" when ready. */
		detail: string;
		/** 0 to 1 while uploading; absent once ready. */
		progress?: number;
		onremove?: () => void;
	};
	let { name, detail, progress, onremove }: Props = $props();

	const uploading = $derived(progress !== undefined);
	const percent = $derived(Math.round(Math.min(1, Math.max(0, progress ?? 0)) * 100));
</script>

<span class={['flex max-w-full items-center gap-2.5 rounded-sm border border-border-strong bg-surface py-1.5 pr-2 pl-2.5', uploading && 'w-62.5']}>
	<span class="inline-flex shrink-0 text-text-2"><Icon icon={FileText} /></span>
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="truncate font-mono text-caption text-text">{name}</span>
		{#if uploading}
			<span role="progressbar" aria-label={name} aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent} class="my-0.5 h-0.75 w-full overflow-hidden rounded-xs bg-surface-2">
				<span class="block h-full rounded-xs bg-accent" style="width: {percent}%"></span>
			</span>
		{/if}
		<span class="truncate font-mono text-label text-text-3">{detail}</span>
	</span>
	{#if onremove}
		<button
			type="button"
			aria-label={uploading ? m.attach_cancel({ name }) : m.attach_remove({ name })}
			class="inline-flex shrink-0 cursor-pointer rounded-xs p-0.5 text-text-2 outline-offset-1 hover:text-text pointer-coarse:size-11 pointer-coarse:items-center pointer-coarse:justify-center"
			onclick={onremove}><Icon icon={X} size={14} /></button
		>
	{/if}
</span>
