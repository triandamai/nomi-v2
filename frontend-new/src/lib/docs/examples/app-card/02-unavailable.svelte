<!-- Google Workspace where this Nomi hasn’t set it up. -->
<script lang="ts">
	import { Mail } from '@lucide/svelte';
	import AppCard from '$lib/ds/settings/AppCard.svelte';
</script>

<div class="w-96 max-w-full">
	<AppCard icon={Mail} name="Google Workspace" caption="Mail, documents, sheets, files and your calendar" agent="tara" forLabel="for Tara" access="read · write" add="Add Google Workspace" unavailable="not set up" />
</div>
