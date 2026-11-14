<script lang="ts">
	import type { Snippet } from 'svelte';
	import Search from '../inputs/Search.svelte';
	import Button from '../actions/Button.svelte';

	// Admin → Filter bar: above a list, the search that narrows it (sent as `?query=`, so the address
	// keeps it and the server does the searching), any filters beside it, and the list's own actions
	// at the end ("Add a plan"). One row on a desktop; it wraps on a phone. Enter searches; the button
	// is there for those who look for one. The query goes with the form as a hidden field kept in step
	// with the box (Search, the old app's twin, takes no name).
	type Props = {
		/** Names the search for screen readers ("Search people"). */
		label: string;
		placeholder: string;
		query?: string;
		/** The query's name in the address. */
		name?: string;
		submitLabel: string;
		/** Other values the address keeps when searching (a tab, a page size). */
		keep?: Record<string, string>;
		filters?: Snippet;
		actions?: Snippet;
	};
	let { label, placeholder, query = $bindable(''), name = 'query', submitLabel, keep = {}, filters, actions }: Props = $props();
</script>

<div class="flex w-full flex-wrap items-center gap-2">
	<form role="search" aria-label={label} method="GET" class="flex min-w-0 flex-1 basis-64 items-center gap-2">
		{#each Object.entries(keep) as [key, value] (key)}<input type="hidden" name={key} {value} />{/each}
		<input type="hidden" {name} value={query} />
		<div class="min-w-0 flex-1"><Search {placeholder} {label} bind:value={query} /></div>
		<Button label={submitLabel} variant="secondary" type="submit" />
	</form>
	{#if filters}<div class="flex flex-wrap items-center gap-2">{@render filters()}</div>{/if}
	{#if actions}<div class="ml-auto flex flex-wrap items-center gap-2">{@render actions()}</div>{/if}
</div>
