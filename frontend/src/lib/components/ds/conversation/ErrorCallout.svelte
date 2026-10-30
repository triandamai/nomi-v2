<script lang="ts">
	import type { Component } from 'svelte';
	import { CircleAlert, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';

	// Design System / Nomi → Conversation: a failure, named plainly, with the provider's own words
	// quoted in mono (code, request id, reason) and the next move one click away: the first action
	// is the main one (inverse), any other is secondary. A link to another site opens in a new tab.
	type Detail = { key: string; value: string; strong?: boolean };
	type Action = { label: string; icon?: Component<LucideProps>; onclick?: () => void; href?: string; external?: boolean };
	type Props = {
		title: string;
		code?: string;
		requestId?: string;
		details?: Detail[];
		actions?: Action[];
	};
	let { title, code, requestId, details = [], actions = [] }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md outline -outline-offset-1 outline-border-strong" role="alert">
	<header class="flex flex-wrap items-center gap-x-2 gap-y-0.5 border-b border-border bg-danger-bg px-3.5 pt-2.5 pb-2.25">
		<span class="inline-flex text-danger"><Icon icon={CircleAlert} size={14} /></span>
		<span class="text-ui font-medium text-danger">{title}</span>
		{#if code}<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{code}</span>{:else}<span class="flex-1"></span>{/if}
		{#if requestId}<span class="font-mono text-label text-text-3">{requestId}</span>{/if}
	</header>
	{#if details.length > 0}
		<dl class="flex flex-col gap-2 bg-surface p-3.5">
			{#each details as detail (detail.key)}
				<div class="flex gap-3">
					<dt class="w-24 shrink-0 font-mono text-label text-text-3">{detail.key}</dt>
					<dd class={['min-w-0 font-mono text-label break-words', detail.strong ? 'text-text' : 'text-text-2']}>{detail.value}</dd>
				</div>
			{/each}
		</dl>
	{/if}
	{#if actions.length > 0}
		<div class="flex flex-wrap items-center gap-2 border-t border-border bg-surface px-3.5 pt-2.25 pb-2.5">
			{#each actions as action, index (action.label)}
				<Button label={action.label} variant={index === 0 ? 'inverse' : 'secondary'} iconEnd={action.icon} href={action.href} external={action.external} onclick={action.onclick} />
			{/each}
		</div>
	{/if}
</section>
