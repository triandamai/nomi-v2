<script lang="ts">
	import PaymentSettings from '../PaymentSettings.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<PaymentSettings access={data.access} provider="doku" />
