<script lang="ts">
	import { ArrowDownUp, Search as SearchIcon, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Project Search and its Filled state (Flow / Mobile Projects — Find & filter, oFdFf):
	// [0,16,10,16], 8 apart, the field 40 tall on surface-2 with 12 corners ([0,12], the 16 glyph,
	// the text at 16pt so iOS doesn't zoom) and Sort (40, surface-2, 12 corners, arrow-down-up)
	// opening the sort sheet. Searching, the field's stroke is focus (1px, no glow), a 20 round clear
	// (text-3, its x 12 in surface-2) sits at its end once something is typed, and a text Cancel (16,
	// text) takes Sort's place: it empties the search and lets the field go.
	type Props = { placeholder: string; value?: string; sortLabel: string; onsort?: () => void };
	let { placeholder, value = $bindable(''), sortLabel, onsort }: Props = $props();

	let field: HTMLInputElement | undefined = $state();
	let focused = $state(false);
	const searching = $derived(focused || value !== '');
</script>

<div class="flex items-center gap-2 px-4 pb-2.5">
	<label class="flex h-10 min-w-0 flex-1 items-center gap-2 rounded-lg border border-transparent bg-surface-2 px-2.75 text-text-3 focus-within:border-focus">
		<Icon icon={SearchIcon} size={16} />
		<input
			bind:this={field}
			type="search"
			{placeholder}
			aria-label={placeholder}
			bind:value
			onfocus={() => (focused = true)}
			onblur={() => (focused = false)}
			class="min-w-0 flex-1 bg-transparent text-phone text-text outline-none placeholder:text-text-3"
		/>
		{#if value}
			<button
				type="button"
				aria-label={m.projects_clear_search()}
				class="inline-flex size-5 shrink-0 cursor-pointer items-center justify-center rounded-full bg-text-3 text-surface-2 outline-offset-2"
				onmousedown={(event) => event.preventDefault()}
				onclick={() => {
					value = '';
					field?.focus();
				}}
			>
				<Icon icon={X} size={12} />
			</button>
		{/if}
	</label>
	{#if searching}
		<button
			type="button"
			class="inline-flex h-10 shrink-0 cursor-pointer items-center text-phone text-text outline-offset-2"
			onmousedown={(event) => event.preventDefault()}
			onclick={() => {
				value = '';
				field?.blur();
				focused = false;
			}}>{m.action_cancel()}</button
		>
	{:else if onsort}
		<button type="button" aria-label={sortLabel} class="inline-flex size-10 shrink-0 cursor-pointer items-center justify-center rounded-lg bg-surface-2 text-text-2 outline-offset-2" onclick={onsort}>
			<Icon icon={ArrowDownUp} size={18} />
		</button>
	{/if}
</div>

<style>
	/* The field has its own clear (the pen's), so the browser's own x stays hidden. */
	input::-webkit-search-cancel-button {
		display: none;
	}
</style>
