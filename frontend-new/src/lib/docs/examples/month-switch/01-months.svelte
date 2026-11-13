<!-- The current month, and an earlier one. -->
<script lang="ts">
	import MonthSwitch from '$lib/ds/settings/MonthSwitch.svelte';
</script>

<div class="flex flex-col items-start gap-3">
	<MonthSwitch month="October 2026" previous={{ href: '#2026-09', label: 'September 2026' }} nextLabel="No later month" />
	<MonthSwitch month="September 2026" previous={{ href: '#2026-08', label: 'August 2026' }} next={{ href: '#2026-10', label: 'October 2026' }} nextLabel="No later month" />
</div>
