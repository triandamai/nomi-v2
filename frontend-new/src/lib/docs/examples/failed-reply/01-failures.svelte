<!-- Failures: a provider out of credits, and Nomi's own allowance used up. -->
<script lang="ts">
	import { ArrowUpRight, KeyRound, RotateCcw, Sparkles } from '@lucide/svelte';
	import FailedReply from '$lib/ds/thread/FailedReply.svelte';
</script>

<div class="flex w-full max-w-180 flex-col gap-8">
	<FailedReply
		text="I couldn't reply because your DeepSeek account is out of credits for this request. Add credits or raise the API key's limit with DeepSeek, then send your message again."
		title="Out of credits"
		actions={[
			{ label: 'Retry', icon: RotateCcw, onclick: () => {} },
			{ label: 'Switch to Nomi credits', icon: Sparkles, onclick: () => {} },
		]}
	/>
	<FailedReply
		text="You've used this month's allowance on Nomi's models. Add your own API key in Models to keep going, or ask for a bigger plan in Billing & usage."
		title="Out of Nomi credits"
		actions={[
			{ label: 'Upgrade plan', icon: ArrowUpRight, href: '#plan' },
			{ label: 'Use my own key', icon: KeyRound, href: '#models' },
		]}
	/>
</div>
