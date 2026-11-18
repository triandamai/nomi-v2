<script lang="ts">
	import { ArrowUpRight, CornerDownLeft, Cpu, EyeOff, Send, Settings2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';

	// Flow / Chat — Attachments, steps 12–13: "Model Menu / Can see images" (PZvaP) and "Model Menu /
	// No match" (er95B), and on a phone the sheets of Mobile steps 14–15. The usual model menu,
	// filtered to the models that can see a picture: each by who pays, the model in mono over
	// "<who pays> · reads images", the models left out counted (never silently dropped), and Models &
	// providers. With none to offer it says so, links to add one, and keeps Send text only.
	// A menu (role menu): focus starts on the first model, or on "Add one…" when there's none;
	// arrows, Home and End move, Escape closes (the page gives focus back to Switch model).
	export type VisionGroup = { group: string; models: { key: string; label: string; meta: string }[] };
	type Props = {
		groups: VisionGroup[];
		/** "deepseek-v3 hidden · can't see images", already written. */
		hidden?: string;
		settingsHref: string;
		phone?: boolean;
		onpick: (key: string) => void;
		ontextonly: () => void;
		onclose: () => void;
	};
	let { groups, hidden, settingsHref, phone = false, onpick, ontextonly, onclose }: Props = $props();

	const empty = $derived(groups.every((group) => group.models.length === 0));
	let menu: HTMLElement | undefined = $state();

	const items = () => [...(menu?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])];

	// The sheet's dialog puts focus on its Close as it opens; the first model (or "Add one…") takes
	// it after, as on a desktop.
	let sheetBody: HTMLElement | undefined = $state();
	$effect(() => {
		if (!phone) items()[0]?.focus();
		else if (sheetBody) {
			const body = sheetBody;
			requestAnimationFrame(() => body.querySelector<HTMLElement>('a, button')?.focus());
		}
	});

	function onkeydown(event: KeyboardEvent) {
		const list = items();
		const at = list.indexOf(document.activeElement as HTMLElement);
		const move = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: list.length - 1 }[event.key];
		if (move !== undefined) {
			event.preventDefault();
			list[(move + list.length) % list.length]?.focus();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			onclose();
		} else if (event.key === 'Tab') onclose();
	}
</script>

{#if phone}
	<BottomSheet open title={empty ? m.files_none_title() : m.files_switch_model()} subtitle={empty ? m.files_switch_model() : m.files_models_sheet_sub()} {onclose}>
		<div bind:this={sheetBody} class="flex flex-col">
		{#if empty}
			{#if hidden}
				<p class="flex gap-2 border-b border-border px-5 pb-1.75 text-text-3"><span class="inline-flex shrink-0"><Icon icon={EyeOff} size={14} /></span><span class="font-mono text-label">{hidden}</span></p>
			{/if}
			<a href={settingsHref} class="flex h-13 items-center gap-3.5 px-5 text-left -outline-offset-2">
				<span class="inline-flex shrink-0 text-text-2"><Icon icon={Settings2} size={20} /></span>
				<span class="min-w-0 flex-1 truncate text-phone text-text">{m.files_none_add()}</span>
				<span class="inline-flex text-text-3"><Icon icon={ArrowUpRight} /></span>
			</a>
			<span class="px-4 pt-2">
				<button type="button" class="flex h-11.5 w-full cursor-pointer items-center justify-center gap-2 rounded-lg border border-border-strong text-phone font-medium text-text outline-offset-2" onclick={ontextonly}>
					{m.files_send_text_only()}
				</button>
			</span>
		{:else}
			{#each groups as group (group.group)}
				{#each group.models as model (model.key)}
					<button type="button" class="flex h-13 w-full cursor-pointer items-center gap-3.5 px-5 text-left -outline-offset-2" onclick={() => onpick(model.key)}>
						<span class="inline-flex shrink-0 text-text-2"><Icon icon={Cpu} size={20} /></span>
						<span class="min-w-0 flex-1 truncate font-mono text-body text-text">{model.label}</span>
						<span class="font-mono text-label text-text-3">{model.meta}</span>
					</button>
				{/each}
			{/each}
			{#if hidden}
				<p class="flex items-center gap-2 border-t border-border px-5 pt-2.75 pb-1 text-text-3"><span class="inline-flex shrink-0"><Icon icon={EyeOff} size={14} /></span><span class="font-mono text-label">{hidden}</span></p>
			{/if}
		{/if}
		</div>
	</BottomSheet>
{:else if empty}
	<div bind:this={menu} role="menu" tabindex="-1" aria-label={m.files_none_title()} class="flex w-98 max-w-full flex-col overflow-hidden rounded-md bg-surface shadow-float outline -outline-offset-1 outline-border-strong" {onkeydown}>
		<div class="flex flex-col gap-1.5 px-3 pt-3 pb-2.5">
			<p class="text-ui font-medium text-text">{m.files_none_title()}</p>
			{#if hidden}<p class="flex items-center gap-2 text-text-3"><Icon icon={EyeOff} size={12} /><span class="font-mono text-label">{hidden}</span></p>{/if}
		</div>
		<span class="h-px bg-border"></span>
		<a role="menuitem" href={settingsHref} class="flex h-9 items-center gap-2.5 px-3 -outline-offset-2 hover:bg-surface-2 focus:bg-surface-2 focus:outline-none">
			<span class="inline-flex text-text-2"><Icon icon={Settings2} size={14} /></span>
			<span class="min-w-0 flex-1 text-ui text-text">{m.files_none_add()}</span>
			<span class="inline-flex text-text-2"><Icon icon={ArrowUpRight} size={14} /></span>
		</a>
		<div class="flex items-center border-t border-border px-3 pt-1.75 pb-2">
			<button role="menuitem" type="button" class="inline-flex h-7.5 cursor-pointer items-center gap-2 rounded-sm border border-border-strong px-2.75 text-ui text-text outline-offset-2 hover:bg-surface-2 focus:bg-surface-2" onclick={ontextonly}>
				{m.files_send_text_only()}<span class="inline-flex text-text-2"><Icon icon={Send} size={14} /></span>
			</button>
		</div>
	</div>
{:else}
	<div bind:this={menu} role="menu" tabindex="-1" aria-label={m.files_switch_model()} class="flex w-80 max-w-full flex-col gap-0.5 rounded-md bg-surface p-1 shadow-float outline -outline-offset-1 outline-border-strong" {onkeydown}>
		{#each groups as group (group.group)}
			<p class="px-2 pt-1.5 pb-0.5 font-mono text-label tracking-label text-text-3 uppercase">{group.group}</p>
			{#each group.models as model (model.key)}
				<button role="menuitem" type="button" class="group flex h-10 w-full cursor-pointer items-center gap-2.5 rounded-xs px-2 text-left hover:bg-surface-2 focus:bg-surface-2 focus:outline-none" onclick={() => onpick(model.key)}>
					<span class="inline-flex shrink-0 text-text-2"><Icon icon={Cpu} size={14} /></span>
					<span class="flex min-w-0 flex-1 flex-col gap-px">
						<span class="truncate font-mono text-caption text-text">{model.label}</span>
						<span class="truncate font-mono text-label text-text-3">{model.meta}</span>
					</span>
					<span aria-hidden="true" class="hidden shrink-0 text-text-2 group-hover:inline-flex group-focus:inline-flex"><Icon icon={CornerDownLeft} size={14} /></span>
				</button>
			{/each}
		{/each}
		<span class="h-px bg-border"></span>
		{#if hidden}
			<p class="flex h-8 items-center gap-2.5 px-2 text-text-3"><span class="inline-flex shrink-0"><Icon icon={EyeOff} size={14} /></span><span class="truncate font-mono text-label">{hidden}</span></p>
		{/if}
		<a role="menuitem" href={settingsHref} class="flex h-8 items-center gap-2.5 rounded-xs px-2 hover:bg-surface-2 focus:bg-surface-2 focus:outline-none">
			<span class="inline-flex shrink-0 text-text-3"><Icon icon={Settings2} size={14} /></span>
			<span class="text-ui text-text-2">{m.files_models_providers()}</span>
		</a>
	</div>
{/if}
