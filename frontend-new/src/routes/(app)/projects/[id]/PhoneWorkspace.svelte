<script lang="ts">
	import type { Workspace } from './workspace.svelte';
	import type { PageData } from './$types';
	type Props = { ws: Workspace; data: PageData };
	let { ws, data }: Props = $props();
</script>

<div>{ws.project.name}{data.title}</div>
