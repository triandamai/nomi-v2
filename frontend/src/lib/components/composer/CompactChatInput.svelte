<script lang="ts" module>
	import type { ComposerFile } from '$lib/attachments';
	import type { ThinkingLevel } from '$lib/types';

	/** What Send hands the page: the typed text (as typed), the thinking level and the files. */
	export interface ComposerSubmit {
		text: string;
		thinkingLevel: ThinkingLevel | null;
		attachments: ComposerFile[];
	}
</script>

<script lang="ts">
	import { onMount, tick, type Snippet } from 'svelte';
	import { Mic } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { formatBytes, maxFileBytes, maxMessageChars } from '$lib/attachments';
	import { filterMentions, insertMention, mentionAt } from '$lib/mentions';
	import { nextOption, type MentionOption } from '$lib/components/ds/chat/mention';
	import { touchScreen } from '$lib/touch';
	import SendButton from '$lib/components/m3/SendButton.svelte';
	import MicButton from '$lib/components/MicButton.svelte';
	import AttachMenu from '$lib/components/ds/chat/AttachMenu.svelte';
	import AttachmentChip from '$lib/components/ds/chat/AttachmentChip.svelte';
	import AttachmentImage from '$lib/components/ds/chat/AttachmentImage.svelte';
	import MentionPicker from '$lib/components/ds/chat/MentionPicker.svelte';
	import ComposerLimit from '$lib/components/ds/chat/ComposerLimit.svelte';
	import VoiceTranscribed from '$lib/components/ds/chat/VoiceTranscribed.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import EffortMenu from './EffortMenu.svelte';

	// The chat's message bar: one rounded box with the message on top and a toolbar under it
	// ([+] · model · thinking on the left, voice and Send on the right). It holds no form of its
	// own: Send (and Enter with a keyboard) calls `onsubmit`, and the page posts the message its
	// own way (ChatThread submits its form). The design system's pieces do the parts it shares with
	// frontend-new: the attach menu, attachment chips, the "@" picker, the length limit.

	let {
		placeholder = m.composer_placeholder(),
		label = m.chat_message(),
		value = $bindable(''),
		textarea = $bindable(),
		thinkingLevel = null,
		onthinkingchange,
		modelControl,
		attachments = [],
		attachError = null,
		onremoveattachment,
		tray,
		mentionOptions = [],
		transcribed = null,
		onsubmit,
		onfiles,
		onrecord,
		disabled = false,
		busy = false,
		stopForm,
		glass = false,
		dropping = false,
	}: {
		placeholder?: string;
		/** The message field's accessible name. */
		label?: string;
		/** The typed text. */
		value?: string;
		/** The message field, for the page to focus it. */
		textarea?: HTMLTextAreaElement;
		/** The chat's thinking level; null hides the thinking pill. */
		thinkingLevel?: ThinkingLevel | null;
		onthinkingchange?: (level: ThinkingLevel) => void;
		/** The page's model picker, beside the [+]. */
		modelControl?: Snippet;
		/** Files picked for this message, shown as chips under the text. */
		attachments?: ComposerFile[];
		/** Why a file couldn't be added. */
		attachError?: string | null;
		onremoveattachment?: (key: string) => void;
		/** More to show with the chips. */
		tray?: Snippet;
		/** Who (and which files) "@" offers: the crew, then a project's files. */
		mentionOptions?: MentionOption[];
		/** How long the recording was, when the text came from one (e.g. "0:14"). */
		transcribed?: string | null;
		onsubmit: (payload: ComposerSubmit) => void;
		/** Files picked in the file picker or pasted into the field. */
		onfiles?: (files: File[]) => void;
		/** Records a voice note in place of the bar; without it the mic dictates into the field. */
		onrecord?: () => void;
		disabled?: boolean;
		/** Sending, or the crew is working: Send shows progress (and stops the crew with `stopForm`). */
		busy?: boolean;
		/** id of the form that stops the crew (see SendButton). */
		stopForm?: string;
		/** Frosted, over the chat's animated backdrop. */
		glass?: boolean;
		/** Files are being dragged over the page's drop zone. */
		dropping?: boolean;
	} = $props();

	const uid = $props.id();
	let fileInput: HTMLInputElement | undefined = $state();
	let photoInput: HTMLInputElement | undefined = $state();
	let linkField: HTMLInputElement | undefined = $state();
	let linkOpen = $state(false);
	let link = $state('');
	// The pill shows a pick straight away; the page may still put it back (a failed save).
	let level = $derived(thinkingLevel);

	// The file shortcut follows the platform: ⌘U on Apple keyboards, Ctrl U elsewhere.
	let mac = $state(false);
	onMount(() => {
		const nav = navigator as Navigator & { userAgentData?: { platform?: string } };
		mac = /mac|iphone|ipad|ipod/i.test(nav.userAgentData?.platform || nav.platform || '');
	});

	// Counted as the server counts: characters, not UTF-16 units.
	const length = $derived([...value].length);
	const max = maxMessageChars();
	const tooLong = $derived(length > max);
	const number = (n: number) => n.toLocaleString(getLocale());

	const uploading = $derived(attachments.some((file) => !file.uploaded));
	const canSend = $derived((value.trim().length > 0 || attachments.length > 0) && !uploading && !disabled && !tooLong);

	function submit() {
		if (!canSend || busy) return;
		onsubmit({ text: value, thinkingLevel: level, attachments });
	}

	function pickFiles() {
		if (disabled) return;
		fileInput?.click();
	}

	function pickLevel(next: ThinkingLevel) {
		level = next;
		onthinkingchange?.(next);
	}

	function onpick(source: 'file' | 'photo' | 'drive' | 'link') {
		if (source === 'file') pickFiles();
		else if (source === 'photo') photoInput?.click();
		else if (source === 'link') {
			linkOpen = true;
			tick().then(() => linkField?.focus());
		}
	}

	/** The link goes into the message, on a line of its own; the crew reads it from there. */
	function addLink() {
		const url = link.trim();
		if (url) value = value.trim() ? `${value.trimEnd()}\n${url}` : url;
		link = '';
		linkOpen = false;
		textarea?.focus();
	}

	/** Too long for one message: the text goes as a file instead. */
	function sendAsFile() {
		onfiles?.([new File([value], 'message.txt', { type: 'text/plain' })]);
		value = '';
	}

	// "@" names a crew member (or a project file): the picker opens under the "@query" being typed.
	let caret = $state(0);
	let active = $state(0);
	let dismissed = $state<number | null>(null);
	const typing = $derived(mentionOptions.length > 0 ? mentionAt(value, caret) : null);
	const matches = $derived(typing ? filterMentions(mentionOptions, typing.query) : []);
	const picking = $derived(typing !== null && matches.length > 0 && dismissed !== typing.start);
	$effect(() => {
		void typing?.query;
		active = 0;
	});

	function trackCaret() {
		caret = textarea?.selectionStart ?? value.length;
	}

	async function choose(option: MentionOption) {
		if (!typing) return;
		const next = insertMention(value, typing.start, caret, option.name);
		value = next.text;
		caret = next.caret;
		await tick();
		textarea?.setSelectionRange(next.caret, next.caret);
		textarea?.focus();
	}

	function onKeydown(event: KeyboardEvent) {
		// An input method (Japanese, Chinese, …) uses Enter to accept a word, not to send.
		if (event.isComposing || event.keyCode === 229) return;
		if (picking) {
			if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
				event.preventDefault();
				active = nextOption(active, event.key, matches.length);
				return;
			}
			if (event.key === 'Enter' || event.key === 'Tab') {
				event.preventDefault();
				choose(matches[active]);
				return;
			}
			if (event.key === 'Escape') {
				event.preventDefault();
				dismissed = typing?.start ?? null;
				return;
			}
		}
		if (event.key.toLowerCase() === 'u' && (mac ? event.metaKey : event.ctrlKey) && !event.altKey && !event.shiftKey) {
			event.preventDefault();
			pickFiles();
			return;
		}
		// A phone's keyboard has no Shift+Enter: there Enter makes a new line and Send sends.
		if (event.key === 'Enter' && !event.shiftKey && !touchScreen()) {
			event.preventDefault();
			submit();
		}
	}

	function onPicked(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const files = [...(input.files ?? [])];
		input.value = '';
		if (files.length > 0) onfiles?.(files);
	}

	function onPaste(event: ClipboardEvent) {
		const files = [...(event.clipboardData?.files ?? [])];
		if (files.length === 0 || !onfiles) return;
		event.preventDefault();
		onfiles(files);
	}

	// Dictation adds to what was already typed rather than replacing it.
	let dictationBase = '';
	$effect(() => {
		if (!value) dictationBase = '';
	});
	function onTranscript(text: string, final: boolean) {
		value = dictationBase ? `${dictationBase} ${text}` : text;
		if (final) dictationBase = value;
	}

	/** What a chip says under the name: progress while uploading, then the type and size. */
	function detail(file: ComposerFile): string {
		if (!file.uploaded) return `${Math.round(file.progress * 100)}% · ${formatBytes(file.size * file.progress)} / ${formatBytes(file.size)}`;
		const type = file.kind === 'voice' ? m.chat_record_voice() : (file.name.split('.').pop() ?? '').toUpperCase();
		return `${type} · ${formatBytes(file.size)}`;
	}

	// The field grows with what's typed (up to its max height) instead of scrolling inside one
	// line: browsers without `field-sizing` get it measured here.
	function autoGrow(field: HTMLTextAreaElement, _value: string) {
		const fit = () => {
			if (typeof CSS !== 'undefined' && CSS.supports('field-sizing', 'content')) return;
			field.style.height = 'auto';
			field.style.height = `${field.scrollHeight}px`;
		};
		fit();
		return { update: () => requestAnimationFrame(fit) };
	}
</script>

<div class="cci" class:cci--glass={glass} class:cci--drop={dropping} class:cci--disabled={disabled}>
	{#if picking}
		<div class="cci__mentions">
			<MentionPicker id="{uid}-mentions" options={matches} bind:active onselect={choose} />
		</div>
	{/if}
	<label for="{uid}-message" class="sr-only">{label}</label>
	<textarea
		id="{uid}-message"
		bind:this={textarea}
		bind:value
		rows="1"
		{placeholder}
		{disabled}
		class="cci__input"
		role={mentionOptions.length > 0 ? 'combobox' : undefined}
		aria-autocomplete={mentionOptions.length > 0 ? 'list' : undefined}
		aria-expanded={mentionOptions.length > 0 ? picking : undefined}
		aria-controls={picking ? `${uid}-mentions` : undefined}
		aria-activedescendant={picking ? `${uid}-mentions-option-${active}` : undefined}
		use:autoGrow={value}
		onkeydown={onKeydown}
		onkeyup={trackCaret}
		onclick={trackCaret}
		oninput={trackCaret}
		onpaste={onPaste}
		onfocus={() => (dictationBase = value)}
		onblur={() => (dismissed = typing?.start ?? null)}
	></textarea>

	{#if transcribed && !tooLong}
		<div class="cci__note"><VoiceTranscribed duration={transcribed} /></div>
	{/if}
	{#if tooLong}
		<div class="cci__note"><ComposerLimit count={number(length)} max={number(max)} onfile={onfiles ? sendAsFile : undefined} /></div>
	{/if}

	{#if linkOpen}
		<div class="cci__link" role="group" aria-label={m.composer_link_field()}>
			<label for="{uid}-link" class="sr-only">{m.composer_link_field()}</label>
			<input
				id="{uid}-link"
				bind:this={linkField}
				bind:value={link}
				type="url"
				inputmode="url"
				placeholder="https://"
				class="cci__link-input"
				onkeydown={(event) => {
					// Enter adds the link (it would otherwise send the chat's form).
					if (event.key === 'Enter') {
						event.preventDefault();
						addLink();
					} else if (event.key === 'Escape') {
						linkOpen = false;
						textarea?.focus();
					}
				}}
			/>
			<button type="button" class="cci__link-add" disabled={!link.trim()} onclick={addLink}>{m.composer_link_add()}</button>
			<button type="button" class="cci__link-cancel" onclick={() => (linkOpen = false)}>{m.common_cancel()}</button>
		</div>
	{/if}

	{#if attachments.length > 0 || attachError || tray}
		<div class="cci__files">
			{#each attachments as file (file.key)}
				{#if file.thumb && file.uploaded}
					<AttachmentImage src={file.thumb} alt={file.name} onremove={() => onremoveattachment?.(file.key)} />
				{:else}
					<AttachmentChip name={file.name} detail={detail(file)} progress={file.uploaded ? undefined : file.progress} onremove={() => onremoveattachment?.(file.key)} />
				{/if}
			{/each}
			{@render tray?.()}
			{#if attachError}
				<p class="cci__error" role="alert">{attachError}</p>
			{/if}
		</div>
	{/if}

	<div class="cci__toolbar" data-composer-toolbar>
		<div class="cci__start">
			<AttachMenu max={formatBytes(maxFileBytes())} {onpick}>
				{#snippet trigger(props)}
					<button
						type="button"
						class="cci__plus"
						{...props}
						aria-label={m.composer_attach()}
						title="{m.composer_attach()} ({mac ? '⌘U' : 'Ctrl U'})"
						{disabled}
					>
						<IconPlus size={20} />
					</button>
				{/snippet}
			</AttachMenu>
			<input bind:this={fileInput} type="file" multiple hidden onchange={onPicked} />
			<input bind:this={photoInput} type="file" accept="image/*" multiple hidden onchange={onPicked} />
			{@render modelControl?.()}
			{#if level}
				<EffortMenu {level} onchange={pickLevel} {disabled} />
			{/if}
		</div>
		<div class="cci__end">
			{#if onrecord}
				<button type="button" class="cci__mic" aria-label={m.chat_record_voice()} title={m.chat_record_voice()} {disabled} onclick={onrecord}>
					<Mic size={20} aria-hidden="true" />
				</button>
			{:else}
				<MicButton ontranscript={onTranscript} {disabled} />
			{/if}
			<SendButton
				size={44}
				working={busy}
				{stopForm}
				type={busy && stopForm ? 'submit' : 'button'}
				disabled={!canSend}
				onclick={busy ? undefined : submit}
			/>
		</div>
	</div>
</div>

<style>
	/* The bar lays itself out by its own width (a phone, a project's side column), not the
	   viewport's: the model selector and the thinking pill read the same container. */
	.cci {
		position: relative;
		container-type: inline-size;
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 6px 8px 8px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow: 0 18px 40px -28px color-mix(in srgb, var(--md-sys-color-on-surface) 45%, transparent);
		transition: border-color var(--nomi-motion-effects-fast);
	}
	.cci:has(.cci__input:focus) {
		border-color: color-mix(in srgb, var(--md-sys-color-primary) 55%, var(--md-sys-color-outline-variant));
	}
	/* Only the bar blurs what's behind it (a backdrop-filter over the whole thread would re-blur
	   every frame the crew moves). */
	.cci--glass {
		border-color: color-mix(in srgb, var(--md-sys-color-surface-container-lowest) 55%, transparent);
		background: color-mix(in srgb, var(--md-sys-color-surface-container-lowest) 70%, transparent);
		-webkit-backdrop-filter: blur(20px) saturate(1.5);
		backdrop-filter: blur(20px) saturate(1.5);
	}
	@media (prefers-reduced-transparency: reduce) {
		.cci--glass {
			background: var(--md-sys-color-surface-container-lowest);
			-webkit-backdrop-filter: none;
			backdrop-filter: none;
		}
	}
	.cci--drop {
		outline: 2px dashed var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.cci__input {
		display: block;
		width: 100%;
		box-sizing: border-box;
		max-height: 200px;
		overflow-y: auto;
		field-sizing: content;
		margin: 0;
		padding: 8px 8px 6px;
		border: none;
		resize: none;
		outline: none;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		line-height: 1.5;
	}
	.cci__input::placeholder {
		color: var(--md-sys-color-on-surface-variant);
	}
	.cci__input:disabled {
		cursor: not-allowed;
	}
	.cci--disabled .cci__input::placeholder {
		opacity: 0.6;
	}
	/* Bigger screens: a long message grows the box (to a good part of the screen) before scrolling. */
	@media (min-width: 768px) {
		.cci__input {
			max-height: min(40vh, 360px);
		}
	}

	.cci__toolbar {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.cci__start {
		flex: 1 1 auto;
		min-width: 0;
		display: flex;
		align-items: center;
		gap: 4px;
	}
	/* Dictation and Send never shrink or wrap: the model selector gives way instead. */
	.cci__end {
		flex: none;
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.cci__plus {
		flex: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 20px;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.cci__plus:hover:not(:disabled) {
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-high);
	}
	.cci__plus:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.cci__plus:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
	@media (pointer: coarse) {
		.cci__plus {
			width: 44px;
			height: 44px;
			border-radius: 22px;
		}
	}

	.cci__files {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		padding: 2px 4px 6px;
	}
	.cci__error {
		flex-basis: 100%;
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-error);
	}
	.cci__mentions {
		position: absolute;
		left: 8px;
		right: 8px;
		bottom: calc(100% + 6px);
		z-index: 2;
	}
	.cci__note {
		padding: 0 8px 4px;
	}
	.cci__link {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 4px 6px;
	}
	.cci__link-input {
		flex: 1;
		min-width: 0;
		height: 40px;
		padding: 0 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.875rem;
	}
	.cci__link-input:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -1px;
	}
	.cci__link-add,
	.cci__link-cancel {
		flex: none;
		height: 40px;
		padding: 0 12px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-primary);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
	}
	.cci__link-add:hover:not(:disabled),
	.cci__link-cancel:hover {
		background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
	}
	.cci__link-add:focus-visible,
	.cci__link-cancel:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.cci__link-add:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
	.cci__link-cancel {
		color: var(--md-sys-color-on-surface-variant);
	}
	.cci__mic {
		flex: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border: none;
		border-radius: 20px;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.cci__mic:hover:not(:disabled) {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
	}
	.cci__mic:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.cci__mic:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
	@media (pointer: coarse) {
		.cci__link-input,
		.cci__link-add,
		.cci__link-cancel {
			height: 44px;
		}
		.cci__mic {
			width: 44px;
			height: 44px;
			border-radius: 22px;
		}
	}
</style>
