<!-- From the card: press ⋯. Only the actions given show; Visit and Take down only when it's live. -->
<script lang="ts">
	import ProjectCardMenu from '$lib/ds/projects/ProjectCardMenu.svelte';

	const noop = () => {};
</script>

<ProjectCardMenu
	name="salah-daily-tracker"
	live
	actions={{ onopen: noop, onvisit: noop, onrename: noop, onduplicate: noop, onmove: noop, onshare: noop, onexport: noop, onsettings: noop, ontakedown: noop, ondelete: noop }}
/>
