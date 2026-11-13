<script lang="ts">
	import { GitFork } from '@lucide/svelte';
	import ProjectTopBar from '$lib/ds/projects/ProjectTopBar.svelte';
	import ProjectPageHeader from '$lib/ds/projects/ProjectPageHeader.svelte';
	import ProjectComposer from '$lib/ds/projects/ProjectComposer.svelte';
	import ProjectToolbar from '$lib/ds/projects/ProjectToolbar.svelte';
	import ProjectSectionHead from '$lib/ds/projects/ProjectSectionHead.svelte';
	import RemixCard from '$lib/ds/projects/RemixCard.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	// Design System / Projects → Page parts & grid (RexQ5): the page's top bar, header and
	// composer, then the toolbar, a section head and a remix card.
	const noop = () => {};
	const workspaces = [
		{ id: 'w1', name: 'Mario', tone: 'plain' as const, meta: 'only you · 1 project', tag: 'Personal', chosen: false },
		{ id: 'w2', name: "Trian's space", tone: 'teal' as const, meta: 'Admin · 3 members · 4 projects', chosen: true },
	];
	const members = [
		{ initials: 'T', name: 'Trian', tone: 'teal' as const },
		{ initials: 'MP', name: 'Mario Prasetyo', tone: 'plain' as const },
		{ initials: 'SR', name: 'Siti Rahma', tone: 'orange' as const },
	];
	const rooms: [room: string, paid: boolean][] = [
		['Room 1', true],
		['Room 2', false],
		['Room 3', true],
	];
</script>

<GallerySection
	title="Page parts & grid"
	label="Page parts"
	description="One grid: 384px columns, 16px gaps. Two columns (784) beside the panel, three (1184) when it's closed. Header and composer are always two columns; toolbar, project grid and remix row span the grid. Nothing stretches to the window."
>
	<GalleryBoard title="Top bar & header" note="workspace crumb · members · invite · title · composer" inline>
		<div class="w-316"><ProjectTopBar {workspaces} onchoose={noop} oncreate={noop} {members} oninvite={noop} /></div>
		<div class="w-196"><ProjectPageHeader /></div>
		<!-- The pen draws both composers in a 1600-wide row and clips the second at the board's edge. -->
		<div class="-mr-5.75 overflow-hidden">
			<div class="flex w-400 gap-8">
				<div class="w-196 shrink-0"><ProjectComposer domain="trian.space" onbuild={noop} onimport={noop} /></div>
				<div class="w-196 shrink-0">
					<ProjectComposer value="Order tracker for my kiloan laundry: customers, weight, pickup date, paid or not" domain="trian.space" database="for orders" onbuild={noop} onimport={noop} />
				</div>
			</div>
		</div>
	</GalleryBoard>
	<GalleryBoard title="Toolbar & sections" note="filter tabs · search · sort · section head · remix card" inline>
		<div class="w-196"><ProjectToolbar counts={{ all: 4, live: 2, building: 1, attention: 1 }} /></div>
		<div class="w-196">
			<ProjectSectionHead
				icon={GitFork}
				title="Remix from the community"
				description="Start from an app someone already built. You get your own copy — their data stays theirs."
				link={{ label: 'Browse showcase', href: '#showcase' }}
			/>
		</div>
		<div class="w-96">
			<RemixCard name="Kos Melati rent tracker" maker="Bu Sari · Yogyakarta · 214 remixes" crew={['dana', 'kala']} tone="dana" onremix={noop}>
				{#snippet preview()}
					<div class="flex h-25 flex-1 flex-col gap-1.75 bg-surface px-3.5 py-2.5">
						{#each rooms as [room, paid] (room)}
							<p class="flex justify-between text-micro text-text">{room} <span class={['font-mono text-nano', paid ? 'text-accent' : 'text-text-3']}>{paid ? 'paid' : 'due'}</span></p>
						{/each}
					</div>
				{/snippet}
			</RemixCard>
		</div>
	</GalleryBoard>
</GallerySection>
