<!-- A breakdown panel. -->
<script lang="ts">
	import StatPanel from '$lib/ds/profile/StatPanel.svelte';
	import UsageBars from '$lib/ds/profile/UsageBars.svelte';
</script>

<div class="w-full max-w-64">
	<StatPanel title="Models you use most" meta="last 30 days">
		<UsageBars bars={[{ label: 'deepseek-flash', value: '13.5K · 62%', share: 1 }, { label: 'gemini-flash', value: '5.2K · 24%', share: 0.39 }]} empty="No tokens in the last 30 days." />
	</StatPanel>
</div>
