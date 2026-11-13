<!-- Models: each against the most used. -->
<script lang="ts">
	import UsageBars from '$lib/ds/profile/UsageBars.svelte';
</script>

<div class="w-full max-w-56">
	<UsageBars
		bars={[
			{ label: 'deepseek-flash', value: '13.5K · 62%', share: 1 },
			{ label: 'gemini-flash', value: '5.2K · 24%', share: 0.39 },
			{ label: 'qwen-coder', value: '2.0K · 9%', share: 0.15 },
		]}
		empty="No tokens in the last 30 days."
	/>
</div>
