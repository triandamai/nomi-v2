<script lang="ts">
	import { TriangleAlert } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Chat — Attachments, step 11 (deDgy, "Model Fix" kthOB) and Mobile step 13 (sE5RV): the
	// line over the composer's toolbar when the model in use can't see a picture that's attached.
	// A warning, not an error: it says the two ways on, Switch model and Send text only. On a desktop
	// one row, 8 apart [0,4], the label in caption; on a phone the line, then the two as pills.
	// `focus` puts focus on Switch model as the line appears (the contract's focus move); its ring is
	// the standard focus ring (XHDLj).
	type Props = {
		phone?: boolean;
		/** Move focus to Switch model when it shows. */
		focus?: boolean;
		/** Ids the line, for Send's aria-describedby. */
		labelId?: string;
		/** The Switch model button, so the menu can give focus back to it. */
		switchButton?: HTMLButtonElement;
		onswitch: () => void;
		ontextonly: () => void;
	};
	let { phone = false, focus = false, labelId, switchButton = $bindable(), onswitch, ontextonly }: Props = $props();

	$effect(() => {
		if (focus) switchButton?.focus();
	});
</script>

{#if phone}
	<div class="flex flex-col gap-2 px-1">
		<p class="flex items-center gap-1.5">
			<span class="inline-flex shrink-0 text-warning"><Icon icon={TriangleAlert} size={12} /></span>
			<span id={labelId} class="text-caption text-text-2">{m.files_fix_phone()}</span>
		</p>
		<span class="flex gap-2">
			<button bind:this={switchButton} type="button" class="inline-flex h-8 cursor-pointer items-center rounded-full border border-border-strong px-3.75 text-phone-sm text-text outline-offset-2 pointer-coarse:h-11" onclick={onswitch}>{m.files_switch_model()}</button>
			<button type="button" class="inline-flex h-8 cursor-pointer items-center rounded-full px-4 text-phone-sm text-text-2 outline-offset-2 pointer-coarse:h-11" onclick={ontextonly}>{m.files_send_text_only()}</button>
		</span>
	</div>
{:else}
	<div class="flex w-full items-center gap-2 px-1">
		<span class="inline-flex shrink-0 text-warning"><Icon icon={TriangleAlert} size={12} /></span>
		<span id={labelId} class="min-w-0 flex-1 text-caption text-text-2">{m.files_fix()}</span>
		<button bind:this={switchButton} type="button" class="shrink-0 cursor-pointer rounded-sm px-1.5 py-0.5 text-caption font-medium text-text outline-offset-0 hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-focus" onclick={onswitch}>{m.files_switch_model()}</button>
		<button type="button" class="shrink-0 cursor-pointer rounded-sm text-caption text-text-2 outline-offset-2 hover:text-text" onclick={ontextonly}>{m.files_send_text_only()}</button>
	</div>
{/if}
