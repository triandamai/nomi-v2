<script lang="ts">
	import { ChevronDown, Plus, UserPlus, Users } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import ThemePickerButton from '../chrome/ThemePickerButton.svelte';

	// Mobile / Projects Header (Flow / Mobile Projects): [6,16,8,20], 6 apart. The top row, 36 tall
	// and 8 apart: the workspace (surface-2, round, its 28 tile with the initial at 12 / 600, the name at 14 /
	// 500 and a chevron) opening the page's workspace sheet; then, at the end, how many people are in
	// it (users, 18) and Invite (36, round, border-strong, user-plus). Under it the large title at 32 /
	// 600 and New (the accent pill, 36 tall, plus and its label at 15 / 600) opening New project.
	type Props = {
		title: string;
		workspace: string;
		/** The people in the workspace; left out when not known. */
		members?: number;
		labels: { workspace: string; members: string; invite: string; new: string };
		onworkspace?: () => void;
		oninvite?: () => void;
		onnew?: () => void;
	};
	let { title, workspace, members, labels, onworkspace, oninvite, onnew }: Props = $props();
</script>

<div class="flex flex-col gap-1.5 pt-1.5 pr-4 pb-2 pl-5">
	<div class="flex h-9 items-center gap-2">
		<button
			type="button"
			aria-label={labels.workspace}
			class="inline-flex h-9 min-w-0 cursor-pointer items-center gap-2 rounded-full bg-surface-2 pr-3 pl-1 outline-offset-2"
			onclick={onworkspace}
		>
			<span aria-hidden="true" class="inline-flex size-7 shrink-0 items-center justify-center rounded-full bg-surface text-caption font-semibold text-text outline -outline-offset-1 outline-border-strong">{workspace.charAt(0).toUpperCase()}</span>
			<span class="truncate text-phone-sm font-medium text-text">{workspace}</span>
			<span class="inline-flex shrink-0 text-text-3"><Icon icon={ChevronDown} size={16} /></span>
		</button>
		<span class="flex-1"></span>
		{#if members !== undefined}
			<span class="inline-flex h-9 items-center gap-1.5 px-2 text-phone-sm text-text-2">
				<Icon icon={Users} size={18} label={labels.members} />{members}
			</span>
		{/if}
		{#if oninvite}
			<button type="button" aria-label={labels.invite} class="inline-flex size-9 shrink-0 cursor-pointer items-center justify-center rounded-full border border-border-strong text-text outline-offset-2" onclick={oninvite}>
				<Icon icon={UserPlus} size={18} />
			</button>
		{/if}
		<ThemePickerButton />
	</div>
	<div class="flex items-center gap-3">
		<h1 class="min-w-0 flex-1 truncate text-phone-large font-semibold tracking-phone-large text-text">{title}</h1>
		{#if onnew}
			<button type="button" class="inline-flex h-9 shrink-0 cursor-pointer items-center gap-1 rounded-full bg-accent pr-3.5 pl-2.5 text-accent-ink outline-offset-2" onclick={onnew}>
				<Icon icon={Plus} size={18} /><span class="text-body font-semibold">{labels.new}</span>
			</button>
		{/if}
	</div>
</div>
