<!-- From the environment: typing one here overrides it. -->
<script lang="ts">
	import SecretField from '$lib/ds/admin/SecretField.svelte';
</script>

<div class="w-full max-w-100">
	<SecretField label="Password" name="password" set status="From the environment (SMTP_PASSWORD). Type one to save it here instead." />
</div>
