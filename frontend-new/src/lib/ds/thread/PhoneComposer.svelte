<script lang="ts">
	import { ArrowUp, AtSign, ChevronDown, Cpu, Mic, Plus, Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import MentionPicker from '../chat/MentionPicker.svelte';
	import { nextOption, type MentionOption } from '../chat/mention';

	// Flow / Mobile Chat — Start & answer → Composer (steps 2–7): the phone's composer, on the ground
	// over a top stroke, [8,12]. Attach on the left (38, surface-2), the message in a round pill
	// (border-strong, [9,10,9,14], kept while typing: Mention picker vfYWh) at 16pt so iOS doesn't
	// zoom, the mic inside it, and Send (mint) only once there's text; while a reply streams, Stop takes Send's place. While the person writes,
	// the toolbar shows under it: the model chip (mono) and "@ Crew", each opening the page's sheet.
	// "@" opens the crew picker above the composer (step 3), the desktop's MentionPicker: it keeps
	// its 300 width where the pen runs it the screen's width (it's the old app's twin).
	// It follows the twin Mobile / Composer and adds the picker, the toolbar and a guard: Stop comes
	// in under the finger that just sent, so a double tap on Send doesn't also stop the reply.
	type Props = {
		value?: string;
		placeholder: string;
		streaming?: boolean;
		crew?: MentionOption[];
		/** The model the next message goes on (the toolbar's chip). */
		model?: string;
		field?: HTMLTextAreaElement;
		onsend: (text: string) => void;
		onstop?: () => void;
		onattach?: () => void;
		onrecord?: () => void;
		onmodel?: () => void;
		/** "@ Crew": the page's sheet of who to address. */
		oncrew?: () => void;
	};
	let { value = $bindable(''), placeholder, streaming = false, crew = [], model, field = $bindable(), onsend, onstop, onattach, onrecord, onmodel, oncrew }: Props = $props();

	const id = $props.id();
	const ready = $derived(value.trim() !== '');
	let focused = $state(false);
	const writing = $derived(focused || ready);

	let mention = $state<{ start: number; query: string } | null>(null);
	let active = $state(0);
	const options = $derived(mention ? crew.filter((option) => option.kind === 'crew' && option.name.toLowerCase().startsWith(mention!.query.toLowerCase())) : []);
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

	const SETTLE_MS = 500;
	let sentAt = -Infinity;
	function send() {
		if (!ready) return;
		onsend(value.trim());
		value = '';
		mention = null;
		sentAt = performance.now();
	}
	function stop(event: MouseEvent) {
		if (event.detail > 1 || performance.now() - sentAt < SETTLE_MS) return;
		onstop?.();
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
				mention = null;
				return;
			}
		}
		if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
			event.preventDefault();
			send();
		}
	}
</script>

<div class="relative w-full">
	{#if picking}
		<div class="absolute bottom-full left-3 mb-2">
			<MentionPicker id="{id}-mentions" {options} bind:active onselect={pick} />
		</div>
	{/if}
	<div data-phone-composer class="flex w-full flex-col gap-2 border-t border-border bg-bg px-3 pt-1.75 pb-2">
		<div class="flex items-end gap-2">
			{#if onattach}
				<button type="button" aria-label={m.composer_attach()} class="inline-flex size-9.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-surface-2 text-text-2 outline-offset-1" onclick={onattach}>
					<Icon icon={Plus} size={20} />
				</button>
			{/if}
			<div class="flex min-h-9.5 min-w-0 flex-1 items-end gap-2 rounded-full border border-border-strong bg-surface py-2 pr-2.25 pl-3.25">
				<textarea
					bind:this={field}
					bind:value
					rows="1"
					{placeholder}
					aria-label={m.composer_field()}
					aria-autocomplete="list"
					aria-controls={picking ? `${id}-mentions` : undefined}
					aria-activedescendant={picking ? `${id}-mentions-option-${active}` : undefined}
					{onkeydown}
					oninput={findMention}
					onclick={findMention}
					onfocus={() => (focused = true)}
					onblur={() => {
						focused = false;
						mention = null;
					}}
					class="field-sizing-content max-h-32 min-w-0 flex-1 resize-none bg-transparent text-phone leading-tight text-text outline-none placeholder:text-text-3"
				></textarea>
				{#if onrecord}
					<button type="button" aria-label={m.composer_record()} class="inline-flex shrink-0 cursor-pointer text-text-2 outline-offset-2" onclick={onrecord}><Icon icon={Mic} size={20} /></button>
				{/if}
			</div>
			{#if streaming && onstop}
				<button type="button" aria-label={m.reply_stop()} class="inline-flex size-9.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-text text-bg outline-offset-1" onclick={stop}>
					<Icon icon={Square} size={14} />
				</button>
			{:else if ready}
				<button type="button" aria-label={m.composer_send()} class="inline-flex size-9.5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-accent text-accent-ink outline-offset-1" onclick={send}>
					<Icon icon={ArrowUp} size={20} />
				</button>
			{/if}
		</div>
		{#if writing && (onmodel || oncrew)}
			<div class="flex items-center gap-2 px-0.5">
				{#if onmodel && model}
					<button
						type="button"
						aria-label={m.composer_model({ model })}
						class="inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-full bg-surface-2 px-2.5 text-text-2 outline-offset-1"
						onmousedown={(event) => event.preventDefault()}
						onclick={onmodel}
					>
						<Icon icon={Cpu} size={12} />
						<span class="max-w-40 truncate font-mono text-label">{model}</span>
						<span class="inline-flex text-text-3"><Icon icon={ChevronDown} size={12} /></span>
					</button>
				{/if}
				{#if oncrew}
					<button type="button" class="inline-flex h-7 cursor-pointer items-center gap-1 rounded-full bg-surface-2 px-2.5 text-ui text-text-2 outline-offset-1" onmousedown={(event) => event.preventDefault()} onclick={oncrew}>
						<Icon icon={AtSign} size={13} />{m.composer_crew()}
					</button>
				{/if}
			</div>
		{/if}
	</div>
</div>
