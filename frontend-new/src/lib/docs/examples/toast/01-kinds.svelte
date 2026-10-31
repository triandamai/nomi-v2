<!-- Success, error and info: what happened, the detail in mono, one action and a close. -->
<script lang="ts">
	import { Download } from '@lucide/svelte';
	import Toast from '$lib/ds/overlays/Toast.svelte';
</script>

<div class="flex w-full max-w-100 flex-col gap-3">
	<Toast
		kind="success"
		title="Deployed to salah-daily-tracker-0ab8f8.trian.space"
		detail="deploy #15 · built in 38s"
		action={{ label: 'Visit', onclick: () => {} }}
		ondismiss={() => {}}
	/>
	<Toast
		kind="error"
		title="Deploy failed: build error in src/lib/pdf.ts"
		detail="Koda can try a fix, or open the log."
		action={{ label: 'Open log', onclick: () => {} }}
		ondismiss={() => {}}
	/>
	<Toast
		kind="info"
		icon={Download}
		title="Repo exported: salah-daily-tracker.zip"
		detail="23 files · 184 KB"
		action={{ label: 'Show', onclick: () => {} }}
		ondismiss={() => {}}
	/>
</div>
