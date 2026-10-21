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
	import { onMount, type Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { formatBytes } from '$lib/attachments';
	import { touchScreen } from '$lib/touch';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import SendButton from '$lib/components/m3/SendButton.svelte';
	import MicButton from '$lib/components/MicButton.svelte';
	import AttachmentIcon from '$lib/components/AttachmentIcon.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import EffortMenu from './EffortMenu.svelte';

	// The chat's message bar: one rounded box with the message on top and a toolbar under it
	// ([+] · model · thinking on the left, dictation and Send on the right). It holds no form of
	// its own: Send (and Enter with a keyboard) calls `onsubmit`, and the page posts the message
	// its own way (ChatThread submits its form).

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
		onsubmit,
		onfiles,
		onvoicenote,
		recording = false,
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
		/** More to show with the chips: the voice-note recorder while it's open. */
		tray?: Snippet;
		onsubmit: (payload: ComposerSubmit) => void;
		/** Files picked in the file picker or pasted into the field. */
		onfiles?: (files: File[]) => void;
		/** Starts a voice note; without it the [+] menu doesn't offer one. */
		onvoicenote?: () => void;
		/** A voice note is being recorded: no second one. */
		recording?: boolean;
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
	let actionsOpen = $state(false);
	// The pill shows a pick straight away; the page may still put it back (a failed save).
	let level = $derived(thinkingLevel);

	// The file shortcut follows the platform: ⌘U on Apple keyboards, Ctrl U elsewhere.
	let mac = $state(false);
	onMount(() => {
		const nav = navigator as Navigator & { userAgentData?: { platform?: string } };
		mac = /mac|iphone|ipad|ipod/i.test(nav.userAgentData?.platform || nav.platform || '');
	});

	const uploading = $derived(attachments.some((file) => !file.uploaded));
	const canSend = $derived((value.trim().length > 0 || attachments.length > 0) && !uploading && !disabled);

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

	function onKeydown(event: KeyboardEvent) {
		// An input method (Japanese, Chinese, …) uses Enter to accept a word, not to send.
		if (event.isComposing || event.keyCode === 229) return;
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
	<label for="{uid}-message" class="sr-only">{label}</label>
	<textarea
		id="{uid}-message"
		bind:this={textarea}
		bind:value
		rows="1"
		{placeholder}
		{disabled}
		class="cci__input"
		use:autoGrow={value}
		onkeydown={onKeydown}
		onpaste={onPaste}
		onfocus={() => (dictationBase = value)}
	></textarea>

	{#if attachments.length > 0 || attachError || tray}
		<div class="cci__files">
			{#each attachments as file (file.key)}
				<span class="file-chip" class:file-chip--voice={file.kind === 'voice'} class:file-chip--busy={!file.uploaded}>
					{#if file.thumb}
						<img class="file-chip__thumb" src={file.thumb} alt="" />
					{:else}
						<AttachmentIcon kind={file.kind} size={16} />
					{/if}
					<span class="file-chip__name">{file.name}</span>
					<span class="file-chip__size">
						{#if file.uploaded}
							{formatBytes(file.size)}
						{:else}
							<span class="sr-only">{m.chat_uploading()}</span>{Math.round(file.progress * 100)}%
						{/if}
					</span>
					{#if !file.uploaded}
						<span class="file-chip__progress" style:--progress={file.progress} aria-hidden="true"></span>
					{/if}
					<button type="button" class="file-chip__remove" aria-label={m.chat_remove_file({ name: file.name })} onclick={() => onremoveattachment?.(file.key)}>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
					</button>
				</span>
			{/each}
			{@render tray?.()}
			{#if attachError}
				<p class="cci__error" role="alert">{attachError}</p>
			{/if}
		</div>
	{/if}

	<div class="cci__toolbar" data-composer-toolbar>
		<div class="cci__start">
			<Menu bind:open={actionsOpen} align="start">
				{#snippet trigger({ toggle })}
					<button
						type="button"
						class="cci__plus"
						aria-label={m.chat_attach_title()}
						aria-haspopup="menu"
						aria-expanded={actionsOpen}
						title={m.chat_attach_title()}
						{disabled}
						onclick={toggle}
					>
						<IconPlus size={20} />
					</button>
				{/snippet}
				<MenuItem
					type="button"
					title={m.chat_attach_files_hint()}
					onclick={() => {
						actionsOpen = false;
						pickFiles();
					}}
				>
					<span data-label>{m.composer_images_files()}</span>
					<kbd class="cci__key">{mac ? '⌘U' : 'Ctrl U'}</kbd>
				</MenuItem>
				{#if onvoicenote}
					<MenuItem
						type="button"
						title={m.chat_record_voice_hint()}
						disabled={recording}
						onclick={() => {
							actionsOpen = false;
							onvoicenote();
						}}
					>
						<span data-label>{m.chat_record_voice()}</span>
					</MenuItem>
				{/if}
			</Menu>
			<input bind:this={fileInput} type="file" multiple hidden onchange={onPicked} />
			{@render modelControl?.()}
			{#if level}
				<EffortMenu {level} onchange={pickLevel} {disabled} />
			{/if}
		</div>
		<div class="cci__end">
			<MicButton ontranscript={onTranscript} {disabled} />
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
	.cci__key {
		flex: none;
		padding: 1px 6px;
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		white-space: nowrap;
	}

	.cci__files {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		padding: 2px 4px 6px;
	}
	.cci__error {
		flex-basis: 100%;
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-error);
	}
	.file-chip {
		position: relative;
		overflow: hidden;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		max-width: 100%;
		height: 36px;
		padding: 0 4px 0 10px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
		font-size: 0.8125rem;
	}
	.file-chip--voice {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.file-chip--busy .file-chip__name {
		opacity: 0.75;
	}
	.file-chip__thumb {
		flex: none;
		width: 26px;
		height: 26px;
		margin-left: -4px;
		border-radius: var(--md-sys-shape-corner-small);
		object-fit: cover;
	}
	/* Upload progress: a bar along the chip's bottom edge. */
	.file-chip__progress {
		position: absolute;
		left: 0;
		bottom: 0;
		height: 3px;
		width: calc(var(--progress) * 100%);
		background: var(--md-sys-color-primary);
		transition: width 160ms linear;
	}
	.file-chip__name {
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 220px;
	}
	.file-chip__size {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.file-chip__remove {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: inherit;
		cursor: pointer;
	}
	.file-chip__remove:hover {
		background: var(--md-sys-color-surface-container-highest);
	}
	.file-chip__remove:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	/* A 44px target on touch: the chip grows to hold it. */
	@media (pointer: coarse) {
		.file-chip {
			height: 44px;
			padding-right: 0;
		}
		.file-chip__remove {
			width: 44px;
			height: 44px;
		}
	}
</style>
