<!-- Reminders, empty: what is missing, who can help, and the one thing to do. -->
<script lang="ts">
	import { Bell } from '@lucide/svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
</script>

<div class="w-full max-w-80">
	<EmptyState
		icon={Bell}
		title="Reminders"
		count={0}
		body="Nothing scheduled today. Ask Kala, or add one yourself."
		action={{ label: 'Add reminder', href: '#reminders' }}
	/>
</div>
