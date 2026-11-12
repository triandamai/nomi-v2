<!-- Under the form: the likelier way first in text-2, the other quiet. -->
<script lang="ts">
	import AuthLink from '$lib/ds/auth/AuthLink.svelte';
</script>

<p class="flex flex-wrap gap-x-4">
	<AuthLink label="New to Nomi? Create an account" href="#register" />
	<AuthLink label="Forgot password?" href="#forgot" quiet />
</p>
