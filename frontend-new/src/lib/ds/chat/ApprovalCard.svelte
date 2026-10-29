<script lang="ts">
	import type { Component } from 'svelte';
	import { Check, Paperclip, Pencil, X, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Tag from '../status/Tag.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import CheckMark from '../inputs/CheckMark.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Approval: every send, spend, delete or external write stops here. Pending,
	// it shows exactly what will go out (fields, the message, what's attached) and waits; once you
	// act it collapses to a one-line outcome that stays in the thread.
	type Props = {
		agent: CrewId;
		title: string;
		status?: 'pending' | 'approved' | 'rejected';
		fields?: { key: string; value: string }[];
		body?: string;
		attachment?: string;
		/** e.g. "Don't ask again for Bu Sri"; the choice goes to onapprove. */
		rememberLabel?: string;
		approveLabel?: string;
		approveIcon?: Component<LucideProps>;
		rejectLabel?: string;
		/** After you act: what happened, e.g. "Approved by you · sent 14:06". */
		outcome?: string;
		/** After you act: where to see it (a link) or how to take it back. */
		action?: { label: string; href?: string; onclick?: () => void };
		onapprove?: (choice: { remember: boolean }) => void;
		onreject?: () => void;
		onedit?: () => void;
	};
	let {
		agent,
		title,
		status = 'pending',
		fields = [],
		body,
		attachment,
		rememberLabel,
		approveLabel,
		approveIcon = Check,
		rejectLabel,
		outcome,
		action,
		onapprove,
		onreject,
		onedit,
	}: Props = $props();

	let remember = $state(false);
</script>

<section class={['flex w-full flex-col overflow-hidden rounded-md border bg-bg', status === 'pending' ? 'border-border-strong' : 'border-border']}>
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 py-2.5">
		<Mascot {agent} size={20} />
		<p class={['min-w-0 flex-1 text-ui font-medium', status === 'rejected' ? 'text-text-2' : 'text-text']}>{title}</p>
		{#if status === 'pending'}<Tag kind="neutral" label={m.approval_needs_you()} />{/if}
	</header>
	{#if status === 'pending'}
		<div class="flex flex-col gap-1.5 px-3.5 py-3">
			{#if fields.length}
				<dl class="flex flex-col gap-1.5">
					{#each fields as field (field.key)}
						<div class="flex items-baseline gap-3">
							<dt class="w-14 shrink-0 font-mono text-label text-text-3">{field.key}</dt>
							<dd class="min-w-0 text-caption break-words text-text-2">{field.value}</dd>
						</div>
					{/each}
				</dl>
			{/if}
			{#if body}<p class="border-l border-border-strong py-0.5 pl-3 text-caption leading-relaxed whitespace-pre-line text-text-2">{body}</p>{/if}
			{#if attachment}
				<p class="flex items-center gap-1.5 pt-1 text-text-3">
					<Icon icon={Paperclip} size={12} />
					<span class="font-mono text-label">{attachment}</span>
				</p>
			{/if}
		</div>
		<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 py-2.5">
			<span class="flex min-w-0 flex-1 basis-40 items-center">
				{#if rememberLabel}
					<label class="inline-flex cursor-pointer items-center gap-2 text-caption text-text-3">
						<CheckMark bind:checked={remember} />
						{rememberLabel}
					</label>
				{/if}
			</span>
			<span class="ml-auto flex flex-wrap items-center justify-end gap-2">
				{#if onedit}<Button label={m.approval_edit()} variant="ghost" iconStart={Pencil} onclick={onedit} />{/if}
				{#if onreject}<Button label={rejectLabel ?? m.approval_reject()} variant="secondary" onclick={onreject} />{/if}
				{#if onapprove}<Button label={approveLabel ?? m.approval_approve()} iconEnd={approveIcon} onclick={() => onapprove({ remember })} />{/if}
			</span>
		</footer>
	{:else}
		<p class="flex items-center gap-2 px-3.5 py-2.5">
			<span class={['inline-flex', status === 'approved' ? 'text-accent' : 'text-text-3']}><Icon icon={status === 'approved' ? Check : X} size={14} /></span>
			<span class="min-w-0 flex-1 text-caption text-text-2">{outcome}</span>
			{#if action?.href}
				<a href={action.href} target="_blank" rel="noopener noreferrer" class="rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text hover:underline hover:decoration-border-strong hover:underline-offset-2">{action.label}</a>
			{:else if action}
				<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={action.onclick}>{action.label}</button>
			{/if}
		</p>
	{/if}
</section>
