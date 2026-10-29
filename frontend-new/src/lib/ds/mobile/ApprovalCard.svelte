<script lang="ts">
	import type { Component } from 'svelte';
	import { ChevronRight, Paperclip, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import PhoneButton from './PhoneButton.svelte';
	import type { CrewId } from '../agents/crew';

	// Mobile / Approval Card: the desktop approval made for a phone. Who wants to do what (opens
	// the full draft), a preview of it, and two full-size buttons.
	type Props = {
		agent: CrewId;
		title: string;
		to?: string;
		subject?: string;
		excerpt?: string;
		attachment?: string;
		approveLabel: string;
		approveIcon?: Component<LucideProps>;
		rejectLabel: string;
		onapprove: () => void;
		onreject: () => void;
		onopen?: () => void;
	};
	let { agent, title, to, subject, excerpt, attachment, approveLabel, approveIcon, rejectLabel, onapprove, onreject, onopen }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg">
	<button type="button" class="flex cursor-pointer items-center gap-2.5 border-b border-border px-3.5 py-3 text-left -outline-offset-2" onclick={onopen} disabled={!onopen}>
		<Mascot {agent} size={20} />
		<span class="min-w-0 flex-1 text-body font-semibold text-text">{title}</span>
		{#if onopen}<span class="inline-flex text-text-3"><Icon icon={ChevronRight} /></span>{/if}
	</button>
	<div class="flex flex-col gap-1.5 p-3.5">
		{#if to}<p class="font-mono text-label break-words text-text-3">to {to}</p>{/if}
		{#if subject}<p class="text-body font-medium text-text">{subject}</p>{/if}
		{#if excerpt}<p class="line-clamp-3 text-phone-sm text-text-2">{excerpt}</p>{/if}
		{#if attachment}
			<p class="flex items-center gap-1.5 pt-1 font-mono text-label text-text-3"><Icon icon={Paperclip} size={12} />{attachment}</p>
		{/if}
	</div>
	<div class="flex gap-2 px-3.5 pb-3.5">
		<PhoneButton label={rejectLabel} variant="secondary" onclick={onreject} />
		<PhoneButton label={approveLabel} icon={approveIcon} onclick={onapprove} />
	</div>
</section>
