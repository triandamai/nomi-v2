<!-- Standalone and inline: on its own line with an arrow, or carrying on a sentence. -->
<script lang="ts">
	import Link from '$lib/ds/actions/Link.svelte';
</script>

<div class="flex w-full max-w-120 flex-col items-start gap-4">
	<Link href="/settings/plan" label="Upgrade to Pro" />
	<p class="text-ui leading-normal text-text-2">
		Your DeepSeek account is out of credits for this request. Add credits with DeepSeek, or
		<Link href="/settings/credits" label="switch to Nomi credits" variant="inline" /> instead.
	</p>
</div>
