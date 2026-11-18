<script lang="ts">
	import type { Component } from 'svelte';
	import { CircleAlert, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Tag from '../status/Tag.svelte';
	import PhoneButton from '../mobile/PhoneButton.svelte';

	// Flow / Mobile Chat — Errors & recovery (AyrHG): why a turn failed, on a phone. The backend's
	// own sentence at 16 / 1.55 above the pen's Mobile / Error Card: the title on danger-bg, the model
	// (and the person's key) in mono, the fixes as full-width buttons, the main one first. The
	// twin's "{PROVIDER} SAID" quote and its status code are left out until the backend reports them
	// (UX contract G8, BACKEND_TODO.md); `tag` is the plan's, where the pen draws it (credits used up).
	type Action = { label: string; icon?: Component<LucideProps>; onclick?: () => void; href?: string; disabled?: boolean };
	type Props = { title: string; sentence?: string; meta?: string; tag?: string; primary?: Action; secondary?: Action };
	let { title, sentence, meta, tag, primary, secondary }: Props = $props();
</script>

<div class="flex w-full flex-col gap-3">
	{#if sentence}<p class="text-phone leading-phone text-text">{sentence}</p>{/if}
	<section role="alert" aria-label={sentence ? `${title}. ${sentence}` : title} class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong">
		<header class="flex items-center gap-2 border-b border-border bg-danger-bg px-3.5 pt-2.5 pb-2.25">
			<span class="inline-flex text-danger"><Icon icon={CircleAlert} /></span>
			<span class="min-w-0 flex-1 text-body font-semibold text-danger">{title}</span>
			{#if tag}<Tag kind="neutral" label={tag} />{/if}
		</header>
		{#if meta || primary || secondary}
			<div class="flex flex-col gap-3 p-3.5">
				{#if meta}<p class="font-mono text-label break-words text-text-3">{meta}</p>{/if}
				{#if primary || secondary}
					<div class="flex flex-col gap-2">
						{#if primary}<PhoneButton {...primary} />{/if}
						{#if secondary}<PhoneButton variant="secondary" {...secondary} />{/if}
					</div>
				{/if}
			</div>
		{/if}
	</section>
</div>
