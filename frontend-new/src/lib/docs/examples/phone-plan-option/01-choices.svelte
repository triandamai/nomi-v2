<!-- Compare plans: yours, one to buy, one coming soon. -->
<script lang="ts">
	import PhonePlanOption from '$lib/ds/settings/PhonePlanOption.svelte';
	import PhoneButton from '$lib/ds/mobile/PhoneButton.svelte';
</script>

<div class="flex w-97.5 max-w-full flex-col gap-2.5 px-4">
	<PhonePlanOption name="Free" price="Rp 0" line="1M tokens a month · every crew member" yours="Your plan" />
	<PhonePlanOption name="Pro" price="Rp 99.000" line="10M tokens a month · priority models">
		<PhoneButton label="Upgrade to Pro" onclick={() => {}} />
	</PhonePlanOption>
	<PhonePlanOption name="Team" price="Coming soon" line="Shared allowance for a workspace">
		<PhoneButton label="Join the waitlist" variant="secondary" onclick={() => {}} />
	</PhonePlanOption>
</div>
