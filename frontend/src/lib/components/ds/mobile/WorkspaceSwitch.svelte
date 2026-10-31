<script lang="ts">
	import { Eye, MessageSquare } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Workspace Switch: on a phone the chat and the preview share one screen, switched
	// here. A radio group (arrow keys move it); the preview counts changes waiting to be seen.
	type View = 'chat' | 'preview';
	type Props = { value?: View; previewCount?: number; onchange?: (value: View) => void };
	let { value = $bindable('chat'), previewCount, onchange }: Props = $props();

	const views = [
		{ value: 'chat', icon: MessageSquare, label: () => m.ws_chat() },
		{ value: 'preview', icon: Eye, label: () => m.ws_preview() },
	] as const;
	let buttons: HTMLButtonElement[] = $state([]);

	function choose(next: View, focus = false) {
		value = next;
		onchange?.(next);
		if (focus) buttons[views.findIndex((view) => view.value === next)]?.focus();
	}

	function onkeydown(event: KeyboardEvent) {
		if (!['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].includes(event.key)) return;
		event.preventDefault();
		choose(value === 'chat' ? 'preview' : 'chat', true);
	}
</script>

<div class="w-full bg-bg px-4 pt-1.5 pb-2.5">
	<div role="radiogroup" aria-label={m.ws_switch()} tabindex="-1" {onkeydown} class="flex h-11 gap-0.5 rounded-md bg-surface-2 p-0.5">
		{#each views as view, index (view.value)}
			{@const checked = view.value === value}
			<button
				bind:this={buttons[index]}
				type="button"
				role="radio"
				aria-checked={checked}
				tabindex={checked ? 0 : -1}
				onclick={() => choose(view.value)}
				class={[
					'inline-flex flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-inner text-phone-sm -outline-offset-2',
					checked ? 'bg-bg font-medium text-text' : 'text-text-2',
				]}
			>
				<Icon icon={view.icon} size={15} />{view.label()}
				{#if view.value === 'preview' && previewCount}
					<span class="inline-flex h-4.5 items-center rounded-full bg-accent px-1.5 text-label font-semibold text-accent-ink">{previewCount}</span>
				{/if}
			</button>
		{/each}
	</div>
</div>
