<script lang="ts" module>
	/** One version's notes: its HTML from the server's markdown renderer (src/lib/server/markdown.ts). */
	export type ReleaseNote = { version: string; date: string; url?: string; html: string };
</script>

<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import Link from '../actions/Link.svelte';
	import Dialog from '../overlays/Dialog.svelte';
	import Prose from '../thread/Prose.svelte';

	// The account menu's "What's new" (App / Account menu, LkJLt): what changed in the version the
	// app is running, from its GitHub release, in a Dialog. It also opens on its own the first time
	// someone loads a version they haven't seen the notes of. The pen draws the menu row, not the
	// dialog, so this is the wide Dialog with the notes in Prose: the running version's first, then
	// the earlier ones of the same environment under their label. The notes scroll inside the
	// dialog, so its title and Got it stay in view.
	type Props = { open?: boolean; releases: ReleaseNote[]; inline?: boolean };
	let { open = $bindable(false), releases, inline = false }: Props = $props();

	const current = $derived(releases[0]);
	const earlier = $derived(releases.slice(1));
</script>

{#if current}
	<Dialog bind:open {inline} wide title={m.release_notes_title({ version: `v${current.version}` })} description={m.release_notes_body()}>
		<div class="-mx-5 max-h-96 overflow-y-auto px-5">
			{@render release(current)}
			{#if earlier.length > 0}
				<p class="mt-6 mb-3 font-mono text-label tracking-label text-text-3 uppercase">{m.release_notes_earlier()}</p>
				<div class="flex flex-col gap-6">
					{#each earlier as note (note.version)}
						<section class="flex flex-col gap-2 border-t border-border pt-3" aria-label={`v${note.version}`}>
							<p class="font-mono text-label text-text-2">v{note.version}</p>
							{@render release(note)}
						</section>
					{/each}
				</div>
			{/if}
		</div>
		{#snippet actions()}
			<Button label={m.release_notes_done()} variant="inverse" onclick={() => (open = false)} />
		{/snippet}
	</Dialog>
{/if}

{#snippet release(note: ReleaseNote)}
	<div class="flex flex-col gap-2">
		<p class="flex items-center gap-2 font-mono text-label text-text-3">
			<span>{note.date}</span>
			{#if note.url}<Link href={note.url} label={m.release_notes_on_github()} external quiet />{/if}
		</p>
		<Prose html={note.html} />
	</div>
{/snippet}
