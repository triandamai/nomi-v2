<script lang="ts">
	import { Check, Database, Download, GitBranch, KeyRound, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import GithubMark from '../foundations/GithubMark.svelte';
	import Button from '../actions/Button.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import SectionLabel from '../status/SectionLabel.svelte';

	// Build → Export: everything Koda built is yours, on every plan. What's included (the repo, a
	// database dump, the env var names; never their values), and two equal ways out.
	type Props = {
		titleId?: string;
		project: string;
		/** e.g. "9 files · git history · main@4be1c07". */
		repo: string;
		database?: string;
		env?: string;
		onpush: () => void;
		ondownload: () => void;
		onclose: () => void;
	};
	let { titleId, project, repo, database, env, onpush, ondownload, onclose }: Props = $props();

	const items = $derived([
		{ icon: GitBranch, title: m.export_repo(), detail: repo },
		...(database ? [{ icon: Database, title: m.export_db(), detail: database }] : []),
		...(env ? [{ icon: KeyRound, title: m.export_env(), detail: env }] : []),
	]);
</script>

<section class="flex w-120 max-w-full flex-col overflow-hidden rounded-md bg-surface outline -outline-offset-1 outline-border-strong">
	<header class="flex items-center gap-2.5 border-b border-border px-5 pt-4 pb-3.75">
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<h2 id={titleId} class="text-body font-medium text-text">{m.export_title({ project })}</h2>
			<span class="text-ui text-text-3">{m.export_sub({ name: 'Koda' })}</span>
		</span>
		<IconButton icon={X} label={m.action_close()} variant="ghost" onclick={onclose} />
	</header>
	<div class="flex flex-col gap-1 p-5">
		<SectionLabel label={m.export_included()} />
		<ul class="flex flex-col gap-1">
			{#each items as item (item.title)}
				<li class="flex gap-3 border-b border-border pt-2.5 pb-2.25">
					<span class="inline-flex h-4.5 shrink-0 items-center text-text-2"><Icon icon={item.icon} size={14} /></span>
					<span class="flex min-w-0 flex-1 flex-col gap-0.75">
						<span class="text-ui font-medium text-text">{item.title}</span>
						<span class="font-mono text-label break-words text-text-3">{item.detail}</span>
					</span>
					<span class="inline-flex h-4.5 shrink-0 items-center text-accent"><Icon icon={Check} size={14} /></span>
				</li>
			{/each}
		</ul>
		{#if env}<p class="text-ui text-text-3">{m.export_env_note()}</p>{/if}
	</div>
	<footer class="grid grid-cols-2 gap-2 border-t border-border px-5 pt-2.75 pb-4">
		<Button label={m.export_github()} variant="secondary" iconEnd={GithubMark} onclick={onpush} />
		<Button label={m.export_zip()} variant="secondary" iconEnd={Download} onclick={ondownload} />
	</footer>
</section>
