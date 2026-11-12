<!-- Variants: the one accent action, working, and a bordered second way on. -->
<script lang="ts">
	import { ArrowRight } from '@lucide/svelte';
	import AuthButton from '$lib/ds/auth/AuthButton.svelte';
</script>

<div class="flex w-full max-w-95 flex-col gap-3">
	<AuthButton label="Sign in" iconEnd={ArrowRight} type="button" />
	<AuthButton label="Sign in" iconEnd={ArrowRight} type="button" loading />
	<AuthButton label="I have an account" variant="secondary" href="#sign-in" />
</div>
