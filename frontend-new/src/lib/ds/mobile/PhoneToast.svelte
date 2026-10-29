<script lang="ts">
	import { Check, CircleAlert, Info } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Toast: what just happened, at the top of a phone screen: 16pt corners, the one
	// floating shadow, one action. Errors interrupt (alert); the rest wait their turn (status).
	type Props = { kind: 'success' | 'error' | 'info'; title: string; detail?: string; action?: { label: string; onclick: () => void } };
	let { kind, title, detail, action }: Props = $props();
</script>

<div role={kind === 'error' ? 'alert' : 'status'} class="flex w-full items-center gap-3 rounded-2xl border border-border-strong bg-surface px-3.5 py-3 shadow-float">
	<span class={['inline-flex shrink-0', kind === 'success' && 'text-accent', kind === 'error' && 'text-danger', kind === 'info' && 'text-text-2']}>
		<Icon icon={kind === 'success' ? Check : kind === 'error' ? CircleAlert : Info} size={18} />
	</span>
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="text-body font-medium text-text">{title}</span>
		{#if detail}<span class="text-ui text-text-3">{detail}</span>{/if}
	</span>
	{#if action}
		<button type="button" class="cursor-pointer rounded-xs text-body font-semibold text-text outline-offset-2 pointer-coarse:min-h-11" onclick={action.onclick}>{action.label}</button>
	{/if}
</div>
