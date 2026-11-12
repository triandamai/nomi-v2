<script lang="ts" module>
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	export type ImportFindings = {
		/** The framework and how it builds. */
		app: string;
		tests: string;
		data: string;
		/** What's missing from the repo, said as a warning. */
		secrets?: string;
	};
	export type SetupLine = { icon: Component<LucideProps>; text: string };
</script>

<script lang="ts">
	import { Rocket } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import Button from '../actions/Button.svelte';
	import SectionLabel from '../status/SectionLabel.svelte';

	// Design System / Projects → Project / Import Review (TcRMc): what Koda found in an imported
	// repo, before anything runs. 380 wide on the page colour (border inside, 20 in, 16 apart):
	// Koda's line with the files read and the time it took, what it found (12px rows under lines
	// drawn inside: a 64 key, the value at 1.4; missing secrets in warning), what it will set up, a
	// field for each missing secret (mono label, a 34 field, mono), then Set up & deploy (mint, the
	// rest of the row) and Add later.
	type Props = {
		repo: string;
		/** "48 files · 00:41". */
		meta: string;
		found: ImportFindings;
		setup: SetupLine[];
		/** The secrets the repo needs that aren't in it. */
		secrets?: string[];
		values?: Record<string, string>;
		ondeploy: (values: Record<string, string>) => void;
		onlater?: () => void;
	};
	let { repo, meta, found, setup, secrets = [], values = $bindable({}), ondeploy, onlater }: Props = $props();

	const id = $props.id();
	const rows = $derived(
		[
			{ key: m.projects_review_app(), value: found.app, warn: false },
			{ key: m.projects_review_tests(), value: found.tests, warn: false },
			{ key: m.projects_review_data(), value: found.data, warn: false },
			found.secrets ? { key: m.projects_review_secrets(), value: found.secrets, warn: true } : null,
		].filter((row) => row !== null),
	);

	function deploy() {
		ondeploy(Object.fromEntries(secrets.map((name) => [name, values[name] ?? ''])));
	}
</script>

<section aria-labelledby="{id}-title" class="flex w-95 max-w-full flex-col gap-4 rounded-md bg-bg p-5 outline -outline-offset-1 outline-border">
	<div class="flex items-center gap-2.5">
		<Mascot agent="koda" size={20} />
		<h2 id="{id}-title" class="min-w-0 flex-1 truncate text-ui font-medium text-text">{m.projects_review_title({ repo })}</h2>
		<span class="shrink-0 font-mono text-label text-text-3">{meta}</span>
	</div>
	<dl class="flex flex-col">
		{#each rows as row (row.key)}
			<div class="flex gap-3 border-b border-border pt-2 pb-1.75">
				<dt class="w-16 shrink-0 text-caption text-text-3">{row.key}</dt>
				<dd class={['min-w-0 flex-1 text-caption leading-cozy', row.warn ? 'text-warning' : 'text-text-2']}>{row.value}</dd>
			</div>
		{/each}
	</dl>
	<SectionLabel label={m.projects_review_setup()} />
	<ul class="flex flex-col gap-2 rounded-sm bg-bg p-3">
		{#each setup as line (line.text)}
			<li class="flex gap-2.5">
				<span class="inline-flex shrink-0 text-text-2"><Icon icon={line.icon} size={14} /></span>
				<span class="min-w-0 flex-1 text-caption leading-cozy text-text-2">{line.text}</span>
			</li>
		{/each}
	</ul>
	{#each secrets as name (name)}
		<div class="flex flex-col gap-1.5">
			<label for="{id}-{name}" class="font-mono text-caption font-medium text-text">{name}</label>
			<span class="flex h-8.5 items-center rounded-sm border border-border-strong bg-surface px-1.75 focus-within:border-accent focus-within:shadow-focus">
				<input
					id="{id}-{name}"
					bind:value={() => values[name] ?? '', (next) => (values = { ...values, [name]: next })}
					placeholder={m.projects_review_paste()}
					autocomplete="off"
					spellcheck="false"
					class="min-w-0 flex-1 bg-transparent font-mono text-ui text-text caret-accent outline-none placeholder:text-text-3"
				/>
			</span>
		</div>
	{/each}
	<div class="flex items-center gap-2">
		<span class="flex min-w-0 flex-1 *:flex-1"><Button label={m.projects_review_deploy()} iconEnd={Rocket} onclick={deploy} /></span>
		{#if onlater}<Button variant="secondary" label={m.projects_review_later()} onclick={onlater} />{/if}
	</div>
</section>
