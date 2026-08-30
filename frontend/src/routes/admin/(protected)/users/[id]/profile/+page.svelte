<script lang="ts">
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let open = $state(true);

	$effect(() => {
		// BottomSheet flips `open` to false on its own dismiss affordances (Escape, backdrop
		// click, drag-to-dismiss) — closing this route-driven panel means navigating back.
		if (!open) goto('/admin/users');
	});

	let displayName = $state(data.user.display_name ?? '');
	let username = $state(data.user.username ?? '');
</script>

<BottomSheet bind:open>
	<h2 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Update user</h2>
	<p class="md-body-medium mt-1" style="color: var(--md-sys-color-on-surface-variant)">{data.user.email}</p>

	{#if form?.error}
		<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
	{/if}

	<form
		method="POST"
		action="?/updateUser"
		use:enhance={() => {
			return async ({ update }) => {
				await update({ reset: false });
			};
		}}
		class="mt-6 flex flex-col gap-3"
	>
		<TextField id="display_name" name="display_name" label="Display name" bind:value={displayName} />
		<TextField id="username" name="username" label="Username" bind:value={username} />
		<Button type="submit" variant="filled" class="w-fit">Save</Button>
	</form>
</BottomSheet>
