<!-- Out of credits: Nomi credits first, then Retry. -->
<script lang="ts">
	import { RotateCcw, Sparkles } from '@lucide/svelte';
	import PhoneErrorCard from '$lib/ds/thread/PhoneErrorCard.svelte';
</script>

<div class="w-89.5">
	<PhoneErrorCard
		title="Out of credits"
		sentence="I couldn’t reply: your DeepSeek account is out of credits for this request."
		meta="deepseek-flash"
		primary={{ label: 'Switch to Nomi credits', icon: Sparkles }}
		secondary={{ label: 'Retry', icon: RotateCcw }}
	/>
</div>
