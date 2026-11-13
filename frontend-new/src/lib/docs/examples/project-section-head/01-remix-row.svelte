<!-- The remix row's head, with its link. -->
<script lang="ts">
	import { GitFork } from '@lucide/svelte';
	import ProjectSectionHead from '$lib/ds/projects/ProjectSectionHead.svelte';
</script>

<ProjectSectionHead
	icon={GitFork}
	title="Remix from the community"
	description="Start from an app someone already built. You get your own copy — their data stays theirs."
	link={{ label: 'Browse showcase', href: '#showcase' }}
/>
