<!-- Versions: step between a message's versions; the ends turn off. -->
<script lang="ts">
	import VersionSwitcher from '$lib/ds/chat/VersionSwitcher.svelte';

	let current = $state(2);
</script>

<VersionSwitcher {current} total={2} onprev={() => (current -= 1)} onnext={() => (current += 1)} />
