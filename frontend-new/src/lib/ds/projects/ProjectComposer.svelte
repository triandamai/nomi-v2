<script lang="ts">
	import { ArrowRight, ArrowUp, Database, GitBranch, Globe } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import GithubMark from '../foundations/GithubMark.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import Button from '../actions/Button.svelte';
	import PhonePill from '../actions/PhonePill.svelte';

	// Design System / Projects → Project / Composer (MDdO8) and its typing state (PV1lT): where a
	// project starts. 784 wide (two grid columns), 98 tall: Koda and the description (15px, 14 / 16
	// in), then what the project gets (24px chips: a git repo, its *.domain, a database) and Import
	// from GitHub and Start building, under a line drawn inside. While there is something written
	// (or focus is in it) the stroke turns accent. When Koda says what the database is for, its chip
	// lights up (accent stroke and icon) with the reason in place of "optional".
	// Enter starts building, Shift+Enter a new line; the field grows with what's written.
	// `phone` is Mobile / New Project Sheet's (Flow / Mobile Projects — New project): the box on the
	// page's ground with 14 corners, 14 in and 12 apart, the text at 16, the chips round and 28 tall,
	// wrapping (the database's in accent once Koda says what it's for), then Import from GitHub and
	// Start building as an inverse pill, outside the box.
	type Props = {
		value?: string;
		/** The workspace's domain: chips read "*.trian.space". Empty where none is set (no chip). */
		domain: string;
		/** What Koda says the database is for ("for orders"); unset, it's optional. */
		database?: string;
		onbuild: (description: string) => void;
		onimport?: () => void;
		phone?: boolean;
	};
	let { value = $bindable(''), domain, database, onbuild, onimport, phone = false }: Props = $props();

	let field: HTMLTextAreaElement | undefined = $state();

	function build(event?: SubmitEvent) {
		event?.preventDefault();
		const description = value.trim();
		if (!description) return field?.focus();
		onbuild(description);
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key !== 'Enter' || event.shiftKey || event.isComposing) return;
		event.preventDefault();
		build();
	}

	/** The field grows with what's written, up to five lines, then scrolls. */
	function grow(element: HTMLTextAreaElement) {
		$effect(() => {
			void value;
			element.style.height = 'auto';
			element.style.height = `${Math.min(element.scrollHeight, 95)}px`;
		});
	}
</script>

{#if phone}
	<form aria-label={m.projects_composer_label()} onsubmit={build} class="flex w-full flex-col gap-3.5">
		<div class={['flex flex-col gap-3 rounded-xl bg-bg p-3.5 outline -outline-offset-1 focus-within:outline-focus', value ? 'outline-focus' : 'outline-border-strong']}>
			<textarea
				bind:this={field}
				bind:value
				use:grow
				{onkeydown}
				rows="2"
				aria-label={m.projects_composer_label()}
				placeholder={m.projects_composer_placeholder()}
				class="min-w-0 resize-none bg-transparent text-phone text-text caret-accent outline-none placeholder:text-text-3"
			></textarea>
			<ul aria-label={m.projects_provisions()} class="flex flex-wrap gap-1.5">
				<li class="inline-flex h-7 items-center gap-1.25 rounded-full border border-border-strong px-2.25">
					<span class="inline-flex text-text-3"><Icon icon={GitBranch} size={13} /></span>
					<span class="font-mono text-label text-text-2">{m.projects_provision_repo()}</span>
				</li>
				{#if domain}
					<li class="inline-flex h-7 items-center gap-1.25 rounded-full border border-border-strong px-2.25">
						<span class="inline-flex text-text-3"><Icon icon={Globe} size={13} /></span>
						<span class="font-mono text-label text-text-2">*.{domain}</span>
					</li>
				{/if}
				<li class={['inline-flex h-7 items-center gap-1.25 rounded-full border px-2.25', database ? 'border-accent bg-accent-bg' : 'border-border-strong']}>
					<span class={['inline-flex', database ? 'text-accent' : 'text-text-3']}><Icon icon={Database} size={13} /></span>
					<span class={['font-mono text-label', database ? 'text-accent' : 'text-text-2']}>{m.projects_provision_database({ note: database ?? m.projects_provision_optional() })}</span>
				</li>
			</ul>
		</div>
		<div class="flex items-center gap-2.5">
			{#if onimport}
				<button type="button" onclick={onimport} class="inline-flex h-11 cursor-pointer items-center gap-1.5 px-1 text-body text-text-2 outline-offset-2">
					<span class="inline-flex"><Icon icon={GithubMark} size={16} /></span>{m.projects_import_github()}
				</button>
			{/if}
			<span class="flex-1"></span>
			<PhonePill type="submit" variant="inverse" label={m.projects_start_building()} icon={ArrowUp} />
		</div>
	</form>
{:else}
<form
	aria-label={m.projects_composer_label()}
	onsubmit={build}
	class={[
		'flex w-196 max-w-full flex-col rounded-md bg-surface outline -outline-offset-1 focus-within:outline-accent',
		value ? 'outline-accent' : 'outline-border-strong',
	]}
>
	<div class="flex items-center gap-3 px-4 py-3.5">
		<Mascot agent="koda" size={20} live />
		<textarea
			bind:this={field}
			bind:value
			use:grow
			{onkeydown}
			rows="1"
			aria-label={m.projects_composer_label()}
			placeholder={m.projects_composer_placeholder()}
			class="min-w-0 flex-1 resize-none bg-transparent text-body text-text caret-accent outline-none placeholder:text-text-3"
		></textarea>
	</div>
	<div class="flex items-center gap-2 border-t border-border pt-2.25 pr-3 pb-2.5 pl-4">
		<ul aria-label={m.projects_provisions()} class="flex min-w-0 items-center gap-2">
			<li class="inline-flex h-6 shrink-0 items-center gap-1.5 rounded-sm bg-surface-2 px-2">
				<span class="inline-flex text-text-3"><Icon icon={GitBranch} size={12} /></span>
				<span class="font-mono text-label text-text-2">{m.projects_provision_repo()}</span>
			</li>
			{#if domain}
				<li class="inline-flex h-6 shrink-0 items-center gap-1.5 rounded-sm bg-surface-2 px-2">
					<span class="inline-flex text-text-3"><Icon icon={Globe} size={12} /></span>
					<span class="font-mono text-label text-text-2">*.{domain}</span>
				</li>
			{/if}
			<li class={['inline-flex h-6 shrink-0 items-center gap-1.5 rounded-sm bg-surface-2 px-2', database && 'outline -outline-offset-1 outline-accent']}>
				<span class={['inline-flex', database ? 'text-accent' : 'text-text-3']}><Icon icon={Database} size={12} /></span>
				<span class={['font-mono text-label', database ? 'text-text' : 'text-text-2']}>{m.projects_provision_database({ note: database ?? m.projects_provision_optional() })}</span>
			</li>
		</ul>
		<span class="ml-auto flex shrink-0 items-center gap-2">
			{#if onimport}
				<button
					type="button"
					onclick={onimport}
					class="inline-flex h-7.5 cursor-pointer items-center gap-1.5 rounded-sm px-2.5 text-ui whitespace-nowrap text-text-2 outline-offset-0 hover:bg-surface-2 hover:text-text focus-visible:outline-3 pointer-coarse:min-h-11"
				>
					<span class="inline-flex text-text-3"><Icon icon={GithubMark} size={14} /></span>
					{m.projects_import_github()}
				</button>
			{/if}
			<Button type="submit" label={m.projects_start_building()} iconEnd={ArrowRight} />
		</span>
	</div>
</form>
{/if}
