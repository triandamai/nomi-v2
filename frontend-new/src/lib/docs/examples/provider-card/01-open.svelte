<!-- Open: Nomi's models, one of them the default. -->
<script lang="ts">
	import ProviderCard from '$lib/ds/settings/ProviderCard.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
</script>

<div class="w-full max-w-220">
	<ProviderCard
		name="Nomi"
		mascot="nomi"
		detail="Free plan · 21.8K / 1M tokens this month · no key needed"
		count="1 of 2 in picker"
		models={[
			{ key: 'admin:a1', label: 'deepseek-flash', enabled: true, isDefault: true, month: '13.5K' },
			{ key: 'admin:a2', label: 'gemini-pro', enabled: false, isDefault: false, month: '—' },
		]}
		columns={{ model: 'Model', month: 'This month', picker: 'In picker' }}
		defaultLabel="Default for chats"
		empty="It offers no models right now."
		open
	>
		{#snippet tag()}<Tag kind="go" label="Included" />{/snippet}
	</ProviderCard>
</div>
