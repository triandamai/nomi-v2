<!-- Drawn open: the panel as the sheet shows it. In a page, use ProjectCardMenu. -->
<script lang="ts">
	import ProjectMenuPanel from '$lib/ds/projects/ProjectMenuPanel.svelte';
	import { projectMenu } from '$lib/ds/projects/ProjectCardMenu.svelte';

	const noop = () => {};
</script>

<ProjectMenuPanel title="salah-daily-tracker" items={projectMenu({ onopen: noop, onrename: noop, onduplicate: noop, onexport: noop, ondelete: noop }, false)} />
