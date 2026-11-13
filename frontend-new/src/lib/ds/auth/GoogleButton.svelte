<script lang="ts">
	import { LoaderCircle } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Auth / Sign in or create → Google: the first way in, 44 high across the form, a bordered
	// surface with the G as the pen draws it (a text-coloured disc, not Google's logo) and, when this
	// browser signed in with Google last time, "last used" in mono accent. It submits the form it
	// sits in (which starts the Google sign-in); loading keeps it from going twice.
	type Props = {
		lastUsed?: boolean;
		loading?: boolean;
		disabled?: boolean;
		type?: 'submit' | 'button';
		onclick?: (event: MouseEvent) => void;
		[attribute: `data-${string}`]: unknown;
	};
	let { lastUsed = false, loading = false, disabled = false, type = 'submit', onclick, ...rest }: Props = $props();
</script>

<button
	{type}
	{onclick}
	disabled={disabled || loading}
	aria-busy={loading || undefined}
	{...rest}
	class="flex h-11 w-full cursor-pointer items-center justify-center gap-2.5 rounded-sm border border-border-strong bg-surface text-ui font-medium text-text outline-offset-0 transition-colors select-none hover:enabled:bg-surface-2 focus-visible:border-transparent focus-visible:outline-3 disabled:cursor-default data-hover:bg-surface-2"
>
	{#if loading}
		<span class="inline-flex animate-spin text-text-2 motion-reduce:animate-none"><Icon icon={LoaderCircle} size={14} /></span>
	{:else}
		<span aria-hidden="true" class="flex size-4.5 shrink-0 items-center justify-center rounded-full bg-text text-label font-semibold text-bg">G</span>
	{/if}
	<span>{m.auth_google()}</span>
	{#if lastUsed}<span class="font-mono text-label font-normal text-accent">{m.auth_last_used()}</span>{/if}
</button>
