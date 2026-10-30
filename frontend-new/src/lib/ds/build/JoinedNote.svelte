<script lang="ts">
	import { UserPlus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Build together → Joined note: where someone joined the chat, what they can do, and who
	// publishes.
	type Props = { name: string; role: 'build' | 'prompt' | 'view'; publisher?: string };
	let { name, role, publisher }: Props = $props();

	const can = $derived(role === 'build' ? m.role_build() : role === 'prompt' ? m.role_prompt() : m.role_view());
	const label = $derived([m.together_joined({ name }), can.toLowerCase(), publisher ? m.together_publishes({ name: publisher }) : ''].filter(Boolean).join(' · '));
</script>

<div role="separator" aria-label={label} class="flex w-full items-center gap-2.5">
	<span class="h-px flex-1 bg-border"></span>
	<span aria-hidden="true" class="inline-flex items-center gap-2.5 font-mono text-label text-text-3"><Icon icon={UserPlus} size={12} />{label}</span>
	<span class="h-px flex-1 bg-border"></span>
</div>
