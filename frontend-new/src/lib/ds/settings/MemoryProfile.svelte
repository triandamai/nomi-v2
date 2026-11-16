<script lang="ts">
	import { tick } from 'svelte';
	import { Pencil, Trash2 } from '@lucide/svelte';
	import Button from '../actions/Button.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import SectionLabel from '../status/SectionLabel.svelte';

	// App / Settings / Memory (hANTE) → You: the YOU label with the profile's size ("312 / 1,375
	// tok"), then the profile row (surface, border-strong inside, [12,14], 14 apart): the person's
	// initials in a 32 square, "Profile" over the text, when it last changed, and Edit and Clear.
	// Edit opens the text in place, its size counted as it's typed (four characters a token, as the
	// backend counts) and Save refused past the budget. An empty profile says what it's for.
	type Props = {
		labels: { title: string; section: string; edit: string; clear: string; save: string; cancel: string; field: string; empty: string; tooLong: string };
		initials: string;
		text: string;
		/** "updated 21 Oct", or null when it was never written. */
		updated: string | null;
		/** The size against the budget: "312 / 1,375 tok". */
		count: string;
		/** The profile's budget in tokens. */
		budget: number;
		busy?: boolean;
		/** The backend's reason when saving failed. */
		error?: string;
		onsave?: (text: string) => void;
		onclear?: () => void;
	};
	let { labels, initials, text, updated, count, budget, busy = false, error, onsave, onclear }: Props = $props();

	const id = $props.id();
	let editing = $state(false);
	let draft = $state('');
	let field: HTMLTextAreaElement | undefined = $state();
	const tokens = $derived(Math.ceil(draft.trim().length / 4));
	const tooLong = $derived(tokens > budget);
	const typedCount = $derived(`${tokens.toLocaleString('en')} / ${budget.toLocaleString('en')} tok`);

	async function edit() {
		draft = text;
		editing = true;
		await tick();
		field?.focus();
	}
	function save() {
		if (tooLong || busy) return;
		onsave?.(draft.trim());
	}
	// Saved: the text came back changed, so the editor closes.
	$effect(() => {
		void text;
		editing = false;
	});
</script>

<section aria-labelledby="{id}-title" class="flex flex-col gap-2">
	<span id="{id}-title"><SectionLabel label={labels.section} count={editing ? typedCount : count} /></span>
	<div data-profile-row class="flex items-center gap-3.5 rounded-md bg-surface px-3.5 py-3 outline -outline-offset-1 outline-border-strong">
		<span data-avatar aria-hidden="true" class="inline-flex size-8 shrink-0 items-center justify-center self-start rounded-sm bg-surface-2 font-mono text-ui text-text-2 outline -outline-offset-1 outline-border-strong">
			{initials}
		</span>
		{#if editing}
			<form
				class="flex min-w-0 flex-1 flex-col gap-2"
				onsubmit={(event) => {
					event.preventDefault();
					save();
				}}
			>
				<textarea
					bind:this={field}
					bind:value={draft}
					aria-label={labels.field}
					aria-invalid={tooLong || !!error}
					aria-describedby={tooLong || error ? `${id}-error` : undefined}
					rows="3"
					onkeydown={(event) => {
						if (event.key === 'Escape') editing = false;
					}}
					class="w-full resize-y rounded-sm border border-border-strong bg-bg px-2.25 py-1.75 text-ui text-text outline-none focus:border-accent focus:shadow-focus"
				></textarea>
				{#if tooLong || error}<p id="{id}-error" role="alert" class="text-caption text-danger">{tooLong ? labels.tooLong : error}</p>{/if}
				<span class="flex gap-2">
					<Button variant="inverse" type="submit" label={labels.save} loading={busy} disabled={tooLong} />
					<Button variant="ghost" label={labels.cancel} onclick={() => (editing = false)} />
				</span>
			</form>
		{:else}
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="text-ui font-medium text-text">{labels.title}</span>
				<span class={['line-clamp-2 text-ui', text ? 'text-text-2' : 'text-text-3']}>{text || labels.empty}</span>
			</span>
			{#if updated}<span class="shrink-0 font-mono text-label text-text-3 max-sm:hidden">{updated}</span>{/if}
			<span class="flex shrink-0 gap-1">
				<IconButton icon={Pencil} label={labels.edit} variant="ghost" iconSize={14} onclick={edit} />
				{#if text}<IconButton icon={Trash2} label={labels.clear} variant="ghost" iconSize={14} disabled={busy} onclick={() => onclear?.()} />{/if}
			</span>
		{/if}
	</div>
</section>
