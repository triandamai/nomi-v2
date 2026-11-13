<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Auth / Sign in or create → Divider: two 1px lines with "OR" between them in the mono label,
	// set apart 12 on each side. Between Google and the email form.
</script>

<p class="flex w-full items-center gap-3">
	<span class="h-px min-w-0 flex-1 bg-border" aria-hidden="true"></span>
	<span class="font-mono text-label tracking-label text-text-3 uppercase">{m.auth_or()}</span>
	<span class="h-px min-w-0 flex-1 bg-border" aria-hidden="true"></span>
</p>
