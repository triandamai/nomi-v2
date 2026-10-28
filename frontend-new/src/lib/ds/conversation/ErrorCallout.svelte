<script lang="ts">
	import { ArrowUpRight, CircleAlert, RotateCcw } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';

	// Design System / Nomi → Conversation: a failure, named plainly, with the provider's own words
	// quoted in mono (code, request id, reason) and the fix one click away.
	type Detail = { key: string; value: string; strong?: boolean };
	type Props = {
		title: string;
		code?: string;
		requestId?: string;
		details?: Detail[];
		retryLabel?: string;
		onretry?: () => void;
		fix?: { label: string; href: string };
	};
	let { title, code, requestId, details = [], retryLabel, onretry, fix }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong" role="alert">
	<header class="flex items-center gap-2 border-b border-border bg-danger-bg px-3.5 py-2.5">
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
					<dd class={['font-mono text-label', detail.strong ? 'text-text' : 'text-text-2']}>{detail.value}</dd>
				</div>
			{/each}
		</dl>
	{/if}
	{#if (retryLabel && onretry) || fix}
		<div class="flex items-center gap-2 border-t border-border bg-surface px-3.5 py-2.5">
			{#if retryLabel && onretry}<Button label={retryLabel} variant="inverse" iconStart={RotateCcw} onclick={onretry} />{/if}
			{#if fix}<Button label={fix.label} variant="secondary" href={fix.href} external iconEnd={ArrowUpRight} />{/if}
		</div>
	{/if}
</section>
