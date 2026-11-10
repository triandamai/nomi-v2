<!-- Sign in: the form in its 380 column, the language switch on top, the terms and status below. The brand panel joins from 1024 wide. -->
<script lang="ts">
	import { ArrowRight, LockKeyhole, Mail } from '@lucide/svelte';
	import AuthLayout from '$lib/ds/auth/AuthLayout.svelte';
	import AuthButton from '$lib/ds/auth/AuthButton.svelte';
	import AuthField from '$lib/ds/auth/AuthField.svelte';
	import LanguageSwitch from '$lib/ds/auth/LanguageSwitch.svelte';

	let language = $state('en');
</script>

<div class="h-215 w-full overflow-hidden rounded-md outline -outline-offset-1 outline-border">
	<AuthLayout version="0.0.75" year={2026} inline>
		{#snippet top()}
			<LanguageSwitch
				value={language}
				options={[
					{ value: 'en', label: 'English' },
					{ value: 'id', label: 'Bahasa Indonesia' },
				]}
				onchange={(value) => (language = value)}
			/>
		{/snippet}
		<div class="flex flex-col gap-1.5">
			<h2 class="text-display font-medium text-text">Sign in to Nomi</h2>
			<p class="text-body leading-normal text-text-2">Welcome back. Your chats, memory and projects are where you left them.</p>
		</div>
		<AuthField label="Email" type="email" icon={Mail} placeholder="you@example.com" />
		<AuthField label="Password" type="password" icon={LockKeyhole} />
		<AuthButton label="Sign in" iconEnd={ArrowRight} type="button" />
	</AuthLayout>
</div>
