<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import ProvidersScreen from '$lib/components/providers/ProvidersScreen.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<PageHeader title={m.prov_title()} lede={m.prov_admin_lede()} />

<div class="body">
	<ProvidersScreen providers={data.providers} connections={data.connections} loadFailed={data.loadFailed} />
</div>

<style>
	.body {
		max-width: 720px;
		margin-top: 24px;
	}
</style>
