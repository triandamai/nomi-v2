<!-- A key the provider refused. -->
<script lang="ts">
	import ConnectPane from '$lib/ds/settings/ConnectPane.svelte';

	let key = $state('sk-wrong');
</script>

<div class="flex h-120 w-full max-w-145 flex-col overflow-hidden rounded-md bg-surface outline -outline-offset-1 outline-border-strong">
	<ConnectPane
		title="Connect OpenAI"
		name="OpenAI"
		billing="billed by OpenAI · doesn’t count toward your plan"
		closeLabel="Close"
		field={{ label: 'API key', secret: true }}
		bind:value={key}
		check={{ kind: 'failed', text: 'The provider refused that key: Incorrect API key provided' }}
		foot="the key is encrypted · only used for your requests"
		cancel="Cancel"
		action="Check and connect"
	/>
</div>
