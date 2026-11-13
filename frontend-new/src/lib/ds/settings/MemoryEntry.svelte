<script lang="ts">
	import { Check, Pencil, Trash2 } from '@lucide/svelte';
	import Button from '../actions/Button.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import StrengthBars from './StrengthBars.svelte';

	// One memory inside an open topic (App / Settings / Memory hANTE, built from its rows): what the
	// crew keeps, in plain words, its strength and how often it was used in mono, then correct it
	// or forget it. One the crew asks about says so, with Still right. Correcting turns the words
	// into a field in place: Enter saves, Escape puts it back.
	type Props = {
		content: string;
		strength: { level: 1 | 2 | 3 | 4 | 5; name: string };
		meta: string;
		check?: { question: string; confirm: string };
		labels: { edit: string; forget: string; save: string; cancel: string; field: string };
		busy?: boolean;
		error?: string;
		onsave?: (content: string) => void;
		onforget?: () => void;
		onconfirm?: () => void;
	};
	let { content, strength, meta, check, labels, busy = false, error, onsave, onforget, onconfirm }: Props = $props();

	let editing = $state(false);
	let draft = $state('');
	const id = $props.id();

	function start() {
		draft = content;
		editing = true;
	}
	function onkeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			editing = false;
		} else if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			onsave?.(draft);
		}
	}
	$effect(() => {
		// Saved: the words came back changed, so the field goes.
		void content;
		editing = false;
	});
</script>

<li class="flex flex-col gap-2 border-b border-border py-2.5 pr-3.5 pl-8.5 last:border-b-0">
	{#if editing}
		<label for="{id}-field" class="sr-only">{labels.field}</label>
		<textarea
			id="{id}-field"
			bind:value={draft}
			{onkeydown}
			rows="2"
			aria-invalid={error ? 'true' : undefined}
			class={['w-full resize-none rounded-sm border bg-bg px-2.25 py-1.75 text-caption text-text caret-accent outline-none focus:shadow-focus', error ? 'border-danger' : 'border-accent']}
			{@attach (field) => field.focus()}
		></textarea>
		{#if error}<p class="text-caption text-danger">{error}</p>{/if}
		<div class="flex items-center gap-2">
			<Button label={labels.save} variant="inverse" loading={busy} onclick={() => onsave?.(draft)} />
			<Button label={labels.cancel} variant="ghost" onclick={() => (editing = false)} />
		</div>
	{:else}
		<div class="flex items-start gap-3">
			<p class="min-w-0 flex-1 text-caption text-text">{content}</p>
			<span class="flex shrink-0 items-center gap-0.5">
				<IconButton icon={Pencil} label={labels.edit} variant="ghost" iconSize={14} onclick={start} />
				<IconButton icon={Trash2} label={labels.forget} variant="ghost" iconSize={14} disabled={busy} onclick={onforget} />
			</span>
		</div>
		<div class="flex flex-wrap items-center gap-3">
			<StrengthBars level={strength.level} name={strength.name} />
			<span class="font-mono text-label text-text-3">{meta}</span>
			{#if check}
				<span class="flex items-center gap-2 text-caption text-warning">
					{check.question}
					<Button label={check.confirm} variant="secondary" iconStart={Check} loading={busy} onclick={onconfirm} />
				</span>
			{/if}
		</div>
	{/if}
</li>
