<!-- Four states: up to date, ready with what will go live, publishing with its time, failed with a retry. -->
<script lang="ts">
	import PublishButton from '$lib/ds/build/PublishButton.svelte';
</script>

<PublishButton state="idle" />
<PublishButton state="ready" count={2} onpublish={() => {}} />
<PublishButton state="publishing" elapsed="0:12" />
<PublishButton state="failed" onretry={() => {}} />
