<!-- Default: what Koda found, what it sets up, and the secrets it needs. -->
<script lang="ts">
	import { Database, GitBranch, Globe } from '@lucide/svelte';
	import ImportReview from '$lib/ds/projects/ImportReview.svelte';
</script>

<ImportReview
	repo="kasir-toko"
	meta="48 files · 00:41"
	found={{ app: 'SvelteKit 2 · builds with npm run build', tests: '12 tests · all pass', data: 'Postgres via DATABASE_URL', secrets: "MIDTRANS_KEY and ADMIN_PIN aren't in the repo" }}
	setup={[
		{ icon: Database, text: 'An empty Postgres database, migrated from /migrations' },
		{ icon: Globe, text: 'kasir-toko-3e9a.trian.space' },
		{ icon: GitBranch, text: 'Commits on nomi/main; your main stays untouched' },
	]}
	secrets={['MIDTRANS_KEY', 'ADMIN_PIN']}
	ondeploy={() => {}}
	onlater={() => {}}
/>
