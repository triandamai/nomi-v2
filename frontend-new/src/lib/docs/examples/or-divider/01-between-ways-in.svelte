<!-- Between two ways in: Google above, the email form below. -->
<script lang="ts">
	import GoogleButton from '$lib/ds/auth/GoogleButton.svelte';
	import OrDivider from '$lib/ds/auth/OrDivider.svelte';
</script>

<div class="flex w-full max-w-95 flex-col gap-5">
	<GoogleButton type="button" />
	<OrDivider />
</div>
