<!-- Memory & context, with the server down. -->
<script lang="ts">
	import PhoneLoadError from '$lib/ds/settings/PhoneLoadError.svelte';
</script>

<div class="w-97.5 max-w-full">
	<PhoneLoadError title="Your memory couldn’t load" body="Nomi’s server didn’t answer (503). Nothing you saved is lost." retry="Try again" onretry={() => {}} />
</div>
