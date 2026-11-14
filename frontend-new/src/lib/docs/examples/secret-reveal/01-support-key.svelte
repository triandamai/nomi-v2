<!-- A support key, shown once. -->
<script lang="ts">
	import SecretReveal from '$lib/ds/admin/SecretReveal.svelte';
</script>

<div class="w-full max-w-120">
	<SecretReveal title="Copy this key now: it won’t be shown again." secret="nsk_5f0c2a9e41d7b3c8a6e2f19d04b7c3e8a1f6d2b9c4e7a0f3" copyLabel="Copy" copiedLabel="Copied" doneLabel="Done" ondone={() => {}} />
</div>
