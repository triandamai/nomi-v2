<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<div class="flex h-full flex-col items-center justify-center gap-4 text-center">
	<h1 class="text-3xl font-semibold text-neutral-900">Hi, {data.userEmail}!</h1>
	<p class="text-lg text-neutral-500">How can I assist you today?</p>
	<form method="POST" action="?/newChat" use:enhance>
		<button
			type="submit"
			class="rounded-lg bg-neutral-900 px-6 py-3 font-medium text-white hover:bg-neutral-800"
		>
			New Chat
		</button>
	</form>
</div>
