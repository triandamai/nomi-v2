<!-- No match and couldn't load: the phone's empty list, with its one pill. -->
<script lang="ts">
	import { CloudOff, RotateCw, SearchX, X } from '@lucide/svelte';
	import PhoneProjectEmpty from '$lib/ds/projects/PhoneProjectEmpty.svelte';
</script>

<div class="flex w-89.5 max-w-full flex-col">
	<PhoneProjectEmpty icon={SearchX} title="No projects match “kasir”" body="Search looks at names, descriptions and URLs in Trian’s space." action={{ label: 'Clear search', icon: X, onclick: () => {} }} />
	<PhoneProjectEmpty tone="danger" icon={CloudOff} title="Couldn’t load your projects" body="Your apps are still live; only this list failed." code="503 from Nomi’s server · 09:14" action={{ label: 'Retry', icon: RotateCw, onclick: () => {} }} />
</div>
