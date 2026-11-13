<!-- Nomi's own, and a provider's. -->
<script lang="ts">
	import ProviderTile from '$lib/ds/settings/ProviderTile.svelte';
</script>

<div class="flex items-center gap-3">
	<ProviderTile name="Nomi" mascot="nomi" />
	<ProviderTile name="Anthropic" />
	<ProviderTile name="Anthropic" size={20} />
</div>
