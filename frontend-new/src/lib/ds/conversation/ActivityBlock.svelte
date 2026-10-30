<script lang="ts">
	import { Check, CircleAlert, LoaderCircle } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import FileRow from './FileRow.svelte';

	// What an agent did, summed up ("Wrote 3 files · +424") with the real files under it.
	type Props = {
		title: string;
		total?: string;
		state?: 'done' | 'working' | 'failed';
		files: { path: string; change?: string }[];
	};
	let { title, total, state = 'done', files }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md outline -outline-offset-1 outline-border">
	<header class="flex items-center gap-2 border-b border-border bg-surface px-3 pt-2 pb-1.75">
		<span class={['inline-flex', state === 'done' && 'text-accent', state === 'failed' && 'text-danger', state === 'working' && 'animate-spin text-text-2 motion-reduce:animate-none']}>
			<Icon icon={state === 'done' ? Check : state === 'failed' ? CircleAlert : LoaderCircle} size={14} />
		</span>
		<span class="min-w-0 flex-1 truncate text-caption text-text">{title}</span>
		{#if total}<span class="font-mono text-label text-text-2">{total}</span>{/if}
	</header>
	<ul class="flex flex-col">
		{#each files as file (file.path)}<li><FileRow path={file.path} change={file.change} /></li>{/each}
	</ul>
</section>
