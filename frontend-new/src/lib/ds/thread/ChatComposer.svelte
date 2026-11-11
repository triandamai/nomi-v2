<script lang="ts" module>
	import type { CrewId } from '../agents/crew';

	export type ModelMenuGroup = { group: string; models: { key: string; label: string; source: string; chosen: boolean }[] };
	export type ThinkingChoice = { label: string; value: string; levels: { value: string; label: string }[] };
	export type Recipient = { agent: CrewId; name: string };
</script>

<script lang="ts">
	import { ArrowUp, Check, Cpu, Lightbulb, Mic, Plus, Sparkles, Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import KeyHint from '../actions/KeyHint.svelte';
	import Chip from '../inputs/Chip.svelte';
	import Menu from '../overlays/Menu.svelte';
	import type { MenuEntry } from '../overlays/MenuPanel.svelte';
	import MentionPicker from '../chat/MentionPicker.svelte';
	import StopHint from '../chat/StopHint.svelte';
	import { nextOption, type MentionOption } from '../chat/mention';

	// Flow / Chat — Ask → think → answer (ENb07) and App / Home → Composer (V55yyr): where a message
	// is written. Composed from the design system's own parts (IconButton, Chip, KeyHint, Menu,
	// MentionPicker, StopHint) in the pen's layout: the field over a toolbar, 14 apart in a chat and
	// 28 on Home (`roomy`), padded [14,12,10,12] inside a border-strong stroke (a CSS border, so 1px
	// comes off each side, AGENTS.md §3).
	// - Home's toolbar picks who it goes to (To) and the model, and shows the keys; a chat's picks
	//   the model and the thinking level. A chip with nothing to offer isn't drawn.
	// - "@" opens the crew picker (above the field when the composer sits at the bottom); arrows move
	//   it, Enter or Tab picks, Escape closes. Focus stays in the field.
	// - While the crew answers (`busy`), Send becomes Stop in the same place, and "Stop reply esc"
	//   shows over it on hover or focus (Message controls, step 1). The pen draws Stop with Send's
	//   arrow; it's a square here, which is what stopping looks like everywhere else.
	// - Attach and voice come with attachments (`onattach`, `onrecord`): not drawn without them.
	type Props = {
		value?: string;
		placeholder: string;
		busy?: boolean;
		roomy?: boolean;
		/** Home: who the message goes to (the To chip), from `crew`. */
		to?: Recipient;
		/** The crew to "@", and to pick in the To chip. */
		crew?: MentionOption[];
		/** The chat's model, and what else it can use, grouped by who pays. */
		model?: string;
		models?: ModelMenuGroup[];
		thinking?: ThinkingChoice;
		hints?: boolean;
		pickerAbove?: boolean;
		field?: HTMLTextAreaElement;
		onsend: (text: string) => void;
		onstop?: () => void;
		onto?: (name: string) => void;
		onmodel?: (key: string) => void;
		onthinking?: (value: string) => void;
		onattach?: () => void;
		onrecord?: () => void;
	};
	let {
		value = $bindable(''),
		placeholder,
		busy = false,
		roomy = false,
		to,
		crew = [],
		model,
		models = [],
		thinking,
		hints = false,
		pickerAbove = false,
		field = $bindable(),
		onsend,
		onstop,
		onto,
		onmodel,
		onthinking,
		onattach,
		onrecord,
	}: Props = $props();

	const id = $props.id();
	const canSend = $derived(value.trim().length > 0);

	// The mention being typed: "@da" right before the caret.
	let mention = $state<{ start: number; query: string } | null>(null);
	let active = $state(0);
	const options = $derived(
		mention ? crew.filter((option) => option.kind === 'crew' && option.name.toLowerCase().startsWith(mention!.query.toLowerCase())) : [],
	);
	const picking = $derived(options.length > 0);

	function findMention() {
		if (!field) return;
		const before = value.slice(0, field.selectionStart ?? value.length);
		const match = /(^|\s)@(\w*)$/.exec(before);
		mention = match ? { start: before.length - match[2].length - 1, query: match[2] } : null;
		active = 0;
	}

	function pick(option: MentionOption) {
		if (!mention || !field) return;
		const caret = field.selectionStart ?? value.length;
		const inserted = `@${option.name} `;
		value = value.slice(0, mention.start) + inserted + value.slice(caret);
		const at = mention.start + inserted.length;
		mention = null;
		requestAnimationFrame(() => field?.setSelectionRange(at, at));
	}

	function send() {
		if (!canSend) return;
		onsend(value.trim());
		value = '';
		mention = null;
	}

	function onkeydown(event: KeyboardEvent) {
		if (picking) {
			if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
				event.preventDefault();
				active = nextOption(active, event.key, options.length);
				return;
			}
			if (event.key === 'Enter' || event.key === 'Tab') {
				event.preventDefault();
				pick(options[active]);
				return;
			}
			if (event.key === 'Escape') {
				event.preventDefault();
				event.stopPropagation();
				mention = null;
				return;
			}
		}
		if (event.key !== 'Enter' || event.shiftKey || event.isComposing) return;
		event.preventDefault();
		send();
	}

	/** The field grows with what's written, up to a point, then scrolls. */
	function grow(element: HTMLTextAreaElement) {
		$effect(() => {
			void value;
			element.style.height = 'auto';
			element.style.height = `${Math.min(element.scrollHeight, 240)}px`;
		});
	}

	const toItems = $derived<MenuEntry[]>(
		crew.filter((option) => option.kind === 'crew').map((option) => ({ label: option.name, icon: option.name === to?.name ? Check : undefined, onselect: () => onto?.(option.name) })),
	);
	const modelItems = $derived<MenuEntry[]>(
		models.flatMap((group) => [
			{ group: group.group },
			...group.models.map((option) => ({ label: option.label, icon: option.chosen ? Check : option.source === m.models_nomi_credits() ? Sparkles : Cpu, meta: option.source, onselect: () => onmodel?.(option.key) })),
		]),
	);
	const thinkingItems = $derived<MenuEntry[]>(
		(thinking?.levels ?? []).map((level) => ({ label: level.label, icon: level.value === thinking?.value ? Check : undefined, onselect: () => onthinking?.(level.value) })),
	);
</script>

<div class="relative w-full">
	<div data-composer class={['flex w-full flex-col rounded-md border border-border-strong bg-surface px-2.75 pt-3.25 pb-2.25', roomy ? 'gap-7' : 'gap-3.5']}>
		<textarea
			bind:this={field}
			bind:value
			{placeholder}
			aria-label={m.composer_field()}
			aria-autocomplete="list"
			aria-controls={picking ? `${id}-mentions` : undefined}
			aria-activedescendant={picking ? `${id}-mentions-option-${active}` : undefined}
			rows="1"
			{onkeydown}
			oninput={findMention}
			onclick={findMention}
			onblur={() => (mention = null)}
			use:grow
			class="w-full resize-none bg-transparent px-1 text-body text-text outline-none placeholder:text-text-3 pointer-coarse:text-phone"
		></textarea>
		<div class="flex items-center gap-1.5">
			{#if onattach}<IconButton icon={Plus} label={m.composer_attach()} onclick={onattach} />{/if}
			{#if to && toItems.length > 1}
				<Menu wide items={toItems}>
					{#snippet trigger(props)}<Chip kind="agent" label={to.name} agent={to.agent} expanded={props['aria-expanded']} onclick={props.onclick} />{/snippet}
				</Menu>
			{/if}
			{#if model && modelItems.length > 0}
				<Menu wide items={modelItems}>
					{#snippet trigger(props)}<Chip kind="model" label={model} expanded={props['aria-expanded']} onclick={props.onclick} />{/snippet}
				</Menu>
			{/if}
			{#if thinking && thinkingItems.length > 0}
				<Menu items={thinkingItems} title={m.thinking_title()}>
					{#snippet trigger(props)}<Chip kind="option" icon={Lightbulb} label={thinking.label} expanded={props['aria-expanded']} onclick={props.onclick} />{/snippet}
				</Menu>
			{/if}
			<span class="flex-1"></span>
			{#if hints}<span class="hidden md:block"><KeyHint hints={[{ keys: '↵', action: m.composer_hint_send() }, { keys: '⇧↵', action: m.composer_hint_newline() }]} /></span>{/if}
			{#if onrecord}<IconButton icon={Mic} label={m.composer_record()} variant="ghost" onclick={onrecord} />{/if}
			{#if busy && onstop}
				<span class="group relative inline-flex">
					<span class="pointer-events-none absolute right-0 bottom-full mb-1.5 hidden group-focus-within:inline-flex group-hover:inline-flex"><StopHint {onstop} /></span>
					<button
						type="button"
						aria-label={m.reply_stop()}
						aria-keyshortcuts="Escape"
						class="inline-flex size-7.5 shrink-0 cursor-pointer items-center justify-center rounded-sm bg-text text-bg outline-offset-1 pointer-coarse:size-11"
						onclick={onstop}><Icon icon={Square} size={14} /></button
					>
				</span>
			{:else}
				<IconButton icon={ArrowUp} label={m.composer_send()} variant="send" disabled={!canSend} onclick={send} />
			{/if}
		</div>
	</div>
	{#if picking}
		<div class={['absolute left-0 z-10', pickerAbove ? 'bottom-full mb-2' : 'top-full mt-2']}>
			<MentionPicker id="{id}-mentions" {options} bind:active onselect={pick} />
		</div>
	{/if}
</div>
