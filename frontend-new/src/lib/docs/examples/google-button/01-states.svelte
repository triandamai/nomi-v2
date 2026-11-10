<!-- States: as it first shows, marked "last used", and on its way to Google. -->
<script lang="ts">
	import GoogleButton from '$lib/ds/auth/GoogleButton.svelte';
</script>

<div class="flex w-full max-w-95 flex-col gap-3">
	<GoogleButton type="button" />
	<GoogleButton type="button" lastUsed />
	<GoogleButton type="button" loading />
</div>
