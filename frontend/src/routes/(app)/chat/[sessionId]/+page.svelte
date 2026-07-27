<script lang="ts">
	import { enhance } from '$app/forms';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let messages = $derived.by(() => {
		const base = [...data.messages];
		if (form?.user_message && form?.assistant_message) {
			base.push(form.user_message, form.assistant_message);
		}
		return base;
	});
</script>

<div class="flex h-full flex-col">
	<div class="flex-1 space-y-4 overflow-y-auto px-6 py-6">
		{#each messages as message (message.id)}
			<MessageBubble {message} />
		{/each}
		{#if form?.turnFailed}
			<div class="flex justify-end">
				<div class="max-w-md rounded-2xl bg-neutral-900 px-4 py-2 text-white">{form.sentText}</div>
			</div>
			<p class="text-right text-sm text-neutral-400">No reply yet — try sending again.</p>
		{/if}
		{#if form?.error}
			<p class="text-center text-sm text-red-600">{form.error}</p>
		{/if}
	</div>

	<form method="POST" use:enhance={() => {
		return async ({ update }) => {
			await update({ reset: true });
		};
	}} class="border-t border-neutral-200 bg-white px-6 py-4">
		<div class="flex items-center gap-2 rounded-full border border-neutral-300 px-4 py-2">
			<input
				name="text"
				type="text"
				placeholder="Ask me anything..."
				required
				class="flex-1 border-none bg-transparent outline-none"
			/>
			<button
				type="submit"
				class="rounded-full bg-neutral-900 px-4 py-1.5 text-sm font-medium text-white hover:bg-neutral-800"
			>
				Send
			</button>
		</div>
	</form>
</div>
