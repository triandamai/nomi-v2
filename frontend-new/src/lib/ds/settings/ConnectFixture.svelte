<script lang="ts">
	import ConnectDialog from './ConnectDialog.svelte';
	import ProviderList from './ProviderList.svelte';
	import ConnectPane from './ConnectPane.svelte';

	// For the tests: the Connect dialog's box with its list and a pane that has checked a key.
	let { onaction, onpick }: { onaction?: () => void; onpick?: (id: string) => void } = $props();
	let key = $state('sk-ant-api03-3f9a');
</script>

<ConnectDialog>
	{#snippet list()}
		<ProviderList
			search="Search providers"
			sections={[
				{ label: 'Connected', providers: [{ id: 'ollama', name: 'Ollama', connected: true }] },
				{ label: 'A', providers: [{ id: 'anthropic', name: 'Anthropic' }] },
				{ label: 'O', providers: [{ id: 'ollama', name: 'Ollama', connected: true }] },
			]}
			chosen="anthropic"
			empty="No provider by that name."
			custom={{ title: 'Custom endpoint', detail: 'Any OpenAI-compatible URL' }}
			{onpick}
		/>
	{/snippet}
	{#snippet pane()}
		<ConnectPane
			title="Connect Anthropic"
			name="Anthropic"
			billing="billed by Anthropic · doesn’t count toward your plan"
			closeLabel="Close"
			field={{ label: 'API key', secret: true, getKey: { href: 'https://console.anthropic.com', label: 'Get a key' } }}
			bind:value={key}
			check={{ kind: 'ok', text: 'It works · 2 models found' }}
			models={{ label: 'Show in picker', note: '1 selected', items: [{ id: 'claude-sonnet-5-5', label: 'claude-sonnet-5-5', shown: true }, { id: 'claude-haiku-5-5', label: 'claude-haiku-5-5', shown: false }] }}
			foot="the key is encrypted · only used for your requests"
			cancel="Cancel"
			action="Add 1 model"
			{onaction}
		/>
	{/snippet}
</ConnectDialog>
