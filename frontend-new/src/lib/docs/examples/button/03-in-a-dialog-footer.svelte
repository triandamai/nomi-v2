<!-- In a footer: a quiet way out, Cancel, then the action. Destructive actions use danger. -->
<script lang="ts">
	import { Download, Trash2 } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
</script>

<div class="flex w-full max-w-105 flex-wrap items-center justify-end gap-2 rounded-md bg-surface px-5 py-3 outline -outline-offset-1 outline-border-strong">
	<Button label="Export repo first" variant="ghost" iconStart={Download} />
	<Button label="Cancel" variant="secondary" />
	<Button label="Delete project" variant="danger" iconStart={Trash2} />
</div>
