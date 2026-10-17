<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';
	import HomeLayout from '$lib/components/home/HomeLayout.svelte';
	import { homeLayout } from '$lib/homeLayouts';
	import type { ActionData, PageData } from './$types';

	// Home is whichever layout the person picked in Preferences → Home (see $lib/homeLayouts).

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const layout = $derived(homeLayout(data.preferences.home_layout));
</script>

<HomeLayout {layout} home={data} error={form?.error ?? null}>
	{#snippet notice()}
		{#if page.url.searchParams.get('error') === 'forbidden'}
			<p class="md-body-medium home__error" role="alert">{m.home_forbidden()}</p>
		{/if}
	{/snippet}
</HomeLayout>

<style>
	.home__error {
		color: var(--md-sys-color-error);
		margin: 0;
	}
</style>
