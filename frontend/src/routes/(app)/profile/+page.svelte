<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { deserialize, enhance } from '$app/forms';
	import Avatar from '$lib/components/m3/Avatar.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let displayName = $state(data.profile?.display_name ?? '');
	let username = $state(data.profile?.username ?? '');
	let avatarUrl = $state(data.profile?.avatar_url ?? '');
	let uploading = $state(false);
	let uploadError = $state<string | null>(null);
	let fileInput: HTMLInputElement | undefined = $state();

	async function handleFileSelected(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		uploading = true;
		uploadError = null;

		const requestBody = new FormData();
		requestBody.set('content_type', file.type);
		const response = await fetch('?/requestAvatarUploadUrl', { method: 'POST', body: requestBody });
		const result = deserialize(await response.text());

		if (result.type !== 'success' || !result.data?.uploadUrl) {
			uploading = false;
			uploadError =
				(result.type === 'failure' && (result.data?.error as string)) || m.profile_prepare_failed();
			input.value = '';
			return;
		}

		const uploadResponse = await fetch(result.data.uploadUrl as string, {
			method: 'PUT',
			body: file,
			headers: { 'Content-Type': file.type },
		});
		uploading = false;
		input.value = '';

		if (!uploadResponse.ok) {
			uploadError = m.profile_upload_failed();
			return;
		}

		avatarUrl = result.data.publicUrl as string;
	}
</script>

<div class="h-full overflow-y-auto px-4 py-8 md:px-10">
	<div class="max-w-lg">
		<h1 class="md-display-small" style="color: var(--md-sys-color-on-surface)">{m.profile_title()}</h1>

		{#if form?.error}
			<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}
		{#if uploadError}
			<p class="md-body-medium mt-2" style="color: var(--md-sys-color-error)">{uploadError}</p>
		{/if}

		<div class="mt-6 flex items-center gap-4">
			<Avatar name={displayName || data.profile?.email || '?'} avatarUrl={avatarUrl || null} size={64} />
			<div>
				<input
					bind:this={fileInput}
					type="file"
					accept="image/png,image/jpeg,image/webp,image/gif"
					class="hidden"
					onchange={handleFileSelected}
				/>
				<Button type="button" variant="outlined" onclick={() => fileInput?.click()} disabled={uploading}>
					{uploading ? m.profile_uploading() : m.profile_change_photo()}
				</Button>
			</div>
		</div>

		<form
		method="POST"
		action="?/updateProfile"
		use:enhance={() => {
			return async ({ update }) => {
				await update({ reset: false });
			};
		}}
		class="mt-6 flex flex-col gap-3"
	>
			<input type="hidden" name="avatar_url" value={avatarUrl} />
			<TextField id="display_name" name="display_name" label={m.profile_display_name()} bind:value={displayName} />
			<TextField id="username" name="username" label={m.profile_username()} bind:value={username} />
			<Button type="submit" variant="filled" class="w-fit">{m.common_save()}</Button>
		</form>
	</div>
</div>
