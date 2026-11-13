<!-- Room for another provider. -->
<script lang="ts">
	import ConnectRow from '$lib/ds/settings/ConnectRow.svelte';
</script>

<div class="w-full max-w-220">
	<ConnectRow title="Connect a provider" detail="API key, or a server on your machine" suggestions={[{ id: 'openrouter', name: 'OpenRouter' }, { id: 'anthropic', name: 'Anthropic' }]} all="All 74" />
</div>
