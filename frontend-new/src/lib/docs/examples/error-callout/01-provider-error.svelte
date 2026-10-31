<!-- A provider error: the provider's own reason in mono, and the next move first. -->
<script lang="ts">
	import { ArrowUpRight, RotateCcw } from '@lucide/svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
</script>

<div class="w-full max-w-160">
	<ErrorCallout
		title="Provider error"
		code="402 insufficient_balance"
		requestId="req_9d41e0b2"
		details={[
			{ key: 'reason', value: 'Insufficient Balance', strong: true },
			{ key: 'provider', value: 'api.deepseek.com' },
		]}
		actions={[
			{ label: 'Retry', icon: RotateCcw, onclick: () => {} },
			{ label: 'Add credits on DeepSeek', icon: ArrowUpRight, onclick: () => {} },
		]}
	/>
</div>
