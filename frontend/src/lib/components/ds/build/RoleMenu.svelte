<script lang="ts">
	import { Check, Eye, Hammer, MessageSquare } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';

	// Build together → Roles: picked when inviting, changeable later in People. Building can
	// publish; prompting asks Koda and the builder publishes; viewing changes nothing.
	type Role = 'build' | 'prompt' | 'view';
	type Props = { value: Role; onchange: (role: Role) => void };
	let { value, onchange }: Props = $props();

	const id = $props.id();
	const ROLES = [
		{ value: 'build' as const, icon: Hammer, title: m.role_build, detail: () => m.role_build_detail({ agent: 'Koda' }) },
		{ value: 'prompt' as const, icon: MessageSquare, title: m.role_prompt, detail: () => m.role_prompt_detail({ agent: 'Koda' }) },
		{ value: 'view' as const, icon: Eye, title: m.role_view, detail: m.role_view_detail },
	];
</script>

<div role="radiogroup" aria-label={m.role_label()} class="flex w-80 max-w-full flex-col gap-0.5 rounded-md bg-surface p-1.5 shadow-float outline -outline-offset-1 outline-border-strong">
	{#each ROLES as role (role.value)}
		{@const chosen = role.value === value}
		<label class={['relative flex cursor-pointer items-center gap-2.5 rounded-sm px-2.5 py-2 hover:bg-surface-2 has-focus-visible:outline-2 has-focus-visible:outline-focus', chosen && 'bg-surface-2']}>
			<input type="radio" name="{id}-role" value={role.value} checked={chosen} onchange={() => onchange(role.value)} class="absolute inset-0 cursor-pointer appearance-none rounded-sm outline-none" />
			<span class={['inline-flex shrink-0', chosen ? 'text-text' : 'text-text-2']}><Icon icon={role.icon} size={14} /></span>
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="text-ui font-medium text-text">{role.title()}</span>
				<span class="text-caption text-text-3">{role.detail()}</span>
			</span>
			{#if chosen}<span class="inline-flex text-accent"><Icon icon={Check} size={14} /></span>{/if}
		</label>
	{/each}
</div>
