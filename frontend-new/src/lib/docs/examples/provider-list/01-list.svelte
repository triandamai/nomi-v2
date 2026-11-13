<!-- The providers, grouped. -->
<script lang="ts">
	import ProviderList from '$lib/ds/settings/ProviderList.svelte';
</script>

<div class="flex h-120 w-80 flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border">
	<ProviderList
		search="Search providers"
		sections={[
			{ label: 'Connected', providers: [{ id: 'ollama', name: 'Ollama', connected: true }] },
			{ label: 'Popular', providers: [{ id: 'anthropic', name: 'Anthropic' }, { id: 'openai', name: 'OpenAI' }] },
			{ label: 'A', providers: [{ id: 'abacus', name: 'Abacus' }, { id: 'anthropic', name: 'Anthropic' }] },
			{ label: 'O', providers: [{ id: 'ollama', name: 'Ollama', connected: true }, { id: 'openai', name: 'OpenAI' }] },
		]}
		chosen="anthropic"
		empty="No provider by that name."
		custom={{ title: 'Custom endpoint', detail: 'Any OpenAI-compatible URL' }}
	/>
</div>
