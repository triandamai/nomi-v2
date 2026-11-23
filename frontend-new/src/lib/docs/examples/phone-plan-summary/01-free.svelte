<!-- Free, a third used, with Compare plans. -->
<script lang="ts">
	import PhonePlanSummary from '$lib/ds/settings/PhonePlanSummary.svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
</script>

<div class="w-97.5 max-w-full px-4">
	<PhonePlanSummary plan="Free" line="1M tokens a month, set for you" used={0.31} usedLabel="312K / 1M tokens · 31% used" resets="resets Mon 1 Dec · 8 days">
		{#snippet action()}<PhoneButton label="Compare plans" variant="secondary" onclick={() => {}} />{/snippet}
	</PhonePlanSummary>
</div>
