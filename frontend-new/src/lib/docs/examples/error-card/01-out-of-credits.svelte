<!-- Out of credits: the provider's own words, the request, and the fixes. -->
<script lang="ts">
	import { ArrowUpRight, Sparkles } from '@lucide/svelte';
	import ErrorCard from '$lib/ds/mobile/ErrorCard.svelte';
</script>

<div class="w-89.5 max-w-full">
	<ErrorCard
		title="Out of credits"
		code="402"
		provider="DeepSeek"
		reason="Insufficient Balance"
		meta="deepseek-flash · key ••a91f · req_9d41e0b2"
		primary={{ label: 'Switch to Nomi credits', icon: Sparkles, onclick: () => {} }}
		secondary={{ label: 'Add credits on DeepSeek', icon: ArrowUpRight, onclick: () => {} }}
	/>
</div>
