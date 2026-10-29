<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import FileRow from '../conversation/FileRow.svelte';

	// Build → File list: the files of a build as they're written, how many of all of them are done,
	// the one being written now, and the ones still to come (faded).
	type File = { path: string; change?: string; state: 'done' | 'writing' | 'queued' };
	let { files, total }: { files: File[]; total: number } = $props();

	const done = $derived(files.filter((file) => file.state === 'done').length);
</script>

<section class="flex w-full flex-col gap-0.5">
	<header class="flex justify-between px-3 pb-1.5 font-mono text-label text-text-3">
		<span class="tracking-label uppercase">{m.files_title()}</span>
		<span>{m.files_written({ done: String(done), total: String(total) })}</span>
	</header>
	<ul class="flex flex-col gap-0.5">
		{#each files as file (file.path)}
			<li class={file.state === 'queued' ? 'opacity-45' : undefined}><FileRow {...file} /></li>
		{/each}
	</ul>
</section>
