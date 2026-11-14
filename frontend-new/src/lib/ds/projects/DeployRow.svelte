<script lang="ts">
	import { RotateCcw } from '@lucide/svelte';
	import Button from '../actions/Button.svelte';
	import Status from '../status/Status.svelte';

	// Flow / Build — Workspace tools (U16wU) → Screen / Deployments (S6Xq9H) → Deploy: one publish,
	// [14,0] over its line (13 below) and 14 apart: its number in mono (40 wide), what it is (13/500)
	// over when and who in mono, and on the right Live, Publishing, Failed (its reason under the
	// title, in the danger colour) or "Roll back to #n". A deploy whose source wasn't kept shows
	// nothing to do.
	type Props = {
		number: string;
		title: string;
		meta: string;
		state: 'live' | 'running' | 'failed' | 'rollback' | 'kept';
		error?: string;
		labels: { live: string; failed: string; running: string; rollback: string };
		busy?: boolean;
		onrollback?: () => void;
	};
	let { number, title, meta, state: shown, error, labels, busy = false, onrollback }: Props = $props();
</script>

<li class="flex min-h-15.5 items-center gap-3.5 border-b border-border pt-3.5 pb-3.25">
	<span class="w-10 shrink-0 font-mono text-ui text-text">{number}</span>
	<span class="flex min-w-0 flex-1 flex-col gap-0.75">
		<span class="truncate text-ui font-medium text-text">{title}</span>
		<span class="truncate font-mono text-label text-text-3">{meta}</span>
		{#if shown === 'failed' && error}<span class="text-caption text-danger">{error}</span>{/if}
	</span>
	{#if shown === 'live'}
		<Status kind="live" label={labels.live} small />
	{:else if shown === 'running'}
		<Status kind="working" label={labels.running} small />
	{:else if shown === 'failed'}
		<span class="shrink-0 text-label text-danger">{labels.failed}</span>
	{:else if shown === 'rollback'}
		<Button variant="secondary" iconEnd={RotateCcw} label={labels.rollback.replace('{number}', number)} loading={busy} onclick={onrollback} />
	{/if}
</li>
