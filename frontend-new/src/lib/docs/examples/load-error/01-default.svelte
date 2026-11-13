<!-- Default: the apps are fine; quote the server's reason. -->
<script lang="ts">
	import LoadError from '$lib/ds/projects/LoadError.svelte';
</script>

<LoadError reason="503 from api.nomi.app · 09:14" onretry={() => {}} />
