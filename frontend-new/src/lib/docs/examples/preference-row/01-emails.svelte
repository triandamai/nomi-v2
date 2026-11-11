<!-- Emails: on and off. -->
<script lang="ts">
	import PreferenceRow from '$lib/ds/inbox/PreferenceRow.svelte';

	let account = $state(true);
	let promos = $state(false);
</script>

<div class="flex w-full max-w-75 flex-col gap-3">
	<PreferenceRow title="Email account changes" description="Invites, plan and billing. Never promos." bind:checked={account} />
	<PreferenceRow title="Email news and offers" description="What’s new in Nomi, now and then." bind:checked={promos} />
</div>
