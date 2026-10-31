<!-- Kinds: a dot plus a word, never colour alone. Working shows who is working. -->
<script lang="ts">
	import Status from '$lib/ds/status/Status.svelte';
</script>

<Status kind="live" label="Live" />
<Status kind="ready" label="ready" />
<Status kind="idle" label="idle" />
<Status kind="failed" label="Deploy failed" />
<Status kind="attention" label="Needs attention" />
<Status kind="working" label="Koda is building" agent="koda" />
