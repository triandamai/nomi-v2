<script lang="ts">
	import type { Component } from 'svelte';
	import { ChevronDown, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Design System / Nomi → Chips. model, option and agent pick something from a menu; provision
	// names what a project was given (a git repo, a database) and isn't interactive.
	type Props = {
		kind: 'model' | 'option' | 'agent' | 'provision';
		label: string;
		icon?: Component<LucideProps>;
		agent?: CrewId;
		expanded?: boolean;
		onclick?: (event: MouseEvent) => void;
	};
	let { kind, label, icon, agent, expanded, onclick }: Props = $props();
</script>

{#if kind === 'provision'}
	<span class="inline-flex h-6 items-center gap-1.5 rounded-sm bg-surface-2 px-2">
		{#if icon}<span class="inline-flex text-text-3"><Icon {icon} size={12} /></span>{/if}
		<span class="font-mono text-label text-text-2">{label}</span>
	</span>
{:else}
	<button
		type="button"
		{onclick}
		aria-haspopup="menu"
		aria-expanded={expanded}
		class={[
			'inline-flex h-7.5 shrink-0 cursor-pointer items-center gap-1.5 rounded-sm bg-surface-2 outline-offset-1 hover:bg-border pointer-coarse:min-h-11',
			kind === 'agent' ? 'pr-2 pl-1.5' : 'px-2',
		]}
	>
		{#if kind === 'agent' && agent}<Mascot {agent} size={20} />{/if}
		{#if kind === 'option' && icon}<span class="inline-flex text-text-3"><Icon {icon} size={14} /></span>{/if}
		<span class={['text-text-2', kind === 'model' && 'font-mono text-caption', kind === 'option' && 'text-caption', kind === 'agent' && 'text-ui']}>{label}</span>
		<span class="inline-flex text-text-3"><Icon icon={ChevronDown} size={12} /></span>
	</button>
{/if}
