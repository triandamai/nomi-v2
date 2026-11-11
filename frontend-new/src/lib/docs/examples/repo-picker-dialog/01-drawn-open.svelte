<!-- Drawn open: choose a repo; one Nomi can't see offers Grant access. -->
<script lang="ts">
	import RepoPickerDialog from '$lib/ds/projects/RepoPickerDialog.svelte';
</script>

<RepoPickerDialog
	inline
	workspace="Trian's space"
	owner="mario"
	value="r1"
	repos={[
		{ id: 'r1', name: 'mario/kasir-toko', meta: 'SvelteKit · private · updated 3d ago' },
		{ id: 'r2', name: 'mario/portfolio-2024', meta: 'Astro · public · 2mo ago' },
		{ id: 'r3', name: 'kopikita/web', meta: 'not shared with Nomi', grantHref: '#grant' },
	]}
	onimport={() => {}}
/>
