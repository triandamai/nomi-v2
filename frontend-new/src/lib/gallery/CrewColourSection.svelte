<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import { CREW } from '$lib/ds/agents/crew';

	const useFor = [
		'Cards and tiles that are a crew member (crew header, selected member). Suggestions and template cards tint only on hover',
		'Who an item is from (inbox source tile, today panels, suggestions)',
		'Per-member data (usage bars, breakdowns)',
		'App identity tiles on projects',
	];
	const neverFor = [
		"Buttons, links, focus or live state — that's nm-accent",
		'Navigation, settings rows, forms, tables',
		'Errors and warnings — keep danger / warning',
		'Large page backgrounds inside the app',
	];
</script>

<!--
	As the pen draws it (Section / Crew colour): one row of ten cards, each in the member's own
	background with its 20px mascot, a 14px dot in its colour, and its name and tokens in its ink.
	All ten share the row equally (138px at the pen's width); the longest token
	(crew-personality-ink) runs into its card's padding on one line, as in the pen.
-->
<!-- Its own head in the pen: a 22px title with the line under it, 720 wide, rather than the usual
     Section Head. -->
<section id="crew-colour" class="flex flex-col gap-6" aria-label="Crew colour">
	<header class="flex flex-col gap-1.5">
		<h2 class="crew-title font-semibold text-text">Crew colour</h2>
		<p class="w-180 max-w-full text-phone-sm leading-normal text-text-2">
			Each crew member owns one colour. Use it only on surfaces that belong to that member. App chrome stays neutral; nm-accent stays the only action
			and live colour.
		</p>
	</header>
	<div class="flex gap-2.5 overflow-x-auto">
		{#each CREW as member (member.id)}
			<div
				data-crew={member.id}
				class={[
					'flex w-0 min-w-34.5 flex-1 flex-col gap-2.5 rounded-lg p-3.5',
					member.id === 'nomi' && 'bg-crew-nomi-bg text-crew-nomi-ink',
					member.id === 'koda' && 'bg-crew-koda-bg text-crew-koda-ink',
					member.id === 'dana' && 'bg-crew-dana-bg text-crew-dana-ink',
					member.id === 'kala' && 'bg-crew-kala-bg text-crew-kala-ink',
					member.id === 'rena' && 'bg-crew-rena-bg text-crew-rena-ink',
					member.id === 'tara' && 'bg-crew-tara-bg text-crew-tara-ink',
					member.id === 'maya' && 'bg-crew-maya-bg text-crew-maya-ink',
					member.id === 'raga' && 'bg-crew-raga-bg text-crew-raga-ink',
					member.id === 'personality' && 'bg-crew-personality-bg text-crew-personality-ink',
					member.id === 'supervisor' && 'bg-crew-supervisor-bg text-crew-supervisor-ink',
				]}
			>
				<span class="flex items-center gap-2">
					<Mascot agent={member.id} size={20} />
					<span
						data-part="dot"
						class={[
							'size-3.5 shrink-0 rounded-full',
							member.id === 'nomi' && 'bg-crew-nomi',
							member.id === 'koda' && 'bg-crew-koda',
							member.id === 'dana' && 'bg-crew-dana',
							member.id === 'kala' && 'bg-crew-kala',
							member.id === 'rena' && 'bg-crew-rena',
							member.id === 'tara' && 'bg-crew-tara',
							member.id === 'maya' && 'bg-crew-maya',
							member.id === 'raga' && 'bg-crew-raga',
							member.id === 'personality' && 'bg-crew-personality',
							member.id === 'supervisor' && 'bg-crew-supervisor',
						]}
					></span>
				</span>
				<!-- 14px / 600 as drawn: the phone's 14px size, the sheet's one card title. -->
				<p data-part="name" class="text-phone-sm font-semibold">{member.name}</p>
				<span class="flex flex-col gap-0.5">
					{#each [`crew-${member.id}`, `crew-${member.id}-bg`, `crew-${member.id}-ink`] as token (token)}
						<span data-part="token" class="font-mono text-micro whitespace-nowrap">{token}</span>
					{/each}
				</span>
			</div>
		{/each}
	</div>
	<div class="grid items-start gap-6 md:grid-cols-2">
		{#each [{ title: 'Use it for', items: useFor }, { title: 'Never for', items: neverFor }] as list (list.title)}
			<div data-rules class="flex flex-col gap-2 rounded-lg p-4 outline -outline-offset-1 outline-border">
				<p class="rules-label font-mono text-label text-text-3 uppercase">{list.title}</p>
				<ul class="flex flex-col gap-2 text-ui text-text-2">
					{#each list.items as item (item)}<li>· {item}</li>{/each}
				</ul>
			</div>
		{/each}
	</div>
</section>

<style>
	/* This section's own sizes in the pen: documentation, not app UI. */
	.crew-title {
		font-size: 22px;
		line-height: 29px;
		letter-spacing: -0.4px;
	}
	.rules-label {
		letter-spacing: 1px;
	}
</style>
