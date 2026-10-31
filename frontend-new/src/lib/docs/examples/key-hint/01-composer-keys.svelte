<!-- The composer's keys: the few that matter in this field, in a quiet mono line. -->
<script lang="ts">
	import KeyHint from '$lib/ds/actions/KeyHint.svelte';
</script>

<KeyHint
	hints={[
		{ keys: '↵', action: 'send' },
		{ keys: '⇧↵', action: 'new line' },
		{ keys: '@', action: 'address crew' },
	]}
/>
