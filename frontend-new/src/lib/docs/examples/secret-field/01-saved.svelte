<!-- Saved: it says so, never shows it, and offers to forget it. -->
<script lang="ts">
	import SecretField from '$lib/ds/admin/SecretField.svelte';
</script>

<div class="w-full max-w-100">
	<SecretField label="API key" name="api_key" set status="Saved here. Leave empty to keep it." forget={{ name: 'clear_api_key', label: 'Forget the saved key' }} />
</div>
