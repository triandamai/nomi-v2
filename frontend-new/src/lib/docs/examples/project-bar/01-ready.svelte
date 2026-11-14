<!-- A project built and ready, not yet live. -->
<script lang="ts">
	import { Activity, Database, History, Link, Settings } from '@lucide/svelte';
	import ProjectBar from '$lib/ds/projects/ProjectBar.svelte';
	import PublishButton from '$lib/ds/build/PublishButton.svelte';
</script>

<div class="w-full overflow-x-auto">
	<div class="min-w-300">
		<ProjectBar
			backHref="#"
			backLabel="Back to projects"
			agent="koda"
			name="warung-kopi-menu"
			status={{ label: 'ready', tone: 'warning' }}
			space="trian.space"
			url="https://warung-kopi-menu-3f9a12.trian.space"
			navLabel="Project tools"
			nav={[
				{ id: 'deployments', label: 'Deployments', icon: History },
				{ id: 'domain', label: 'Domain', icon: Link },
				{ id: 'monitor', label: 'Monitor', icon: Activity },
				{ id: 'database', label: 'Database', icon: Database },
				{ id: 'settings', label: 'Settings', icon: Settings },
			]}
			shareLabel="Share"
			exportLabel="Export repo"
			initials="MP"
		>
			{#snippet publish()}<PublishButton state="ready" count={2} onpublish={() => {}} />{/snippet}
		</ProjectBar>
	</div>
</div>
