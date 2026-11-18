<!-- Filtered to the models that can see a picture, by who pays, with the one left out counted. -->
<script lang="ts">
	import VisionModelMenu from '$lib/ds/thread/VisionModelMenu.svelte';
</script>

<VisionModelMenu
	groups={[
		{ group: 'Your keys', models: [{ key: 'k1', label: 'claude-sonnet', meta: 'Anthropic · reads images' }] },
		{ group: 'Nomi credits', models: [{ key: 'n1', label: 'gemini-flash', meta: 'Nomi credits · reads images' }] },
	]}
	hidden="deepseek-v3 hidden · can’t see images"
	settingsHref="#"
	onpick={() => {}}
	ontextonly={() => {}}
	onclose={() => {}}
/>
