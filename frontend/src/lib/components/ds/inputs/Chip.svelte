<script lang="ts">
	import type { Component } from 'svelte';
	import { ChevronDown, type LucideProps } from '@lucide/svelte';
	import M3Chip from '../../m3/Chip.svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Twin of frontend-new's Chip. model, option and agent are M3 assist chips that open a menu;
	// provision names what a project was given (a git repo, a database) and isn't interactive.
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
	<span class="provision">
		{#if icon}<Icon {icon} size={14} />{/if}
		<span class="mono">{label}</span>
	</span>
{:else}
	<M3Chip {onclick} aria-haspopup="menu" aria-expanded={expanded}>
		{#snippet leading()}
			{#if kind === 'agent' && agent}<Mascot {agent} size={20} />{:else if kind === 'option' && icon}<Icon {icon} size={18} />{/if}
		{/snippet}
		<span class:mono={kind === 'model'}>{label}</span>
		<Icon icon={ChevronDown} size={18} />
	</M3Chip>
{/if}

<style>
	.provision {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 28px;
		padding: 0 10px;
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.mono {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-medium-size);
	}
</style>
