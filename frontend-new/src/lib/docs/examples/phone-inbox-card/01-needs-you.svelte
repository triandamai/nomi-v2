<!-- Needs you: an unread invite with what to do about it. -->
<script lang="ts">
	import PhoneInboxCard from '$lib/ds/inbox/PhoneInboxCard.svelte';
	import PhonePill from '$lib/ds/actions/PhonePill.svelte';
</script>

<div class="w-97.5 max-w-full">
	<PhoneInboxCard kind="account" when="18 Oct" title="Trian invited you to Trian's space" body="Work on its 4 projects together with the crew. Your own chats and memory stay private." unread meta="expires in 4 days">
		{#snippet actions()}
			<PhonePill label="Open invite" variant="inverse" />
			<PhonePill label="Mark read" />
		{/snippet}
	</PhoneInboxCard>
</div>
