<!-- A live project's sheet: open, change and share, remove. -->
<script lang="ts">
	import PhoneProjectMenu from '$lib/ds/projects/PhoneProjectMenu.svelte';

	const noop = () => {};
	const actions = { onopen: noop, onvisit: noop, onrename: noop, onduplicate: noop, onmove: noop, onshare: noop, onexport: noop, onsettings: noop, ontakedown: noop, ondelete: noop };
</script>

<div class="flex w-97.5 max-w-full flex-col rounded-t-sheet bg-surface py-2 outline -outline-offset-1 outline-border-strong">
	<PhoneProjectMenu {actions} live />
</div>
