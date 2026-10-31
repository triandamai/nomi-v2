<!-- Everything included: the repo, a database dump, the env var names, and two equal ways out. -->
<script lang="ts">
	import ExportPanel from '$lib/ds/build/ExportPanel.svelte';
</script>

<ExportPanel
	project="warung-kopi-menu"
	repo="9 files · git history · main@4be1c07"
	database="warung-kopi-menu.sql · menu_items, orders"
	env="DATABASE_URL · PUBLIC_SITE_URL · ADMIN_PIN"
	onpush={() => {}}
	ondownload={() => {}}
	onclose={() => {}}
/>
