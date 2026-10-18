<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ProvidersScreen from '$lib/components/providers/ProvidersScreen.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<div class="page">
	<div class="page__inner">
		<PageHeader title={m.prov_title()} lede={m.prov_lede()} />
		<ProvidersScreen providers={data.providers} connections={data.connections} loadFailed={data.loadFailed} />
	</div>
</div>

<style>
	.page {
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(14px, 4vw, 44px) 44px;
		box-sizing: border-box;
	}
	.page__inner {
		display: flex;
		flex-direction: column;
		gap: 24px;
		max-width: 720px;
		margin: 0 auto;
	}
</style>
