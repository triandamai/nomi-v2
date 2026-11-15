<!-- Settings column: Learning's switches and the two budgets, the memory one past 80%. -->
<script lang="ts">
	import MemorySettings from '$lib/ds/settings/MemorySettings.svelte';

	const labels = {
		learning: 'Learning',
		budget: 'Budget',
		learn: { title: 'Learn from chats', body: 'Keep lasting facts from your conversations.' },
		profile: { title: 'Keep a profile', body: 'A compact summary of who you are and how you work.' },
		search: { title: 'Search past chats', body: 'Let the crew look up details in old chats when relevant.' },
		memoryBudget: 'Memory',
		profileBudget: 'Profile',
		unit: 'tok',
		used: (used: string, budget: string) => `${used} / ${budget} used · loaded into every reply`,
		outOfRange: (min: string, max: string) => `Between ${min} and ${max} tokens.`,
	};
	let values = $state({ learn_from_chats: true, keep_profile: true, search_past_chats: false, memory_budget: 2200, profile_budget: 1375, memory_used: 1812, profile_used: 312 });
</script>

<div class="outline -outline-offset-1 outline-border">
	<MemorySettings {labels} {values} language="en" onchange={(change) => (values = { ...values, ...change })} />
</div>
