<script lang="ts">
	// A soul.md field where "@" mentions a crew member (@Name) or a tool (@tool_name), the way the
	// chat composer does: the picker opens under the "@query" being typed (crew first, then tools),
	// keys move and pick, Escape closes it. Picking a tool also tells the page (`ontoolmention`) so a
	// member the person can edit gets it in its tools.
	import { tick } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { crewMentionOptions, filterMentions, insertMention, mentionAt, toolMentionOptions, type MentionChoice } from '$lib/mentions';
	import { nextOption } from '$lib/components/ds/chat/mention';
	import SoulMentionPicker from './SoulMentionPicker.svelte';
	import type { CrewRosterMember } from '$lib/crew';
	import type { CatalogTool } from '$lib/types';

	let {
		value = $bindable(''),
		name,
		label,
		rows = 6,
		maxlength,
		crew,
		tools,
		ontoolmention,
		disabled = false,
	}: {
		value: string;
		name: string;
		/** The field's accessible name. */
		label: string;
		rows?: number;
		maxlength?: number;
		/** Who "@" offers first. */
		crew: CrewRosterMember[];
		/** The catalog's tools, offered after the crew. */
		tools: CatalogTool[];
		/** A tool was picked from the "@" list. */
		ontoolmention?: (tool: string) => void;
		disabled?: boolean;
	} = $props();

	const uid = $props.id();
	let textarea: HTMLTextAreaElement | undefined = $state();
	let caret = $state(0);
	let active = $state(0);
	let dismissed = $state<number | null>(null);

	const options = $derived<MentionChoice[]>([...crewMentionOptions(crew), ...toolMentionOptions(tools)]);
	const typing = $derived(options.length > 0 ? mentionAt(value, caret) : null);
	const matches = $derived(typing ? filterMentions(options, typing.query) : []);
	const picking = $derived(typing !== null && matches.length > 0 && dismissed !== typing.start);
	$effect(() => {
		void typing?.query;
		active = 0;
	});

	function trackCaret() {
		caret = textarea?.selectionStart ?? value.length;
	}

	async function choose(option: MentionChoice) {
		if (!typing) return;
		const next = insertMention(value, typing.start, caret, option.name);
		value = next.text;
		caret = next.caret;
		if (option.kind === 'tool') ontoolmention?.(option.name);
		await tick();
		textarea?.setSelectionRange(next.caret, next.caret);
		textarea?.focus();
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.isComposing || event.keyCode === 229 || !picking) return;
		if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
			event.preventDefault();
			active = nextOption(active, event.key, matches.length);
		} else if (event.key === 'Enter' || event.key === 'Tab') {
			event.preventDefault();
			choose(matches[active]);
		} else if (event.key === 'Escape') {
			// Closes the picker, not a dialog or sheet the field sits in.
			event.preventDefault();
			event.stopPropagation();
			dismissed = typing?.start ?? null;
		}
	}
</script>

<div class="soul">
	<div class="soul__box">
	<textarea
		bind:this={textarea}
		bind:value
		{name}
		{rows}
		{maxlength}
		{disabled}
		class="soul__field"
		aria-label={label}
		role={options.length > 0 ? 'combobox' : undefined}
		aria-autocomplete={options.length > 0 ? 'list' : undefined}
		aria-expanded={options.length > 0 ? picking : undefined}
		aria-controls={picking ? `${uid}-mentions` : undefined}
		aria-activedescendant={picking ? `${uid}-mentions-option-${active}` : undefined}
		{onkeydown}
		onkeyup={trackCaret}
		onclick={trackCaret}
		oninput={trackCaret}
		onblur={() => (dismissed = typing?.start ?? null)}
	></textarea>
	{#if picking}
		<div class="soul__mentions">
			<SoulMentionPicker id="{uid}-mentions" options={matches} bind:active onselect={choose} />
		</div>
	{/if}
	</div>
	<p class="soul__hint">{m.toolpicker_soul_hint()}</p>
</div>

<style>
	.soul__box {
		position: relative;
	}
	.soul {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.soul__field {
		box-sizing: border-box;
		width: 100%;
		padding: 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		line-height: 1.5;
		resize: vertical;
	}
	.soul__field:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.soul__mentions {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		z-index: 3;
		max-width: 100%;
	}
	.soul__hint {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
</style>
