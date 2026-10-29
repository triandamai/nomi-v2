<script lang="ts">
	import { CircleAlert, CircleCheck, LoaderCircle, Rocket } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Build → Workspace / Publish, in the top bar. Changes stay in the preview until you press it;
	// the count is what will go live. Up to date after a publish; publishing shows its time; a
	// failed publish keeps the last live deploy running and offers a retry.
	type Props = {
		state: 'idle' | 'ready' | 'publishing' | 'failed';
		count?: number;
		elapsed?: string;
		onpublish?: () => void;
		onretry?: () => void;
	};
	let { state, count = 0, elapsed = '', onpublish, onretry }: Props = $props();
</script>

<button
	type="button"
	disabled={state === 'idle' || state === 'publishing'}
	aria-busy={state === 'publishing' || undefined}
	aria-label={state === 'ready' && count ? m.ws_publish_count({ count: String(count) }) : undefined}
	onclick={state === 'failed' ? onretry : onpublish}
	class={[
		'inline-flex h-7.5 shrink-0 cursor-pointer items-center gap-1.5 rounded-sm px-3 text-ui whitespace-nowrap outline-offset-0 focus-visible:outline-3 disabled:cursor-default pointer-coarse:min-h-11',
		state === 'idle' && 'border border-border-strong text-text-2',
		state === 'ready' && 'bg-accent font-medium text-accent-ink',
		state === 'publishing' && 'bg-accent-bg font-medium text-accent',
		state === 'failed' && 'border border-danger-border bg-danger-bg font-medium text-danger',
	]}
>
	{#if state === 'idle'}
		<Icon icon={CircleCheck} size={14} />{m.ws_up_to_date()}
	{:else if state === 'ready'}
		<Icon icon={Rocket} size={14} />{m.ws_publish()}
		{#if count}<span class="inline-flex h-4.5 items-center rounded-full bg-accent-ink px-1.5 font-mono text-label font-normal text-accent">{count}</span>{/if}
	{:else if state === 'publishing'}
		<span class="inline-flex animate-spin motion-reduce:animate-none"><Icon icon={LoaderCircle} size={14} /></span>{m.ws_publishing({ elapsed })}
	{:else}
		<Icon icon={CircleAlert} size={14} />{m.ws_publish_failed()}
	{/if}
</button>
