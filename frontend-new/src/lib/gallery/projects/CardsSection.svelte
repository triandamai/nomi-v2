<script lang="ts">
	import { CalendarCheck, CircleCheck, Database, FileText, GitBranch, Globe, SearchX, Store, X } from '@lucide/svelte';
	import ProjectCard from '$lib/ds/projects/ProjectCard.svelte';
	import ProjectCardLoading from '$lib/ds/projects/ProjectCardLoading.svelte';
	import ProjectMenuPanel from '$lib/ds/projects/ProjectMenuPanel.svelte';
	import { projectMenu } from '$lib/ds/projects/ProjectCardMenu.svelte';
	import SortMenu from '$lib/ds/projects/SortMenu.svelte';
	import StopDialog from '$lib/ds/projects/StopDialog.svelte';
	import DeployDialog from '$lib/ds/projects/DeployDialog.svelte';
	import ProjectMoveDialog from '$lib/ds/projects/ProjectMoveDialog.svelte';
	import WorkspaceSwitcher from '$lib/ds/chrome/WorkspaceSwitcher.svelte';
	import InviteDialog from '$lib/ds/projects/InviteDialog.svelte';
	import FilterTab from '$lib/ds/projects/FilterTab.svelte';
	import ProjectSearch from '$lib/ds/projects/ProjectSearch.svelte';
	import SortButton from '$lib/ds/projects/SortButton.svelte';
	import ProjectEmpty from '$lib/ds/projects/ProjectEmpty.svelte';
	import FirstProject from '$lib/ds/projects/FirstProject.svelte';
	import LoadError from '$lib/ds/projects/LoadError.svelte';
	import GithubConnectDialog from '$lib/ds/projects/GithubConnectDialog.svelte';
	import RepoPickerDialog from '$lib/ds/projects/RepoPickerDialog.svelte';
	import ImportReview from '$lib/ds/projects/ImportReview.svelte';
	import ProjectDetailPanel from '$lib/ds/projects/ProjectDetailPanel.svelte';
	import AskField from '$lib/ds/projects/AskField.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';
	import SalahThumb from './SalahThumb.svelte';

	// Design System / Projects → Cards & overlays (DLQ3G): the card in every state, its menus and
	// confirms, the workspace dialogs, the toolbar's parts with the empty and error states, the
	// GitHub import, and the selected project's panel.
	const noop = () => {};
	const salah = { name: 'salah-daily-tracker', description: 'Daily salah tracker with countdown and streaks', tone: 'nomi' as const };
	const warung = { name: 'warung-kopi-menu', description: 'Menu and pre-order page for Warung Kopi Bu Siti', tone: 'dana' as const };
	const actions = { onopen: noop, onvisit: noop, onrename: noop, onduplicate: noop, onmove: noop, onshare: noop, onexport: noop, onsettings: noop, ontakedown: noop, ondelete: noop };
	const onpick = noop;
</script>

<GallerySection
	title="Cards & overlays"
	label="Projects"
	description="Card: 384 × 192, fixed. Identity on top, a status zone of exactly two lines (status + right meta for step or time; one detail line of up to ~45 characters), then a divided footer with chats/time and one action. Zone tint: neutral, warning when Koda needs you, danger when something broke. Action: inverse when it needs you, secondary to open or resume, ghost to stop or cancel. Selected = 2px nm-text-2 stroke."
>
	<GalleryBoard title="Project cards" note="live · building · stopped · failed · draft · importing · needs answer · down · renaming · loading" inline>
		<div class="flex flex-col gap-5">
			<div class="flex gap-4">
				<ProjectCard {...salah} status="live" url="salah-daily-tracker-0ab8f8.trian.space" footer="2 chats · deploy #15 · 2d ago" href="#salah" {actions} />
				<ProjectCard {...warung} status="building" meta="3 of 6" detail="writing routes/menu/+page.svelte" footer="1 chat · started 08:13" onaction={noop} {actions} />
				<ProjectCard {...warung} status="stopped" meta="3 of 6" detail="nothing deployed · files kept" footer="1 chat · stopped 08:15" onaction={noop} {actions} />
			</div>
			<div class="flex gap-4">
				<ProjectCard
					name="invoice-sinar-jaya"
					description="Invoices and payment tracking for PT Sinar Jaya"
					tone="kala"
					status="failed"
					meta="deploy #6"
					detail="build error in src/lib/pdf.ts · #5 still live"
					footer="3 chats · 5d ago"
					onaction={noop}
					{actions}
				/>
				<ProjectCard name="budget-keluarga" description="Shared family budget with monthly categories" tone="tara" status="draft" detail="not deployed yet · 3 files" footer="1 chat · 3w ago" onaction={noop} {actions} />
				<ProjectCard
					name="kasir-toko-sembako"
					description="Imported from github.com/mario/kasir-toko"
					tone="kala"
					status="importing"
					meta="00:22"
					detail="reading 48 files · finding the build"
					footer="from GitHub · started 09:02"
					onaction={noop}
					{actions}
				/>
			</div>
			<div class="flex gap-4">
				<ProjectCard
					name="laundry-kiloan"
					description="Orders and pickup tracker for Laundry Bu Ani"
					tone="maya"
					status="attention"
					questions={3}
					detail="who uses it · how orders come in · prices"
					footer="1 chat · started 09:30"
					onaction={noop}
					{actions}
				/>
				<ProjectCard {...salah} status="down" meta="4 min" detail="502 for visitors · last OK 09:10" footer="2 chats · checked 09:14" onaction={noop} {actions} />
				<ProjectCard {...salah} name="salah-tracker" status="live" url="salah-daily-tracker-0ab8f8.trian.space" footer="2 chats · deploy #15 · 2d ago" href="#salah" renaming onrename={noop} oncancelrename={noop} {actions} />
			</div>
			<div class="flex gap-4"><ProjectCardLoading /></div>
		</div>
	</GalleryBoard>

	<GalleryBoard title="Menus & confirms" note="card ⋯ menu · sort · stop · deploy · move to workspace" inline>
		<div class="flex items-start gap-8">
			<ProjectMenuPanel title="salah-daily-tracker" items={projectMenu(actions, true)} />
			<SortMenu value="updated" onchoose={noop} />
			<StopDialog inline name="warung-kopi-menu" done={3} next={4} onstop={noop} />
			<DeployDialog
				inline
				name="budget-keluarga"
				files={3}
				url="budget-keluarga-7c21.trian.space"
				data="none yet · add a database later"
				uses="1 build · 26 of 30 left in Oct"
				ondeploy={noop}
			/>
		</div>
	</GalleryBoard>

	<GalleryBoard title="Workspace dialogs" note="move to workspace · switch workspace · invite to workspace" inline>
		<div class="flex items-start gap-8">
			<ProjectMoveDialog
				inline
				name="salah-daily-tracker"
				chats={2}
				owner="Trian"
				value="w3"
				workspaces={[
					{ id: 'w1', name: 'Mario', meta: 'personal · only you', tone: 'plain', members: 1 },
					{ id: 'w2', name: "Trian's space", meta: 'current · 3 members', tone: 'nomi', members: 3, current: true },
					{ id: 'w3', name: 'Kopi Kita', meta: 'Admin · 5 members', tone: 'tara', members: 5 },
				]}
				onmove={noop}
			/>
			<WorkspaceSwitcher
				inline
				current="Trian's space"
				workspaces={[
					{ id: 'w1', name: 'Mario', meta: 'only you · 1 project', tone: 'plain', tag: 'Personal', chosen: false },
					{ id: 'w2', name: "Trian's space", meta: 'Admin · 3 members · 4 projects', tone: 'teal', chosen: true },
					{ id: 'w3', name: 'Kopi Kita', meta: 'invited by Andi · 2 projects', tone: 'amber', chosen: false },
				]}
				onchoose={noop}
				oncreate={noop}
				settingsHref="#settings"
			/>
			<InviteDialog inline workspace="Trian's space" emails={['siti@kopikita.id', 'budi.s@gmail.com']} oninvite={noop} oncopylink={noop} />
		</div>
	</GalleryBoard>

	<GalleryBoard title="Toolbar, empty & error" note="filter tab · search · sort · no match · empty tab · first project · couldn't load" inline>
		<div class="flex flex-col gap-5">
			<div class="flex items-center gap-6">
				<FilterTab label="All" count={4} active />
				<FilterTab label="Live" count={2} />
				<div class="w-55"><ProjectSearch /></div>
				<div class="w-55"><ProjectSearch value="kasir" /></div>
				<SortButton />
			</div>
			<div class="flex gap-6">
				<div class="w-175">
					<ProjectEmpty
						icon={SearchX}
						title="No projects match “kasir”"
						body="Search looks at names, descriptions and URLs in Trian's space."
						action={{ label: 'Clear search', icon: X, onclick: noop }}
					/>
				</div>
				<div class="w-175">
					<ProjectEmpty icon={CircleCheck} title="Nothing needs attention" body="When a deploy fails or Koda is waiting on you, the project shows up here." />
				</div>
			</div>
			<div class="flex items-start gap-6">
				<div class="w-205">
					<FirstProject
						starters={[
							{ icon: CalendarCheck, label: 'Booking page for my barbershop', meta: 'web app · database', onpick },
							{ icon: Store, label: 'Menu and pre-order page for a warung', meta: 'web app · database', onpick },
							{ icon: FileText, label: 'One-page portfolio with a contact form', meta: 'site · no database', onpick },
						]}
					/>
				</div>
				<div class="w-145"><LoadError reason="503 from api.nomi.app · 09:14" onretry={noop} /></div>
			</div>
		</div>
	</GalleryBoard>

	<GalleryBoard title="Import from GitHub" note="connect · pick a repo · what Koda found" inline>
		<div class="flex items-start gap-8">
			<GithubConnectDialog inline oncontinue={noop} />
			<RepoPickerDialog
				inline
				workspace="Trian's space"
				owner="mario"
				value="r1"
				repos={[
					{ id: 'r1', name: 'mario/kasir-toko', meta: 'SvelteKit · private · updated 3d ago' },
					{ id: 'r2', name: 'mario/portfolio-2024', meta: 'Astro · public · 2mo ago' },
					{ id: 'r3', name: 'mario/warung-api', meta: 'Node · private · 1y ago' },
					{ id: 'r4', name: 'kopikita/web', meta: 'not shared with Nomi', grantHref: '#grant' },
				]}
				onimport={noop}
			/>
			<div class="w-95">
				<ImportReview
					repo="kasir-toko"
					meta="48 files · 00:41"
					found={{ app: 'SvelteKit 2 · builds with npm run build', tests: '12 tests · all pass', data: 'Postgres via DATABASE_URL', secrets: "MIDTRANS_KEY and ADMIN_PIN aren't in the repo" }}
					setup={[
						{ icon: Database, text: 'An empty Postgres database, migrated from /migrations' },
						{ icon: Globe, text: 'kasir-toko-3e9a.trian.space' },
						{ icon: GitBranch, text: 'Commits on nomi/main; your main stays untouched' },
					]}
					secrets={['MIDTRANS_KEY', 'ADMIN_PIN']}
					ondeploy={noop}
					onlater={noop}
				/>
			</div>
		</div>
	</GalleryBoard>

	<GalleryBoard title="Project detail" note="selected project panel · ask Koda from the panel" inline>
		<div class="flex items-start gap-8">
			<div class="w-95">
				<ProjectDetailPanel
					name="salah-daily-tracker"
					tone="nomi"
					live="live · deploy #14 · 2d ago"
					workspaceHref="#workspace"
					url="salah-daily-tracker-0ab8f8.trian.space"
					parts={{ code: '23 files · main@a3f9c21', url: '…0ab8f8.trian.space', data: 'localStorage only · no server', deploys: '14 · last built in 41s', members: 'Trian (owner), you' }}
					onexport={noop}
					chats={[
						{ title: 'Build a daily salah tracker', age: '2d', href: '#c1' },
						{ title: 'Add Hijri offset setting', age: '2d', href: '#c2' },
					]}
					onask={noop}
					onclose={noop}
				>
					{#snippet preview()}<SalahThumb />{/snippet}
				</ProjectDetailPanel>
			</div>
			<div class="w-85"><AskField project="salah-daily-tracker" value="Add a Maghrib reminder 10 minutes before" onask={noop} /></div>
		</div>
	</GalleryBoard>
</GallerySection>
