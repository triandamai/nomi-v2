<!-- Plan usage: the percentage, and the real figure under it. -->
<script lang="ts">
	import Meter from '$lib/ds/status/Meter.svelte';
</script>

<div class="w-50">
	<Meter label="Free plan" value={0.02} caption="21.8K / 1M tokens this month" />
</div>
