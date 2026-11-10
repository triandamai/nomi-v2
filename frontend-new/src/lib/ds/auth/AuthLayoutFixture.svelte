<script lang="ts">
	import AuthLayout from './AuthLayout.svelte';

	let { healthy = true, greeting }: { healthy?: boolean; greeting?: string | false } = $props();
</script>

<AuthLayout version="1.2.3" year={2026} {healthy} {greeting}>
	{#snippet top()}<button type="button">Language here</button>{/snippet}
	<h1>Sign in to Nomi</h1>
	<p>The form</p>
</AuthLayout>
