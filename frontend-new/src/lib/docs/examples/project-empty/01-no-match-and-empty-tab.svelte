<!-- No match and an empty tab: one with something to do, one only words. -->
<script lang="ts">
	import { CircleCheck, SearchX, X } from '@lucide/svelte';
	import ProjectEmpty from '$lib/ds/projects/ProjectEmpty.svelte';
</script>

<div class="flex flex-col gap-6">
	<ProjectEmpty icon={SearchX} title="No projects match “kasir”" body="Search looks at names, descriptions and URLs in Trian's space." action={{ label: 'Clear search', icon: X, onclick: () => {} }} />
	<ProjectEmpty icon={CircleCheck} title="Nothing needs attention" body="When a deploy fails or Koda is waiting on you, the project shows up here." />
</div>
