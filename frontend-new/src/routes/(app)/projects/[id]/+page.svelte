<script lang="ts">
	import { untrack } from 'svelte';
	import { Activity, ArrowDownUp, ChevronLeft, ChevronRight, Coins, Cpu, Database, Download, Globe, History, Hourglass, KeyRound, Link, Lock, MemoryStick, MonitorX, Palette, Pencil, RefreshCw, Rocket, Settings, Trash2, UserMinus, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import ChatView from '$lib/chat/ChatView.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import ProjectWorkspace from '$lib/ds/projects/ProjectWorkspace.svelte';
	import ProjectBar from '$lib/ds/projects/ProjectBar.svelte';
	import ChatPaneHeader from '$lib/ds/projects/ChatPaneHeader.svelte';
	import PaneTabs from '$lib/ds/projects/PaneTabs.svelte';
	import ProjectDialog from '$lib/ds/projects/ProjectDialog.svelte';
	import DeployRow from '$lib/ds/projects/DeployRow.svelte';
	import AddressRow from '$lib/ds/projects/AddressRow.svelte';
	import MonitorStat from '$lib/ds/projects/MonitorStat.svelte';
	import TrafficBars from '$lib/ds/projects/TrafficBars.svelte';
	import ResourceRow from '$lib/ds/projects/ResourceRow.svelte';
	import LogLines from '$lib/ds/projects/LogLines.svelte';
	import TableChip from '$lib/ds/projects/TableChip.svelte';
	import RowsGrid from '$lib/ds/projects/RowsGrid.svelte';
	import SettingRow from '$lib/ds/settings/SettingRow.svelte';
	import PersonRow from '$lib/ds/settings/PersonRow.svelte';
	import SecretRow from '$lib/ds/projects/SecretRow.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Select from '$lib/ds/inputs/Select.svelte';
	import { ago } from '$lib/adapters/time';
	import { maskAddress } from '$lib/settings/connections';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import DnsRecords from '$lib/ds/projects/DnsRecords.svelte';
	import { domainView } from '$lib/projects/domains';
	import Search from '$lib/ds/inputs/Search.svelte';
	import { cellText } from '$lib/projects/database';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import { shiftMonth } from '$lib/admin/format';
	import type { MetricsRange } from '$lib/adapters/api';
	import SubdomainField from '$lib/ds/projects/SubdomainField.svelte';
	import { invalidate } from '$app/navigation';
	import CodeView from '$lib/ds/build/CodeView.svelte';
	import { plainLines } from '$lib/ds/chat/code';
	import PhoneFrame from '$lib/ds/build/PhoneFrame.svelte';
	import PublishButton from '$lib/ds/build/PublishButton.svelte';
	import WorkspaceSheet from '$lib/ds/build/WorkspaceSheet.svelte';
	import BuildShareDialog from '$lib/ds/build/BuildShareDialog.svelte';
	import ExportDialog from '$lib/ds/build/ExportDialog.svelte';
	import Checkbox from '$lib/ds/inputs/Checkbox.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import Terminal from '$lib/ds/build/Terminal.svelte';
	import SandboxWait from '$lib/projects/SandboxWait.svelte';
	import { terminalLines } from '$lib/projects/terminal';
	import Presence from '$lib/ds/build/Presence.svelte';
	import PresenceMenu from '$lib/ds/build/PresenceMenu.svelte';
	import FollowBar from '$lib/ds/build/FollowBar.svelte';
	import PeerCursor from '$lib/ds/build/PeerCursor.svelte';
	import { initials as initialsOf, peerFor, whereLabel } from '$lib/projects/peers';
	import AsksInLine from '$lib/ds/projects/AsksInLine.svelte';
	import PublishRequest from '$lib/ds/build/PublishRequest.svelte';
	import { requestMeta } from '$lib/projects/asks';
	import { phoneView } from '$lib/phone.svelte';
	import { Workspace } from './workspace.svelte';
	import PhoneWorkspace from './PhoneWorkspace.svelte';
	import DesignCanvas from '$lib/ds/design/DesignCanvas.svelte';

	// A project's workspace (Flow / Build — Sign in → live URL o882RC, Building and Live; Workspace
	// tools U16wU, Changes ready → Publish → Publish failed → Published → Deployments): the project
	// bar with Publish, the project's chat where Koda works (the chat itself, as on its own page), and
	// the preview pane, over which a project tool opens as a sheet. The preview shows the live app
	// once there is one, else the project's own files when they're a static site; anything else waits
	// for its runtime. Code lists the project's files. Publish confirms what goes live; while it runs
	// the workspace looks again every few seconds; a failed publish keeps the live deploy up and says
	// why. Deployments lists every publish, live and failed, with Roll back (a publish too). Domain,
	// Monitor, Database, Settings, Share and the terminal come with their sheets (AGENTS.md §9).
	// Below 768 the workspace is the phone's own screen (Mobile Build): both read one `Workspace`,
	// and only one of them is mounted.
	let { data } = $props();

	const ws = new Workspace(() => data);
	const phone = phoneView(untrack(() => Boolean(data.phone)));
	$effect(() => {
		ws.phone = phone.current;
	});

	// `?open=` from a card's menu, the detail panel or search on a file: the desktop's dialog or view.
	$effect(() => {
		const asked = ws.opened;
		if (!asked || phone.current) return;
		ws.opened = null;
		untrack(() => {
			if (asked.open === 'settings') ws.sheet = 'settings';
			else if (asked.open === 'export') void ws.openExport();
			else if (asked.open === 'share' && ws.liveUrl) ws.sharing = true;
			else if (asked.open === 'takedown' && ws.liveUrl) ws.openDialog('takedown');
			else if (asked.open === 'code') {
				ws.view = 'code';
				if (asked.file && ws.project.files.some((one) => one.path === asked.file)) ws.openFile(asked.file);
			}
		});
	});

	// An opened sheet takes the focus, so Escape (and a screen reader) is in it at once.
	let sheetBox = $state<HTMLElement>();
	$effect(() => {
		if (ws.sheet) sheetBox?.querySelector<HTMLElement>('section')?.focus();
	});
	let presenceOpen = $state(false);
	let hereBox = $state<HTMLElement>();
	$effect(() => {
		if (!presenceOpen) return;
		const close = (event: PointerEvent) => {
			if (hereBox && !hereBox.contains(event.target as Node)) presenceOpen = false;
		};
		document.addEventListener('pointerdown', close, true);
		return () => document.removeEventListener('pointerdown', close, true);
	});
	const WHERE = $derived(ws.where);
	const ROLES = $derived(ws.roles);
	const RANGE_NOTE: Record<MetricsRange, () => string> = { '1h': m.monitor_last_hour, '24h': m.monitor_today, '7d': m.monitor_week };
	const trafficLabel = $derived(ws.site ? m.monitor_visits_traffic({ range: RANGE_NOTE[ws.range]() }) : m.monitor_traffic({ range: RANGE_NOTE[ws.range]() }));
	const rowsLabel = (count: number) => (count < 0 ? m.database_rows_unknown() : count === 1 ? m.database_row_one() : m.database_rows({ count }));
	const kilobytes = (bytes: number) => (bytes < 1024 * 1024 ? `${Math.max(1, Math.round(bytes / 1024))} KB` : `${(bytes / (1024 * 1024)).toFixed(1)} MB`);
</script>

<svelte:head><title>{ws.project.name} · {m.app_name()}</title></svelte:head>

{#snippet together()}
	{#if ws.theirs.length || ws.line.length > 1}
		<div class="flex flex-col gap-4">
			{#each ws.theirs as row (row.id)}
				{@const name = row.requested_by_name}
				<div class="flex flex-col items-end gap-1">
					<PublishRequest
						who={{ name, initials: initialsOf(name), peer: peerFor(name) }}
						meta={requestMeta(row, ws.deploy?.unpublished_changes ?? null, ws.time.timeZone)}
						onpublish={() => void ws.answerRequest(row, 'approve')}
						onreview={ws.review}
					/>
					<Button variant="ghost" label={m.publish_request_decline()} aria-label={m.publish_request_decline_of({ name })} onclick={() => void ws.answerRequest(row, 'decline')} />
				</div>
			{/each}
			{#if ws.line.length > 1}
				<AsksInLine asks={ws.line} first={ws.movable >= 0 ? ws.movable : undefined} onfirst={(index) => void ws.letFirst(index)} />
			{/if}
		</div>
	{/if}
{/snippet}

{#snippet here()}
	<div bind:this={hereBox} class="relative shrink-0 max-sm:hidden">
		<button
			type="button"
			aria-haspopup="dialog"
			aria-expanded={presenceOpen}
			aria-label={m.presence_open({ names: ws.peers.map((person) => person.name).join(', ') })}
			onclick={() => (presenceOpen = !presenceOpen)}
			class="inline-flex cursor-pointer items-center rounded-sm outline-offset-2"
		>
			<Presence people={[{ name: ws.myName, initials: ws.faces[0] }, ...ws.peers.map((person) => ({ name: person.name, initials: person.initials, peer: person.peer }))].map((person, index, all) => ({ ...person, online: index === all.length - 1 }))} />
		</button>
		{#if presenceOpen}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div class="absolute top-full right-0 z-20 mt-2" onkeydown={(event) => event.key === 'Escape' && (presenceOpen = false)}>
				<PresenceMenu
					people={[
						...ws.peers.map((person) => ({ name: person.name, initials: person.initials, peer: person.peer, where: whereLabel(person.view, person.path, WHERE) })),
						{ name: m.presence_you({ name: ws.myName }), initials: ws.faces[0], where: whereLabel(ws.view, ws.view === 'code' ? ws.codePath : null, WHERE), you: true },
					]}
					onfollow={(index) => {
						ws.following = ws.peers[index]?.user_id ?? null;
						presenceOpen = false;
					}}
				/>
			</div>
		{/if}
	</div>
{/snippet}

{#if phone.current}
	<PhoneWorkspace {ws} {data} />
{:else}
<ProjectWorkspace bind:view={ws.workspaceView}>
	{#snippet projectbar()}
		<ProjectBar
			backHref="/projects"
			backLabel={m.workspace_back()}
			agent="koda"
			name={ws.project.name}
			status={ws.status}
			space={ws.project.org_name}
			url={ws.liveUrl ?? undefined}
			nav={[...(ws.deploy ? [{ id: 'deployments', label: m.workspace_deployments(), icon: History }] : []), ...(data.address ? [{ id: 'domain', label: m.workspace_domain(), icon: Link }] : []), ...(ws.deploy ? [{ id: 'monitor', label: m.workspace_monitor(), icon: Activity }] : []), { id: 'usage', label: m.workspace_usage(), icon: Coins }, ...(data.database ? [{ id: 'database', label: m.workspace_database(), icon: Database }] : []), { id: 'settings', label: m.project_settings(), icon: Settings }]}
			navLabel={m.workspace_tools()}
			active={ws.sheet ?? undefined}
			onnav={(id) => (ws.sheet = ws.sheet === id ? null : (id as typeof ws.sheet))}
			shareLabel={m.workspace_share()}
			exportLabel={m.workspace_export()}
			initials={ws.initials}
			onexport={ws.openExport}
			presence={ws.peers.length ? here : undefined}
			onshare={ws.liveUrl ? () => (ws.sharing = true) : undefined}
			renaming={ws.projectRenaming}
			onstartrename={() => (ws.projectRenaming = true)}
			onrename={(name) => {
				ws.projectRenaming = false;
				void ws.renameFromBar(name);
			}}
			oncancelrename={() => (ws.projectRenaming = false)}
		>
			{#snippet publish()}
				{#if ws.role === 'viewer'}
					<!-- A viewer can't publish (the backend asks for an admin): the state, with nothing to press (F1). -->
					{#if ws.button.state === 'publishing'}
						<PublishButton state="publishing" elapsed={ws.button.elapsed} />
					{:else if ws.button.state === 'idle'}
						<PublishButton state="idle" />
					{/if}
				{:else if ws.role === 'member'}
					<!-- Someone who can't publish asks the admins to, where Publish would be (step 10). -->
					{#if ws.button.state === 'publishing'}
						<PublishButton state="publishing" elapsed={ws.button.elapsed} />
					{:else if ws.mine}
						<!-- On a phone the bar keeps the project's name: the button is its icon. -->
						<span class="contents max-sm:hidden"><Button variant="secondary" iconStart={Hourglass} label={m.publish_asked()} aria-haspopup="dialog" onclick={() => (ws.mineOpen = true)} /></span>
						<span class="contents sm:hidden"><IconButton icon={Hourglass} label={m.publish_asked()} aria-haspopup="dialog" onclick={() => (ws.mineOpen = true)} /></span>
					{:else if ws.button.state === 'idle'}
						<PublishButton state="idle" />
					{:else}
						<span class="contents max-sm:hidden"><Button iconStart={Rocket} label={m.publish_ask()} loading={ws.asking} onclick={ws.askPublish} /></span>
						<span class="contents sm:hidden"><IconButton icon={Rocket} label={m.publish_ask()} disabled={ws.asking} onclick={ws.askPublish} /></span>
					{/if}
				{:else if ws.button.state === 'ready'}
					<PublishButton state="ready" count={ws.button.count} onpublish={() => (ws.confirming = true)} />
				{:else if ws.button.state === 'publishing'}
					<PublishButton state="publishing" elapsed={ws.button.elapsed} />
				{:else if ws.button.state === 'failed'}
					<PublishButton state="failed" onretry={() => (ws.confirming = true)} />
				{:else if ws.button.state === 'idle'}
					<PublishButton state="idle" />
				{/if}
			{/snippet}
		</ProjectBar>
	{/snippet}
	{#snippet chat()}
		<ChatPaneHeader title={m.workspace_chat()} crew={['koda', 'nomi']} />
		<div class="flex min-h-0 flex-1 flex-col overflow-hidden"><ChatView {data} pane quote={ws.quote} onunquote={() => (ws.quote = null)} onactivity={() => void ws.loadQueue()} after={together} /></div>
	{/snippet}
	<div class="relative flex min-h-0 flex-1 flex-col">
		<PaneTabs
			labels={{
				views: m.workspace_views(),
				preview: m.workspace_preview(),
				design: m.workspace_design(),
				code: m.workspace_code(),
				terminal: m.workspace_terminal(),
				device: m.workspace_device(),
				desktop: m.workspace_desktop(),
				phone: m.workspace_phone(),
				reload: m.workspace_reload(),
				open: m.workspace_open_tab(),
				close: m.workspace_close_preview(),
			}}
			views={ws.paneViews}
			bind:view={ws.view}
			bind:device={ws.device}
			address={ws.sandboxOn ? (ws.frame ? { shown: m.workspace_preview_sandbox() } : undefined) : ws.frame ? { url: ws.frame, shown: ws.shownUrl, live: ws.liveUrl ? m.workspace_live() : undefined, deploy: ws.liveUrl && ws.liveNumber ? m.projects_deploy({ number: ws.liveNumber }) : undefined } : undefined}
			onreload={async () => {
				// A fresh grant for the static preview, then the frame again.
				if (!ws.sandboxOn && !ws.liveUrl) await invalidate('app:workspace-project');
				ws.reloads += 1;
			}}
		/>
		{#if ws.followed}
			<FollowBar name={ws.followed.name} peer={ws.followed.peer} where={whereLabel(ws.followed.view, ws.followed.path, WHERE)} onstop={() => (ws.following = null)} />
		{/if}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="relative min-h-0 flex-1 overflow-auto p-5" onpointermove={ws.trackPointer}>
			{#each ws.pointers as pointer (pointer.user_id)}
				<span class="pointer-events-none absolute z-10" style="left: {pointer.x * 100}%; top: {pointer.y * 100}%"><PeerCursor name={pointer.name} peer={pointer.peer} /></span>
			{/each}
			{#if ws.view === 'design'}
				{#if ws.hasDesign && data.design}
					<DesignCanvas
						sessionId={ws.project.session_id}
						directions={data.design.directions ?? null}
						previews={data.design.previews}
						feedback={data.design.feedback}
						approval={data.design.approval}
						embedded
						onrefresh={async () => {
							await invalidate('app:workspace-project');
						}}
					/>
				{:else}
					<EmptyState
						icon={Palette}
						title={ws.buildingOrPublished ? m.workspace_no_design_artifact_title() : m.workspace_no_design_title()}
						body={ws.buildingOrPublished ? m.workspace_no_design_artifact_body() : m.workspace_no_design_body()}
					/>
				{/if}
			{:else if ws.view === 'code'}
				{#if ws.project.files.length && ws.codePath}
					<CodeView
						tree={ws.tree}
						path={ws.codePath}
						meta={ws.codeBinary ? m.code_binary() : ws.codeText === null ? m.code_reading() : ws.codeText.split('\n').length === 1 ? m.code_meta_one() : m.code_meta({ count: ws.codeText.split('\n').length })}
						lines={ws.codeText === null ? [] : (ws.codeLines ?? plainLines(ws.codeText))}
						onopen={(index) => {
							const entry = ws.tree[index];
							if (entry && entry.kind !== 'folder') {
								ws.openFile(entry.path);
							}
						}}
						onask={() => {
							if (!ws.codePath) return;
							ws.quote = { file: ws.codePath, range: m.code_whole_file(), snippet: '', text: m.code_about_file({ file: ws.codePath }) };
							ws.workspaceView = 'chat';
						}}
						onasklines={ws.askAbout}
					/>
				{:else}
					<EmptyState icon={MonitorX} title={m.workspace_no_files_title()} body={m.workspace_no_files_body()} />
				{/if}
			{:else if ws.view === 'terminal' && ws.runtime}
				{@const lines = terminalLines(ws.runtime.log)}
				<Terminal
					run={m.sandbox_terminal_run()}
					lines={lines.length ? lines : [{ kind: 'out', text: m.sandbox_terminal_empty() }]}
					failure={ws.sandboxPhase === 'failed' ? { title: m.sandbox_failed_title(), detail: ws.runtime.error || undefined, onfix: ws.askToFix } : undefined}
				/>
			{:else if ws.frame}
				{#key ws.reloads}
					{#if ws.device === 'phone'}
						<div class="flex justify-center"><PhoneFrame src={ws.frame} title={ws.project.name} /></div>
					{:else}
						<iframe src={ws.frame} title={ws.project.name} class="h-full min-h-150 w-full rounded-md bg-surface outline -outline-offset-1 outline-border"></iframe>
					{/if}
				{/key}
			{:else if ws.sandboxOn}
				<SandboxWait phase={ws.sandboxPhase} log={ws.runtime?.log ?? ''} error={ws.runtime?.error ?? null} onretry={() => void ws.runtime?.retry()} />
			{:else}
				<EmptyState icon={MonitorX} title={m.workspace_nothing_title()} body={ws.project.status === 'building' ? m.workspace_nothing_building() : m.workspace_nothing_body()} />
			{/if}
		</div>

		{#if ws.sheet === 'settings'}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet icon={Settings} title={m.project_settings()} meta={ws.project.name} onclose={() => (ws.sheet = null)}>
					<div class="flex flex-col border-t border-border">
						<SettingRow title={m.projects_rename_title()} description={ws.project.name}>
							{#snippet control()}<Button variant="secondary" iconStart={Pencil} label={m.projects_rename()} onclick={() => ws.openDialog('rename')} />{/snippet}
						</SettingRow>
						{#if ws.people}
							<SettingRow title={m.people_title()} description={m.people_summary({ count: ws.people.people.filter((one) => one.project_role || one.is_owner).length })}>
								{#snippet control()}<Button variant="secondary" iconStart={Users} label={m.settings_manage()} onclick={() => (ws.sheet = 'people')} />{/snippet}
							</SettingRow>
						{/if}
						{#if ws.secrets}
							<SettingRow title={m.env_title()} description={ws.secrets.secrets.length === 1 ? m.env_summary_one() : m.env_summary({ count: ws.secrets.secrets.length })}>
								{#snippet control()}<Button variant="secondary" iconStart={KeyRound} label={m.settings_manage()} onclick={() => (ws.sheet = 'environment')} />{/snippet}
							</SettingRow>
						{/if}
					</div>
					{#if ws.projectAdmin}
						<section class="flex flex-col gap-2">
							<h3 class="font-mono text-label tracking-label text-text-3 uppercase">{m.settings_ownership()}</h3>
							<div class="flex flex-col border-t border-border">
								<SettingRow title={m.settings_transfer()} description={m.settings_transfer_body({ name: ws.project.name })}>
									{#snippet control()}<Button variant="secondary" label={m.settings_transfer_action()} onclick={() => ws.openDialog('transfer')} />{/snippet}
								</SettingRow>
								<SettingRow title={m.settings_takedown()} description={ws.liveUrl ? m.settings_takedown_body() : m.settings_takedown_not_live()}>
									{#snippet control()}{#if ws.liveUrl}<Button variant="secondary" label={m.settings_takedown_action()} onclick={() => ws.openDialog('takedown')} />{/if}{/snippet}
								</SettingRow>
								<SettingRow title={m.settings_delete()} description={m.settings_delete_body()} danger>
									{#snippet control()}<Button variant="danger" label={m.settings_delete_action()} onclick={() => ws.openDialog('delete')} />{/snippet}
								</SettingRow>
							</div>
						</section>
					{/if}
				</WorkspaceSheet>
			</div>
		{/if}
		{#if ws.sheet === 'people' && ws.people}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet icon={Users} title={m.people_title()} meta={ws.project.name} onclose={() => (ws.sheet = 'settings')}>
					{#if ws.projectAdmin}
						<form
							class="flex flex-wrap items-start gap-2"
							onsubmit={(event) => {
								event.preventDefault();
								void ws.peopleAct('invite', { email: ws.inviteEmail, role: ws.inviteRole }, m.people_invited({ email: ws.inviteEmail.trim() }));
							}}
						>
							<div class="min-w-60 flex-1"><TextField label={m.people_email()} type="email" bind:value={ws.inviteEmail} error={ws.peopleError} /></div>
							<div class="w-44 pt-6">
								<Select
									label={m.people_invite_as()}
									options={[
										{ value: 'build', label: m.people_role_admin() },
										{ value: 'prompt', label: m.people_role_member() },
										{ value: 'view', label: m.people_role_viewer() },
									]}
									bind:value={ws.inviteRole}
								/>
							</div>
							<div class="pt-6"><Button type="submit" variant="inverse" label={m.people_invite()} loading={ws.busyPeople} disabled={!ws.inviteEmail.trim()} /></div>
						</form>
						<div class="flex flex-wrap items-center gap-3">
							<span class="text-ui text-text-2">{m.people_access()}</span>
							<Segmented
								label={m.people_access()}
								options={[
									{ value: 'org', label: m.people_access_org({ workspace: ws.project.org_name }) },
									{ value: 'members', label: m.people_access_members() },
								]}
								value={ws.people.access}
								onchange={(value) => void ws.peopleAct('access', { access: value }, value === 'members' ? m.people_access_limited() : m.people_access_opened())}
							/>
						</div>
					{/if}
					<ul class="flex flex-col border-t border-border">
						{#each ws.people.people as person (person.user_id)}
							<PersonRow
								name={person.name}
								initial={[...person.name][0]?.toUpperCase() ?? '?'}
								avatar={person.avatar_url}
								you={person.user_id === ws.people.me ? m.workspace_you() : undefined}
								meta={[person.email ? maskAddress(person.email) : null, person.is_owner ? m.people_started_it() : null].filter(Boolean).join(' · ')}
							>
								{#snippet controls()}
									{#if ws.projectAdmin && !person.is_owner}
										<div class="w-40">
											<Select
												label={m.people_role_of({ name: person.name })}
												options={person.project_role ? ROLES : [{ value: '', label: ws.people?.access === 'org' ? m.people_via_workspace() : m.people_not_added() }, ...ROLES]}
												value={person.project_role ?? ''}
												onchange={(role) => role && void ws.peopleAct('role', { user: person.user_id, role }, m.people_role_changed({ name: person.name }))}
											/>
										</div>
										{#if person.project_role}<IconButton icon={UserMinus} variant="ghost" label={m.people_remove({ name: person.name })} onclick={() => void ws.peopleAct('remove', { user: person.user_id }, m.people_removed({ name: person.name }))} />{/if}
									{:else}
										<span class="text-caption text-text-2">{person.is_owner ? m.people_owner() : person.project_role ? ROLES.find((one) => one.value === person.project_role)?.label : ws.people?.access === 'org' ? m.people_via_workspace() : m.people_not_added()}</span>
									{/if}
								{/snippet}
							</PersonRow>
						{/each}
					</ul>
					<p class="text-ui text-text-3">{m.people_note()}</p>
				</WorkspaceSheet>
			</div>
		{/if}
		{#if ws.sheet === 'environment' && ws.secrets}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet icon={KeyRound} title={m.env_title()} meta={ws.project.name} onclose={() => (ws.sheet = 'settings')}>
					{#if ws.secrets.secrets.length}
						<ul class="flex flex-col border-t border-border">
							{#each ws.secrets.secrets as secret (secret.name)}
								<SecretRow
									name={secret.name}
									meta={[m.env_updated({ when: ago(secret.updated_at, new Date()) }), secret.updated_by_name].filter(Boolean).join(' · ')}
									masked="••••••"
									remove={m.env_remove({ name: secret.name })}
									onremove={ws.secrets.can_edit ? () => ws.removeSecret(secret.name) : undefined}
								/>
							{/each}
						</ul>
					{:else}
						<p class="text-ui text-text-3">{m.env_none()}</p>
					{/if}
					{#if ws.secrets.can_edit}
						<form
							class="flex flex-wrap items-start gap-2"
							onsubmit={(event) => {
								event.preventDefault();
								void ws.saveSecret();
							}}
						>
							<div class="min-w-48 flex-1"><TextField label={m.env_name()} bind:value={ws.envName} mono autocomplete="off" placeholder="ADMIN_PIN" error={ws.envError?.field === 'name' ? ws.envError.message : undefined} /></div>
							<div class="min-w-48 flex-1"><TextField label={m.env_value()} type="password" bind:value={ws.envValue} mono autocomplete="off" error={ws.envError && ws.envError.field !== 'name' ? ws.envError.message : undefined} /></div>
							<div class="pt-6"><Button type="submit" variant="secondary" label={m.env_add()} loading={ws.savingEnv} disabled={!ws.envName.trim() || !ws.envValue} /></div>
						</form>
					{/if}
					<p class="text-ui text-text-3">{m.env_note()}</p>
				</WorkspaceSheet>
			</div>
		{/if}
		{#if ws.sheet === 'database'}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet icon={Database} title={m.workspace_database()} meta={ws.project.name} onclose={() => (ws.sheet = null)}>
					{#if ws.liveUrl}
						<div>
							<Segmented
								label={m.database_source()}
								options={[
									{ value: 'development', label: m.database_development() },
									{ value: 'production', label: m.database_production() },
								]}
								value={ws.dbSource}
								onchange={(value) => {
									ws.dbSource = value as typeof ws.dbSource;
									ws.dbTable = null;
									ws.editingRows = false;
								}}
							/>
						</div>
					{/if}
					{#if !ws.canRead}
						<EmptyState icon={Database} title={m.deploy_unavailable_title()} body={m.database_unavailable_body()} />
					{:else if !ws.database && ws.dbLoading}
						<p class="text-ui text-text-3">{m.database_loading()}</p>
					{:else if ws.database && 'error' in ws.database}
						<EmptyState icon={Database} title={ws.database.status === 404 ? m.database_none_title() : m.database_failed_title()} body={ws.database.error || m.database_failed_body()} />
					{:else if ws.database && !ws.database.tables.length}
						<EmptyState icon={Database} title={m.database_empty_title()} body={m.database_empty_body()} />
					{:else if ws.database}
						<div class="flex flex-wrap gap-2">
							{#each ws.database.tables as table (`${table.schema}.${table.name}`)}
								<TableChip
									name={table.name}
									rows={rowsLabel(table.rows_estimate)}
									selected={ws.database.table?.name === table.name && ws.database.table?.schema === table.schema}
									onclick={() => {
									ws.dbTable = `${table.schema}.${table.name}`;
									ws.editingRows = false;
								}}
								/>
							{/each}
						</div>
						{#if ws.database.rows && ws.database.table}
							{@const columns = ws.database.rows.columns}
							{@const raw = ws.database.rows.rows}
							<RowsGrid
								label={ws.database.table.name}
								{columns}
								rows={raw.map((row) => row.map(cellText))}
								edit={ws.editingRows && ws.dbKey
									? {
											locked: ws.dbKey.map((name) => columns.indexOf(name)),
											initial: (row, column) => (raw[row][column] === null || raw[row][column] === undefined ? '' : typeof raw[row][column] === 'object' ? JSON.stringify(raw[row][column]) : String(raw[row][column])),
											onsave: ws.saveCell,
											cellLabel: (row, column) => m.database_edit_cell({ column: columns[column], row: row + 1 }),
										}
									: undefined}
							/>
							<p class="font-mono text-label text-text-3">{ws.database.rows.truncated ? m.database_showing_first({ count: ws.database.rows.rows.length }) : rowsLabel(ws.database.rows.rows.length)}</p>
						{/if}
						<p class="flex items-center gap-2.5 rounded-sm bg-surface-2 px-3 py-2.5 text-ui text-text-2">
							<span class="inline-flex shrink-0"><Icon icon={Lock} size={14} /></span>{ws.editingRows && ws.browsed
								? m.database_editing_note({ table: ws.browsed.table.name, database: ws.dbSource === 'production' ? m.database_production() : m.database_development() })
								: ws.browsed && !ws.dbKey
									? m.database_no_key()
									: m.database_read_only_edit()}
						</p>
						<div class="flex flex-wrap gap-2">
							<Button variant="secondary" label={ws.editingRows ? m.database_done_editing() : m.database_edit_rows()} disabled={!ws.dbKey} onclick={() => (ws.editingRows = !ws.editingRows)} />
							<Button variant="secondary" iconStart={Download} label={m.database_download_dump()} href={`/projects/${ws.project.id}/database/dump?source=${ws.dbSource}`} />
						</div>
					{/if}
				</WorkspaceSheet>
			</div>
		{/if}
		{#if ws.sheet === 'monitor' && ws.deploy}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet icon={Activity} title={m.workspace_monitor()} meta={ws.project.name} onclose={() => (ws.sheet = null)}>
					{#if !ws.liveUrl}
						<EmptyState icon={Activity} title={m.monitor_unpublished_title()} body={m.monitor_unpublished_body()} />
					{:else if ws.monitorFailed}
						<EmptyState icon={Activity} title={m.monitor_failed_title()} body={m.monitor_failed_body()} />
					{:else}
						<div class="flex flex-col gap-4.5">
							<div>
								<Segmented
									label={m.monitor_range()}
									options={[
										{ value: '1h', label: '1h' },
										{ value: '24h', label: m.monitor_range_today() },
										{ value: '7d', label: '7d' },
									]}
									value={ws.range}
									onchange={(value) => (ws.range = value as MetricsRange)}
								/>
							</div>
							<div class="grid grid-cols-2 gap-2 sm:grid-cols-4">
								{#if ws.site}
									<MonitorStat label={m.monitor_visits()} value={ws.stats?.visitors} note={RANGE_NOTE[ws.range]()} notMeasured={m.monitor_not_measured()} />
									<MonitorStat label={m.monitor_form_sends()} value={ws.siteNumbers?.formSends} note={ws.siteNumbers?.formShare ? m.monitor_form_share({ share: ws.siteNumbers.formShare }) : undefined} notMeasured={m.monitor_not_measured()} />
									<MonitorStat label="P95" value={ws.stats?.p95} note={m.monitor_page_load()} notMeasured={m.monitor_not_measured()} />
									<MonitorStat label={m.monitor_uptime()} value={ws.siteNumbers?.uptime} note={ws.siteNumbers?.uptimeFor ? m.monitor_uptime_for({ time: ws.siteNumbers.uptimeFor }) : undefined} notMeasured={m.monitor_not_measured()} />
								{:else}
									<MonitorStat label={m.monitor_requests()} value={ws.stats?.requests} note={RANGE_NOTE[ws.range]()} notMeasured={m.monitor_not_measured()} />
									<MonitorStat label={m.monitor_errors()} value={ws.stats?.errors} note={ws.stats?.errorShare ? m.monitor_error_share({ share: ws.stats.errorShare }) : undefined} notMeasured={m.monitor_not_measured()} />
									<MonitorStat label="P95" value={ws.stats?.p95} note={m.monitor_p95_note()} notMeasured={m.monitor_not_measured()} />
									<MonitorStat label={m.monitor_visitors()} value={ws.stats?.visitors} note={RANGE_NOTE[ws.range]()} notMeasured={m.monitor_not_measured()} />
								{/if}
							</div>
							{#if ws.traffic?.bars.length}
								<section class="flex flex-col gap-2">
									<h3 class="flex justify-between font-mono text-label text-text-3">
										<span class="tracking-label uppercase">{trafficLabel}</span>
										{#if ws.traffic.peak}<span>{m.monitor_peak({ at: ws.traffic.peak.at, count: ws.traffic.peak.count })}</span>{/if}
									</h3>
									<TrafficBars label={trafficLabel} bars={ws.traffic.bars} axis={[ws.traffic.bars[0].at, ws.traffic.bars[ws.traffic.bars.length - 1].at]} />
								</section>
							{/if}
							{#if ws.site}
								<!-- The pen's site Monitor puts where visits come from in Resources' place, its rows wearing the resources' icons (pasted): each wears the globe. -->
								<section class="flex flex-col gap-2">
									<h3 class="font-mono text-label tracking-label text-text-3 uppercase">{m.monitor_sources()}</h3>
									{#if ws.sources?.length}
										<div class="flex flex-col">
											{#each ws.sources as source (source.name)}<ResourceRow icon={Globe} name={source.name} value={source.value} />{/each}
										</div>
									{:else}
										<p class="text-ui text-text-3">{m.monitor_sources_none()}</p>
									{/if}
								</section>
							{:else}
							<section class="flex flex-col gap-2">
								<h3 class="font-mono text-label tracking-label text-text-3 uppercase">{m.monitor_resources()}</h3>
								{#if ws.resources?.memory}
									<div class="flex flex-col">
										<ResourceRow icon={MemoryStick} name={m.monitor_memory()} value={ws.resources.memory} />
										<ResourceRow icon={Cpu} name={m.monitor_cpu()} value={ws.resources.cpu ?? ''} />
										<ResourceRow icon={ArrowDownUp} name={m.monitor_bandwidth()} value={ws.resources.bandwidth ?? ''} />
									</div>
								{:else}
									<p class="text-ui text-text-3">{m.monitor_not_measured()}</p>
								{/if}
							</section>
							{/if}
							<section class="flex flex-col gap-0.5">
								<h3 class="font-mono text-label tracking-label text-text-3 uppercase">{m.monitor_log()}</h3>
								{#if ws.monitoring?.logs?.length}
									<LogLines label={m.monitor_log()} lines={ws.monitoring.logs.slice(-40)} />
								{:else}
									<p class="text-ui text-text-3">{ws.monitoring?.note ?? m.monitor_log_empty()}</p>
								{/if}
							</section>
						</div>
					{/if}
				</WorkspaceSheet>
			</div>
		{/if}
		{#if ws.sheet === 'usage'}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet icon={Coins} title={m.workspace_usage()} meta={ws.project.name} onclose={() => (ws.sheet = null)}>
					<div class="flex flex-col gap-4.5">
						<p class="text-ui text-text-2">{m.usage_sheet_scope()}</p>
						{#if ws.usageFailed}
							<EmptyState icon={Coins} title={m.usage_sheet_failed_title()} body={m.usage_sheet_failed_body()} />
							<div><Button variant="secondary" label={m.usage_sheet_retry()} onclick={() => ws.loadUsage(ws.usageMonth)} /></div>
						{:else if !ws.usage || !ws.usageShown}
							<p class="text-ui text-text-3" role="status">{m.usage_sheet_loading()}</p>
						{:else}
							{@const shown = ws.usageShown}
							<div class="flex items-center gap-2">
								<IconButton icon={ChevronLeft} label={m.usage_sheet_prev()} onclick={() => ws.loadUsage(shiftMonth(ws.usageMonth ?? ws.usage!.month, -1))} />
								<span class="min-w-36 text-center text-ui font-medium text-text" aria-live="polite">{shown.month}</span>
								<IconButton icon={ChevronRight} label={m.usage_sheet_next()} disabled={shown.current} onclick={() => ws.loadUsage(shiftMonth(ws.usageMonth ?? ws.usage!.month, 1))} />
							</div>
							<div class={['flex flex-col gap-4.5', ws.usageLoading && 'opacity-60']} aria-busy={ws.usageLoading}>
								<div class="grid grid-cols-1 gap-2 sm:grid-cols-3">
									<MonitorStat label={m.usage_sheet_spend()} value={shown.spend} note={shown.spendUsd} notMeasured="" />
									<MonitorStat label={m.usage_sheet_tokens()} value={shown.tokens} notMeasured="" />
									<MonitorStat label={m.usage_sheet_calls()} value={shown.calls} notMeasured="" />
								</div>
								{#if shown.ownKey}<p class="text-ui text-text-3">{m.usage_sheet_own_key({ tokens: shown.ownKey })}</p>{/if}
								{#if shown.empty}
									<EmptyState icon={Coins} title={m.usage_sheet_empty_title({ month: shown.month })} body={m.usage_sheet_empty_body()} />
								{:else}
									<section class="flex flex-col gap-2">
										<h3 class="font-mono text-label tracking-label text-text-3 uppercase">{m.usage_sheet_by_model()}</h3>
										<ul class="flex flex-col">
											{#each shown.models as row (row.key)}
												<li class="flex items-center gap-3 border-b border-border py-2 last:border-b-0">
													<span class="min-w-0 flex-1 truncate text-ui text-text">{row.label}{#if row.ownKey}<span class="ml-2 rounded-sm px-1.5 py-0.5 font-mono text-label text-text-3 outline -outline-offset-1 outline-border">{m.usage_sheet_own_key_tag()}</span>{/if}</span>
													<span class="w-16 text-right font-mono text-label text-text-2">{row.tokens}</span>
													<span class="w-20 text-right font-mono text-label text-text">{row.spend}</span>
												</li>
											{/each}
										</ul>
									</section>
									{#if shown.people?.length}
										<section class="flex flex-col gap-2">
											<h3 class="font-mono text-label tracking-label text-text-3 uppercase">{m.usage_sheet_by_person()}</h3>
											<ul class="flex flex-col">
												{#each shown.people as row (row.id)}
													<li class="flex items-center gap-3 border-b border-border py-2 last:border-b-0">
														<span class="min-w-0 flex-1 truncate text-ui text-text">{row.name ?? m.usage_sheet_unknown_person()}</span>
														<span class="w-16 text-right font-mono text-label text-text-2">{row.tokens}</span>
														<span class="w-20 text-right font-mono text-label text-text">{row.spend}</span>
													</li>
												{/each}
											</ul>
										</section>
									{/if}
								{/if}
								<p class="font-mono text-label text-text-3">{shown.since ? m.usage_sheet_counted_since({ date: shown.since }) : m.usage_sheet_counting_starts()}</p>
							</div>
						{/if}
					</div>
				</WorkspaceSheet>
			</div>
		{/if}
		{#if ws.sheet === 'domain' && ws.address}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet icon={Link} title={m.workspace_domain()} meta={ws.project.name} onclose={() => (ws.sheet = null)}>
					{#if !ws.address.hostname}
						<EmptyState icon={Globe} title={m.domain_none_title()} body={m.domain_none_body()} />
					{:else}
						<div class="flex flex-col gap-2">
							<p class="font-mono text-label tracking-label text-text-3 uppercase">{m.domain_addresses()}</p>
							<AddressRow address={ws.address.hostname} href={ws.address.url ?? undefined} state={ws.liveUrl ? { label: m.deploy_state_live(), tone: 'live' } : undefined} />
							{#if ws.address.pending_hostname}<AddressRow address={ws.address.pending_hostname} state={{ label: m.domain_next_publish(), tone: 'waiting' }} strong />{/if}
							{#each ws.ownDomains as own (own.hostname)}
								{@const shown = domainView(own, ws.time.timeZone)}
								<AddressRow address={own.hostname} href={shown.state.tone === 'live' ? `https://${own.hostname}` : undefined} state={shown.state} strong />
							{/each}
						</div>
						{#each ws.ownDomains as own (own.hostname)}
							{@const shown = domainView(own, ws.time.timeZone)}
							<section aria-label={own.hostname} class="flex flex-col gap-4">
								{#if ws.ownDomains.length > 1}<p class="font-mono text-ui text-text">{own.hostname}{#if own.primary} · {m.domain_primary_tag()}{/if}</p>{/if}
								<DnsRecords
									records={shown.records}
									checked={shown.checked}
									labels={{ heading: m.domain_registrar(), type: m.domain_col_type(), name: m.domain_col_name(), value: m.domain_col_value(), status: m.domain_col_status(), found: m.domain_found(), waiting: m.domain_waiting(), copy: m.domain_copy_value() }}
								/>
								<p class="text-ui text-text-3">{shown.note}</p>
								<div class="flex flex-wrap gap-2">
									<Button variant="secondary" label={m.domain_check_now()} iconEnd={RefreshCw} loading={ws.domainBusy === `domainCheck:${own.hostname}`} onclick={() => void ws.domainAction('domainCheck', own.hostname)} />
									{#if ws.address.can_edit && shown.canMakePrimary}
										<Button variant="secondary" label={m.domain_make_primary()} loading={ws.domainBusy === `domainPrimary:${own.hostname}`} onclick={() => void ws.domainAction('domainPrimary', own.hostname, m.domain_primary_done({ hostname: own.hostname }))} />
									{/if}
									{#if ws.address.can_edit}<Button variant="ghost" iconStart={Trash2} label={m.domain_remove()} onclick={() => (ws.removingDomain = own.hostname)} />{/if}
								</div>
							</section>
						{/each}
						{#if ws.renaming}
							<div class="flex flex-col gap-3">
								<SubdomainField
									bind:value={ws.name}
									suffix={ws.address.base_domain ?? ws.address.hostname.split('.').slice(1).join('.')}
									check={ws.check}
									labels={{ field: m.domain_field(), checking: m.domain_checking(), available: m.domain_available(), suggestions: m.domain_free_instead() }}
									oninput={ws.checkSoon}
									onpick={(picked) => {
										ws.name = picked;
										ws.checkSoon(picked);
									}}
								/>
								<div class="flex gap-2">
									<Button variant="inverse" label={m.domain_save()} loading={ws.saving} disabled={!ws.name.trim() || ws.check.state === 'taken' || ws.check.state === 'checking'} onclick={ws.saveName} />
									<Button variant="ghost" label={m.action_cancel()} onclick={() => (ws.renaming = false)} />
								</div>
								<p class="text-ui text-text-3">{m.domain_rename_note()}</p>
							</div>
						{:else if ws.address.can_edit}
							<div><Button variant="secondary" label={m.domain_change()} onclick={ws.startRenaming} /></div>
						{/if}
					{/if}
					{#if ws.address.can_edit}
						<section aria-labelledby="own-domain-title" class="flex flex-col gap-3 border-t border-border pt-4">
							<header class="flex flex-col gap-0.5">
								<h3 id="own-domain-title" class="text-ui font-medium text-text">{m.domain_add_title()}</h3>
								<p class="text-caption text-text-3">{ws.liveUrl ? m.domain_add_body() : m.domain_add_live_first()}</p>
							</header>
							{#if ws.liveUrl}
								<form
									class="flex items-end gap-2"
									onsubmit={async (event) => {
										event.preventDefault();
										if (await ws.domainAction('domainAdd', ws.newDomain.trim().toLowerCase())) ws.newDomain = '';
									}}
								>
									<div class="min-w-0 flex-1"><TextField label={m.domain_add_field()} bind:value={ws.newDomain} placeholder="warungkopi.id" mono name="hostname" autocomplete="off" /></div>
									<Button type="submit" variant="inverse" label={m.domain_add()} loading={ws.domainBusy?.startsWith('domainAdd:') ?? false} disabled={!ws.newDomain.trim()} />
								</form>
							{/if}
						</section>
					{/if}
					{#if ws.domainError}<p role="alert" class="text-caption text-danger">{ws.domainError}</p>{/if}
				</WorkspaceSheet>
			</div>
		{/if}
		{#if ws.sheet === 'deployments' && ws.deploy}
			<div bind:this={sheetBox} class="absolute inset-y-0 right-0 flex w-180 max-w-full">
				<WorkspaceSheet
					icon={History}
					title={m.workspace_deployments()}
					meta={ws.rows.length === 1 ? m.deploy_count_one({ name: ws.project.name }) : m.deploy_count({ name: ws.project.name, count: ws.rows.length })}
					onclose={() => (ws.sheet = null)}
				>
					{#if !ws.deploy.available}
						<EmptyState icon={Rocket} title={m.deploy_unavailable_title()} body={m.deploy_unavailable_body()} />
					{:else if !ws.rows.length}
						<EmptyState icon={Rocket} title={m.deploy_none_title()} body={m.deploy_none_body()} />
					{:else}
						<div class="flex flex-col gap-4">
							{#if ws.liveUrl}<p class="font-mono text-label tracking-label text-text-3 uppercase">{m.deploy_live_at({ address: ws.shownUrl })}</p>{/if}
							<ul class="flex flex-col">
								{#each ws.rows as row (row.id)}
									<DeployRow
										number={row.number}
										title={row.title}
										meta={row.meta}
										state={row.state}
										error={row.error}
										labels={{ live: m.deploy_state_live(), failed: m.deploy_state_failed(), running: m.deploy_state_running(), rollback: m.deploy_roll_back_to({ number: '{number}' }) }}
										busy={ws.rollingBack === row.id}
										onrollback={() => ws.rollBack(row.id)}
									/>
								{/each}
							</ul>
							<p class="text-ui text-text-3">{m.deploy_rollback_note({ number: ws.nextNumber })}</p>
						</div>
					{/if}
				</WorkspaceSheet>
			</div>
		{/if}
	</div>
</ProjectWorkspace>

<ProjectDialog
	open={ws.confirming}
	title={ws.button.state === 'ready' ? (ws.button.count === 1 ? m.deploy_confirm_one() : m.deploy_confirm({ count: ws.button.count })) : m.deploy_confirm_retry()}
	description={ws.liveUrl ? m.deploy_confirm_body({ address: ws.shownUrl, number: ws.nextNumber }) : m.deploy_confirm_first({ number: ws.nextNumber })}
	width={480}
	onclose={() => (ws.confirming = false)}
>
	{#if ws.liveNumber}<p class="font-mono text-label text-text-3">{m.deploy_confirm_note({ number: ws.liveNumber })}</p>{/if}
	{#snippet actions()}
		<Button variant="ghost" label={m.action_cancel()} onclick={() => (ws.confirming = false)} />
		<Button variant="primary" iconEnd={Rocket} label={m.deploy_publish_as({ number: ws.nextNumber })} loading={ws.sending} onclick={ws.startPublish} />
	{/snippet}
</ProjectDialog>

<ProjectDialog open={ws.mineOpen && Boolean(ws.mine)} title={m.publish_request_mine_title()} description={m.publish_request_mine_body()} width={420} onclose={() => (ws.mineOpen = false)}>
	{#if ws.mine}<p class="font-mono text-label text-text-3">{requestMeta(ws.mine, ws.deploy?.unpublished_changes ?? null, ws.time.timeZone)}</p>{/if}
	{#snippet actions()}
		<Button variant="ghost" label={m.publish_request_keep()} onclick={() => (ws.mineOpen = false)} />
		<Button variant="secondary" label={m.publish_request_cancel()} loading={Boolean(ws.mine) && ws.answering === ws.mine?.id} onclick={() => ws.mine && void ws.answerRequest(ws.mine, 'cancel')} />
	{/snippet}
</ProjectDialog>

<ProjectDialog open={ws.dialog === 'transfer'} title={m.transfer_title({ name: ws.project.name })} description={m.transfer_body()} width={460} onclose={() => (ws.dialog = null)}>
	<div class="flex flex-col gap-3 pt-2">
		<Search
			placeholder={m.transfer_search()}
			bind:value={ws.targetQuery}
			oninput={() => {
				clearTimeout(ws.findTimer);
				ws.findTimer = setTimeout(() => void ws.findTargets(ws.targetQuery.trim()), 300);
			}}
		/>
		{#if ws.targets.length}
			<div role="radiogroup" aria-label={m.transfer_to()} class="flex max-h-48 flex-col overflow-y-auto rounded-sm outline -outline-offset-1 outline-border">
				{#each ws.targets as one (one.id)}
					<label class={['flex cursor-pointer items-center gap-2.5 border-b border-border px-3 py-2.5 last:border-b-0', ws.target === one.id ? 'bg-surface-2' : 'hover:bg-surface-2']}>
						<input type="radio" name="transfer-target" value={one.id} bind:group={ws.target} class="accent-text" />
						<span class="min-w-0 flex-1 truncate text-ui text-text">{one.name}</span>
						{#if one.owner_name}<span class="shrink-0 font-mono text-label text-text-3">{m.transfer_owner({ name: one.owner_name })}</span>{/if}
					</label>
				{/each}
			</div>
		{:else}
			<p class="text-caption text-text-3">{m.transfer_none()}</p>
		{/if}
		<TextField label={m.confirm_type_name({ name: ws.project.name })} bind:value={ws.confirmName} mono autocomplete="off" error={ws.actError} />
	</div>
	{#snippet actions()}
		<Button variant="ghost" label={m.action_cancel()} onclick={() => (ws.dialog = null)} />
		<Button variant="inverse" label={m.transfer_send()} loading={ws.acting} disabled={!ws.target || ws.confirmName.trim() !== ws.project.name} onclick={() => ws.act('transfer', { to: ws.target ?? '', name: ws.confirmName.trim() })} />
	{/snippet}
</ProjectDialog>

<ProjectDialog open={ws.dialog === 'rename'} title={m.projects_rename_title()} width={460} onclose={() => (ws.dialog = null)}>
	<div class="flex flex-col gap-3 pt-2">
		<TextField label={m.projects_rename_label()} bind:value={ws.renameName} autocomplete="off" error={ws.actError} />
	</div>
	{#snippet actions()}
		<Button variant="ghost" label={m.action_cancel()} onclick={() => (ws.dialog = null)} />
		<Button variant="inverse" label={m.workspace_rename_save()} loading={ws.acting} disabled={!ws.renameName.trim() || ws.renameName.trim() === ws.project.name} onclick={() => ws.act('rename', { name: ws.renameName.trim() })} />
	{/snippet}
</ProjectDialog>

<ProjectDialog open={ws.removingDomain !== null} title={m.domain_remove_title({ hostname: ws.removingDomain ?? '' })} description={m.domain_remove_body()} width={460} onclose={() => (ws.removingDomain = null)}>
	{#snippet actions()}
		<Button variant="ghost" label={m.action_cancel()} onclick={() => (ws.removingDomain = null)} />
		<Button
			variant="danger"
			label={m.domain_remove()}
			loading={ws.domainBusy?.startsWith('domainRemove:') ?? false}
			onclick={async () => {
				const hostname = ws.removingDomain ?? '';
				if (await ws.domainAction('domainRemove', hostname, m.domain_removed({ hostname }))) ws.removingDomain = null;
			}}
		/>
	{/snippet}
</ProjectDialog>

<ProjectDialog open={ws.dialog === 'takedown'} title={m.takedown_title({ name: ws.project.name })} description={m.takedown_body({ address: ws.shownUrl })} width={460} onclose={() => (ws.dialog = null)}>
	<div class="flex flex-col gap-2 pt-2">
		<p class="font-mono text-label text-text-3"><span class="tracking-label uppercase">{m.takedown_stays()}</span> · {m.takedown_stays_body({ files: ws.project.files.length, deploys: ws.rows.length })}</p>
		<p class="font-mono text-label text-text-3">{m.takedown_back()}</p>
		{#if ws.actError}<p class="text-caption text-danger">{ws.actError}</p>{/if}
	</div>
	{#snippet actions()}
		<Button variant="ghost" label={m.takedown_keep()} onclick={() => (ws.dialog = null)} />
		<Button variant="danger" label={m.settings_takedown()} loading={ws.acting} onclick={() => ws.act('takedown', {})} />
	{/snippet}
</ProjectDialog>

<ProjectDialog open={ws.dialog === 'delete'} title={m.delete_title({ name: ws.project.name })} description={m.delete_body()} width={460} onclose={() => (ws.dialog = null)}>
	<div class="flex flex-col gap-3 pt-2">
		<div class="flex items-center justify-between gap-3 rounded-sm bg-surface-2 px-3 py-2.5">
			<span class="text-ui text-text-2">{m.delete_export_first()}</span>
			<Button variant="secondary" iconStart={Download} label={m.workspace_export()} href="/projects/{ws.project.id}/export" data-sveltekit-reload />
		</div>
		<TextField label={m.confirm_type_name({ name: ws.project.name })} bind:value={ws.confirmName} mono autocomplete="off" error={ws.actError} />
	</div>
	{#snippet actions()}
		<Button variant="ghost" label={m.action_cancel()} onclick={() => (ws.dialog = null)} />
		<Button variant="danger" label={m.delete_forever()} loading={ws.acting} disabled={ws.confirmName.trim() !== ws.project.name} onclick={() => ws.act('delete', { name: ws.confirmName.trim() })} />
	{/snippet}
</ProjectDialog>

{#if ws.liveUrl}
	<BuildShareDialog
		bind:open={ws.sharing}
		url={ws.liveUrl}
		preview={{ title: ws.project.name, ...(ws.project.description && { description: ws.project.description }) }}
		visibility={ws.showcased ? 'public' : 'unlisted'}
		onvisibility={data.showcase?.can_edit ? (next) => void ws.setShown(next === 'public') : undefined}
		oninvite={ws.people && ws.projectAdmin
			? () => {
					ws.sharing = false;
					ws.sheet = 'people';
				}
			: undefined}
	/>
{/if}

<ExportDialog
	bind:open={ws.exporting}
	project={ws.project.name}
	repo={ws.summary ? (ws.summary.files === 1 ? m.export_holds_one({ size: kilobytes(ws.summary.bytes) }) : m.export_holds({ files: ws.summary.files, size: kilobytes(ws.summary.bytes) })) : m.export_reading()}
	database={ws.summary && ws.summary.database !== 'none' ? m.export_database({ kind: ws.summary.database }) : undefined}
	env={ws.summary?.env.length ? ws.summary.env.join(' · ') : undefined}
	onpush={() => {
		ws.exporting = false;
		ws.pushError = undefined;
		ws.pushing = true;
	}}
	ondownload={() => (location.href = `/projects/${ws.project.id}/export`)}
/>

<ProjectDialog open={ws.pushing} title={m.github_title()} description={m.github_body()} width={460} onclose={() => (ws.pushing = false)}>
	<div class="flex flex-col gap-3 pt-2">
		<TextField label={m.github_repo()} bind:value={ws.repo} mono placeholder="owner/{ws.project.name}" autocomplete="off" error={ws.pushError} />
		<Checkbox label={m.github_create()} bind:checked={ws.createRepo} />
		<Checkbox label={m.github_private()} bind:checked={ws.privateRepo} />
		<p class="text-caption text-text-3">{m.github_note()} <a href="/settings/connections" class="text-text-2 underline underline-offset-2 hover:text-text">{m.github_connections()}</a></p>
	</div>
	{#snippet actions()}
		<Button variant="ghost" label={m.action_cancel()} onclick={() => (ws.pushing = false)} />
		<Button variant="inverse" label={m.github_push()} loading={ws.sendingPush} disabled={!ws.repo.trim()} onclick={ws.push} />
	{/snippet}
</ProjectDialog>
{/if}
