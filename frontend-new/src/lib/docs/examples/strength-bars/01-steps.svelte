<!-- The five steps. -->
<script lang="ts">
	import StrengthBars from '$lib/ds/settings/StrengthBars.svelte';
</script>

<div class="flex items-start gap-6">
	<StrengthBars level={1} name="faint" />
	<StrengthBars level={2} name="weak" />
	<StrengthBars level={3} name="steady" />
	<StrengthBars level={4} name="strong" />
	<StrengthBars level={5} name="core" />
</div>
