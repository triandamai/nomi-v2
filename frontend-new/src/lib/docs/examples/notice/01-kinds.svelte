<!-- Kinds: saved, failed with the backend's reason, a test that passed, something to know first. -->
<script lang="ts">
	import Notice from '$lib/ds/admin/Notice.svelte';
</script>

<div class="flex w-full max-w-120 flex-col gap-2">
	<Notice kind="success" text="Saved. It applies within a few seconds." />
	<Notice kind="error" text="Couldn’t save the plan." detail="another plan already uses that slug" />
	<Notice kind="info" text="Xendit client initialized successfully" />
	<Notice kind="warning" text="Publishing isn’t set up on this Nomi." detail="SHIPYARD_URL and SHIPYARD_KEY" />
</div>
