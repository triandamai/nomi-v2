<!-- Pick a role: build, prompt or view. -->
<script lang="ts">
	import RoleMenu from '$lib/ds/build/RoleMenu.svelte';

	let role = $state<'build' | 'prompt' | 'view'>('build');
</script>

<RoleMenu value={role} onchange={(next) => (role = next)} />
