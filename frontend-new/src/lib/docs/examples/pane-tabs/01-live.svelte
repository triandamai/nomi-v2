<!-- The live app, on deploy #1. -->
<script lang="ts">
	import PaneTabs from '$lib/ds/projects/PaneTabs.svelte';
</script>

<div class="w-full max-w-250">
	<PaneTabs
		labels={{ views: 'View', preview: 'Preview', code: 'Code', terminal: 'Terminal', device: 'Device', desktop: 'Desktop', phone: 'Phone', reload: 'Reload', open: 'Open in a new tab', close: 'Close the preview' }}
		address={{ url: '#', shown: 'warung-kopi-menu-3f9a12.trian.space', live: 'live', deploy: 'deploy #1' }}
		onclose={() => {}}
	/>
</div>
