<!-- Show a toast: call toasts.show() from anywhere; the region stacks them at the bottom right. -->
<script lang="ts">
	import { Download } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import ToastRegion from '$lib/ds/overlays/ToastRegion.svelte';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
</script>

<Button
	label="Export repo"
	variant="secondary"
	iconStart={Download}
	onclick={() =>
		toasts.show({
			kind: 'info',
			icon: Download,
			title: 'Repo exported: salah-daily-tracker.zip',
			detail: '23 files · 184 KB',
			action: { label: 'Show', onclick: () => {} },
		})}
/>
<ToastRegion />
