<script lang="ts">
	import { Search as SearchIcon, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Design System / Projects → Project / Search (m5wmYk) and Search / Filled (m3EFzX): 220 × 30,
	// 10 in, a quiet stroke while empty. Once it filters the stroke turns accent and a clear (×)
	// comes in at the end; Escape clears too. Keyboard focus adds the field glow, as every field's.
	type Props = { value?: string; placeholder?: string; oninput?: (value: string) => void };
	let { value = $bindable(''), placeholder = m.projects_search(), oninput }: Props = $props();

	let field: HTMLInputElement | undefined = $state();

	function clear() {
		value = '';
		oninput?.('');
		field?.focus();
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key !== 'Escape' || !value) return;
		event.preventDefault();
		clear();
	}
</script>

<label
	class={[
		'flex h-7.5 w-55 max-w-full shrink-0 items-center gap-2 rounded-sm border px-2.25 text-text-3 focus-within:border-accent focus-within:shadow-focus',
		value ? 'border-accent' : 'border-border',
	]}
>
	<Icon icon={SearchIcon} size={14} />
	<input
		bind:this={field}
		type="search"
		{placeholder}
		aria-label={placeholder}
		bind:value
		oninput={() => oninput?.(value)}
		{onkeydown}
		class="min-w-0 flex-1 bg-transparent text-ui text-text outline-none placeholder:text-text-3"
	/>
	{#if value}
		<button type="button" aria-label={m.projects_search_clear()} onclick={clear} class="inline-flex cursor-pointer rounded-xs text-text-3 outline-offset-2 hover:text-text">
			<Icon icon={X} size={14} />
		</button>
	{/if}
</label>

<style>
	/* Its own clear stands in for the browser's. */
	input::-webkit-search-cancel-button {
		appearance: none;
	}
</style>
