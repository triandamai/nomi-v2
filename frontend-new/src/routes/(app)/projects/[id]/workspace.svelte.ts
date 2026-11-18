import { onMount, untrack } from 'svelte';
import { m } from '$lib/paraglide/messages';
import { pageAction } from '$lib/pageAction';
import { clock as appClock } from '$lib/clock';
import { deployRows, publishState, publishing } from '$lib/projects/deploy';
import type { DeployOverview, ExportSummary, MetricsRange, ProjectUsage } from '$lib/adapters/api';
import { toasts } from '$lib/ds/overlays/toasts.svelte';
import type { PaneView } from '$lib/ds/projects/PaneTabs.svelte';
import type { SubdomainCheck } from '$lib/ds/projects/SubdomainField.svelte';
import type { DatabaseView, Monitoring, TransferTarget } from '$lib/server/workspace';
import { monitorStats, resourceRows, siteStats, sourceRows, trafficBars } from '$lib/projects/monitor';
import { usageView } from '$lib/projects/usage';
import { getLocale } from '$lib/paraglide/runtime';
import type { CodeLine } from '$lib/ds/chat/code';
import { fileTree, firstFile, quoteLines } from '$lib/projects/code';
import { SandboxRuntime } from '$lib/projects/sandbox.svelte';
import { ProjectLive } from '$lib/projects/live.svelte';
import { distinctInitials, peerFor, pointersIn } from '$lib/projects/peers';
import { answeredRequests, askLine, movableAsk, myOpenRequest, openRequests, type PublishRequestRow, type QueuedAsk } from '$lib/projects/asks';
import { goto, invalidate } from '$app/navigation';
import { page } from '$app/state';
import type { PageData } from './$types';

/** The desktop's tool sheets (the phone opens the same ones as bottom sheets). */
export type ToolSheet = 'deployments' | 'domain' | 'monitor' | 'usage' | 'database' | 'settings' | 'people' | 'environment';
/** A `?open=` link, read once when the workspace opens (each layout opens it its own way). */
export type OpenRequest = { open: string; file: string | null };
/** What a new person's invite gave back (People, G10): the link to copy and when it runs out. */
export type InviteSent = { email: string; added: boolean; path: string | null; expiresAt: string | null };

/**
 * A project's workspace, shared by its two layouts (AGENTS.md §9 "A project's workspace"; Mobile
 * Build): the desktop's split and the phone's screen read the same state and call the same
 * actions, and only one of them is mounted. Everything here was the desktop page's script: the
 * deploy overview and its poll, the sandbox, the project's live link (presence, follow, deploy
 * changes), the asks in line and the requests to publish, and every tool's state and action.
 * Made while the page initialises (it starts its own effects and timers there).
 */
export class Workspace {
	readonly #data: () => PageData;
	readonly time = appClock();

	constructor(data: () => PageData) {
		this.#data = data;
		this.deploy = untrack(() => data().deploy);
		this.showcased = untrack(() => data().showcase?.showcase ?? false);
		this.codePath = untrack(() => firstFile(data().project.files));

		$effect(() => {
			this.deploy = this.#data().deploy;
		});
		$effect(() => {
			this.showcased = this.#data().showcase?.showcase ?? false;
		});
		// The sandbox runs while the workspace is open.
		$effect(() => {
			if (!this.sandboxOn) return;
			const one = new SandboxRuntime(this.projectId);
			this.runtime = one;
			void one.start();
			return () => {
				one.stop();
				this.runtime = null;
			};
		});
		onMount(() => {
			const tick = setInterval(() => {
				this.now = new Date();
				if (this.deploy && publishing(this.deploy)) void this.refresh();
			}, 3000);
			return () => clearInterval(tick);
		});
		// `?open=`: read once, then the address is tidied so a reload doesn't open it again.
		onMount(() => {
			const open = page.url.searchParams.get('open');
			if (!open) return;
			this.opened = { open, file: page.url.searchParams.get('file') };
			const url = new URL(page.url);
			url.searchParams.delete('open');
			url.searchParams.delete('file');
			void goto(url, { replaceState: true, noScroll: true, keepFocus: true });
		});
		$effect(() => {
			if (this.sheet !== 'monitor' || !this.liveUrl) return;
			const asked = this.range;
			void this.loadMonitoring(asked);
			const timer = setInterval(() => !document.hidden && void this.loadMonitoring(asked), 30_000);
			return () => clearInterval(timer);
		});
		$effect(() => {
			// Another project opened in this page starts again from its own month.
			if (this.sheet !== 'usage' || !this.project.id) return;
			untrack(() => void this.loadUsage(null));
		});
		$effect(() => {
			if (this.sheet === 'database' && this.canRead) void this.loadDatabase(this.dbSource, this.dbTable);
		});
		$effect(() => {
			if (this.codeOpen && this.codePath) void this.readFile(this.codePath);
		});
		// The project's live link keeps the person counted as here while the workspace is open.
		$effect(() => {
			const one = new ProjectLive(this.projectId);
			this.live = one;
			one.start();
			return () => {
				one.stop();
				this.live = null;
			};
		});
		$effect(() => {
			this.live?.moveTo({ view: this.view, path: this.view === 'code' ? this.codePath : null });
		});
		$effect(() => {
			// Both are read here, so a move to another file alone follows too.
			const where = this.followedView;
			const path = this.followedPath;
			if (!where || (where === 'terminal' && !this.sandboxOn)) return;
			untrack(() => {
				if (this.view !== where) this.view = where;
				if (where === 'code' && path && path !== this.codePath) this.openFile(path);
			});
		});
		// Moving on your own stops following; so does them leaving.
		$effect(() => {
			if (this.following && (!this.followed || this.view !== untrack(() => this.followedView))) this.following = null;
		});
		$effect(() => {
			if (!this.live || !Object.keys(this.live.pointers).length) return;
			const timer = setInterval(() => (this.pointerClock = Date.now()), 1000);
			return () => clearInterval(timer);
		});
		onMount(() => {
			void this.loadQueue();
			void this.loadRequests();
		});
		$effect(() => {
			const link = this.live;
			if (!link) return;
			// A request asked or answered, a publish moving on: both read again.
			return link.onDeployChange(() => {
				void this.loadRequests();
				void this.refresh();
			});
		});
	}

	/** The phone's layout is showing (a few toasts say things its own way). */
	phone = false;

	get data() {
		return this.#data();
	}
	readonly project = $derived(this.data.project);
	/** A static site's own files, sandboxed; the address carries a grant that lasts an hour at most. */
	readonly staticPreview = $derived(this.data.preview);

	// ── Publishing ────────────────────────────────────────────────────────────────────────────
	deploy = $state<DeployOverview | null>(null);
	now = $state(new Date());
	readonly liveRow = $derived(this.deploy?.publishes.find((one) => one.is_live && one.environment === 'production'));
	readonly liveUrl = $derived(this.deploy?.url ?? this.data.live?.url ?? null);
	readonly liveNumber = $derived(this.liveRow?.version ?? this.data.live?.deploys ?? null);
	/** What the Publish button says. */
	readonly button = $derived(this.deploy ? publishState(this.deploy, this.project.files.length, this.now) : ({ state: 'off' } as const));
	readonly productionPublishes = $derived(this.deploy?.publishes.filter((one) => one.environment === 'production') ?? []);
	readonly rows = $derived(this.deploy ? deployRows(this.productionPublishes, this.now, this.time.timeZone) : []);
	readonly nextNumber = $derived((this.deploy?.publishes.reduce((top, one) => Math.max(top, one.version), 0) ?? 0) + 1);
	readonly status = $derived(
		this.liveUrl
			? { label: m.workspace_status_live(), tone: 'accent' as const }
			: this.project.status === 'building'
				? { label: m.workspace_status_building(), tone: 'quiet' as const }
				: this.project.status === 'ready'
					? { label: m.workspace_status_ready(), tone: 'warning' as const }
					: { label: m.workspace_status_planning(), tone: 'quiet' as const },
	);
	readonly shownUrl = $derived(this.liveUrl ? this.liveUrl.replace(/^https?:\/\//, '').replace(/\/$/, '') : m.workspace_preview_files());
	readonly initials = $derived(
		(this.data.person?.name ?? '')
			.split(/\s+/)
			.map((word: string) => word.charAt(0))
			.join('')
			.slice(0, 2)
			.toUpperCase() || '·',
	);

	async refresh() {
		const response = await fetch(`/projects/${this.project.id}/deploy`).catch(() => null);
		if (!response?.ok) return;
		const next = (await response.json()) as DeployOverview;
		this.settled(this.deploy, next);
		this.deploy = next;
	}

	/** A publish that has just finished says so: live with a way back, or failed with its reason. */
	settled(before: DeployOverview | null, after: DeployOverview) {
		const was = before?.publishes.find((one) => one.status === 'running');
		if (!was) return;
		const done = after.publishes.find((one) => one.id === was.id);
		if (done?.status === 'succeeded') {
			const previous = after.publishes.find((one) => one.status === 'succeeded' && one.id !== done.id && one.can_roll_back && one.environment === done.environment);
			// A phone says a roll back as the pen draws it (J1r3Q): which build it is, and Restore.
			if (this.phone && done.rollback_of_version !== null) {
				toasts.show({
					kind: 'success',
					title: m.pb_rolled_back_live({ number: done.version }),
					detail: m.pb_same_build({ number: done.rollback_of_version }),
					...(previous && { action: { label: m.pb_restore({ number: previous.version }), onclick: () => void this.rollBack(previous.id) } }),
				});
				return;
			}
			toasts.show({
				kind: 'success',
				title: m.deploy_live({ number: done.version }),
				...(previous && { action: { label: m.deploy_roll_back_to({ number: previous.version }), onclick: () => void this.rollBack(previous.id) } }),
			});
		} else if (done?.status === 'failed') {
			toasts.show({ kind: 'error', title: m.deploy_failed({ number: done.version }), detail: done.error ?? m.deploy_failed_body() });
		}
	}

	confirming = $state(false);
	sending = $state(false);
	startPublish = async () => {
		this.sending = true;
		const result = await pageAction(`/projects/${this.project.id}`, 'publish');
		this.sending = false;
		this.confirming = false;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.deploy_couldnt_start(), detail: result.message });
			return;
		}
		await this.refresh();
	}

	rollingBack = $state<string>();
	async rollBack(publishId: string) {
		this.rollingBack = publishId;
		const result = await pageAction(`/projects/${this.project.id}`, 'rollback', { publish: publishId });
		this.rollingBack = undefined;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.deploy_couldnt_start(), detail: result.message });
			return;
		}
		await this.refresh();
	}

	// ── The sandbox ───────────────────────────────────────────────────────────────────────────
	readonly sandboxOn = $derived(this.data.sandbox);
	readonly projectId = $derived(this.project.id);
	runtime = $state<SandboxRuntime | null>(null);
	readonly sandboxPhase = $derived(this.runtime?.phase ?? 'loading');
	readonly frame = $derived(this.sandboxOn ? (this.runtime?.previewUrl ?? null) : (this.liveUrl ?? this.staticPreview));
	/** Ask Koda to fix: the sandbox's error quoted above the composer, on the chat. */
	askToFix = () => {
		const error = this.runtime?.error || m.sandbox_failed_body();
		this.quote = { file: m.sandbox_quote_file(), range: m.sandbox_quote_range(), snippet: error, text: m.sandbox_fix_text({ error }) };
		this.workspaceView = 'chat';
	}

	// ── Domain ────────────────────────────────────────────────────────────────────────────────
	readonly address = $derived(this.data.address);
	renaming = $state(false);
	name = $state('');
	check = $state<SubdomainCheck>({ state: 'idle' });
	saving = $state(false);
	#checkTimer: ReturnType<typeof setTimeout> | undefined;
	startRenaming = () => {
		this.name = this.address?.subdomain ?? '';
		this.check = { state: 'idle' };
		this.renaming = true;
	}
	checkSoon = (next: string) => {
		clearTimeout(this.#checkTimer);
		const wanted = next.trim().toLowerCase();
		if (!wanted || wanted === this.address?.subdomain) return void (this.check = { state: 'idle' });
		this.check = { state: 'checking' };
		this.#checkTimer = setTimeout(async () => {
			const response = await fetch(`/projects/${this.project.id}/domain/check?name=${encodeURIComponent(wanted)}`).catch(() => null);
			const answer = response ? ((await response.json().catch(() => null)) as { available?: boolean; message?: string; suggestions?: string[]; error?: string } | null) : null;
			if (this.name.trim().toLowerCase() !== wanted) return;
			if (!answer || answer.error !== undefined) this.check = { state: 'taken', message: answer?.error || m.domain_check_failed() };
			else this.check = answer.available ? { state: 'available' } : { state: 'taken', message: answer.message || m.domain_taken(), suggestions: answer.suggestions };
		}, 400);
	}
	saveName = async () => {
		this.saving = true;
		const result = await pageAction(`/projects/${this.project.id}`, 'domain', { name: this.name });
		this.saving = false;
		if (!result.ok) {
			this.check = { state: 'taken', message: result.message || m.action_failed() };
			return;
		}
		this.renaming = false;
		toasts.show({ kind: 'success', title: m.domain_changed() });
		await invalidate('app:workspace-project');
	}

	readonly ownDomains = $derived(this.data.ownDomains ?? []);
	newDomain = $state('');
	domainBusy = $state<string | null>(null);
	domainError = $state<string | null>(null);
	removingDomain = $state<string | null>(null);
	async domainAction(name: 'domainAdd' | 'domainCheck' | 'domainPrimary' | 'domainRemove', hostname: string, done?: string): Promise<boolean> {
		this.domainBusy = `${name}:${hostname}`;
		this.domainError = null;
		const result = await pageAction(`/projects/${this.project.id}`, name, { hostname });
		this.domainBusy = null;
		if (!result.ok) {
			this.domainError = result.message || m.action_failed();
			return false;
		}
		if (done) toasts.show({ kind: 'success', title: done });
		await invalidate('app:workspace-project');
		return true;
	}

	// ── Monitor ───────────────────────────────────────────────────────────────────────────────
	range = $state<MetricsRange>('24h');
	monitoring = $state<Monitoring | null>(null);
	monitorFailed = $state(false);
	async loadMonitoring(asked: MetricsRange) {
		const response = await fetch(`/projects/${this.project.id}/monitor?range=${asked}`).catch(() => null);
		if (asked !== this.range) return;
		this.monitorFailed = !response?.ok;
		this.monitoring = response?.ok ? ((await response.json()) as Monitoring) : null;
	}
	readonly stats = $derived(this.monitoring?.metrics ? monitorStats(this.monitoring.metrics) : null);
	/** A site is watched by its visits; an app leads with its requests and errors. */
	readonly site = $derived(this.project.stack === 'static');
	readonly traffic = $derived(this.monitoring?.metrics ? trafficBars(this.monitoring.metrics.buckets, this.time.timeZone, this.site ? 'visitors' : 'requests') : null);
	readonly resources = $derived(this.monitoring?.usage ? resourceRows(this.monitoring.usage) : null);
	readonly siteNumbers = $derived(this.monitoring?.metrics ? siteStats(this.monitoring.metrics, this.now) : null);
	readonly sources = $derived(sourceRows(this.monitoring?.metrics?.sources));

	// ── Tokens & cost ─────────────────────────────────────────────────────────────────────────
	usageMonth = $state<string | null>(null);
	usage = $state<ProjectUsage | null>(null);
	usageFailed = $state(false);
	usageLoading = $state(false);
	async loadUsage(asked: string | null) {
		this.usageMonth = asked;
		this.usageLoading = true;
		this.usageFailed = false;
		const response = await fetch(`/projects/${this.project.id}/usage${asked ? `?month=${asked}` : ''}`).catch(() => null);
		// A slow answer for a month the person already left is dropped.
		if (asked !== this.usageMonth) return;
		const next = response?.ok ? ((await response.json().catch(() => null)) as ProjectUsage | null) : null;
		if (asked !== this.usageMonth) return;
		this.usageLoading = false;
		if (next) this.usage = next;
		else this.usageFailed = true;
	}
	readonly usageShown = $derived(this.usage ? usageView(this.usage, getLocale()) : null);

	// ── Database ──────────────────────────────────────────────────────────────────────────────
	dbSource = $state<'development' | 'production'>('development');
	dbTable = $state<string | null>(null);
	database = $state<DatabaseView | null>(null);
	dbLoading = $state(false);
	async loadDatabase(source: 'development' | 'production', table: string | null) {
		this.dbLoading = true;
		const query = new URLSearchParams({ source, ...(table && { table }) });
		const response = await fetch(`/projects/${this.project.id}/database?${query}`).catch(() => null);
		this.dbLoading = false;
		this.database = response ? ((await response.json().catch(() => ({ error: '', status: 503 }))) as DatabaseView) : { error: '', status: 503 };
	}
	/** Databases live where apps are published: without it there's nothing to read. */
	readonly canRead = $derived(this.deploy?.available !== false);
	editingRows = $state(false);
	readonly browsed = $derived(this.database && !('error' in this.database) && this.database.rows && this.database.table ? { table: this.database.table, rows: this.database.rows } : null);
	readonly dbKey = $derived(this.browsed?.table.primary_key?.length ? this.browsed.table.primary_key : null);
	saveCell = async (row: number, column: number, value: string): Promise<boolean> => {
		if (!this.browsed || !this.dbKey) return false;
		const { table, rows: shown } = this.browsed;
		const raw = shown.rows[row];
		const key = Object.fromEntries(this.dbKey.map((name) => [name, raw[shown.columns.indexOf(name)]]));
		const response = await fetch(`/projects/${this.project.id}/database/rows`, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ source: this.dbSource, schema: table.schema, table: table.name, key, changes: { [shown.columns[column]]: value } }),
		}).catch(() => null);
		const body = response ? ((await response.json().catch(() => null)) as { row?: Record<string, unknown>; error?: string } | null) : null;
		if (!response?.ok || !body?.row) {
			toasts.show({ kind: 'error', title: m.database_edit_failed(), detail: body?.error || undefined });
			return false;
		}
		const kept = body.row;
		shown.rows[row] = shown.columns.map((name, at) => (name in kept ? kept[name] : raw[at]));
		toasts.show({ kind: 'success', title: m.database_saved() });
		return true;
	}

	// ── Settings: transfer, take down, delete ─────────────────────────────────────────────────
	dialog = $state<'transfer' | 'takedown' | 'delete' | null>(null);
	targets = $state<TransferTarget[]>([]);
	targetQuery = $state('');
	target = $state<string | null>(null);
	confirmName = $state('');
	acting = $state(false);
	actError = $state<string>();
	openDialog(which: 'transfer' | 'takedown' | 'delete') {
		this.confirmName = '';
		this.actError = undefined;
		this.target = null;
		this.dialog = which;
		if (which === 'transfer') void this.findTargets('');
	}
	findTimer: ReturnType<typeof setTimeout> | undefined;
	async findTargets(query: string) {
		const response = await fetch(`/projects/${this.project.id}/transfer?q=${encodeURIComponent(query)}`).catch(() => null);
		this.targets = response?.ok ? ((await response.json()) as TransferTarget[]) : [];
	}
	async act(name: 'transfer' | 'takedown' | 'delete', fields: Record<string, string>) {
		this.acting = true;
		this.actError = undefined;
		const result = await pageAction(`/projects/${this.project.id}`, name, fields);
		this.acting = false;
		if (!result.ok) return void (this.actError = result.message || m.action_failed());
		if (name === 'takedown') {
			this.dialog = null;
			toasts.show({ kind: 'success', title: m.takedown_done({ name: this.project.name }) });
			await invalidate('app:workspace-project');
			await this.refresh();
		}
	}

	// ── People and Environment ────────────────────────────────────────────────────────────────
	readonly people = $derived(this.data.people);
	readonly secrets = $derived(this.data.secrets);
	readonly projectAdmin = $derived(this.people?.my_role === 'admin');
	readonly roles = $derived([
		{ value: 'admin', label: m.people_role_admin() },
		{ value: 'member', label: m.people_role_member() },
		{ value: 'viewer', label: m.people_role_viewer() },
	]);
	inviteEmail = $state('');
	inviteRole = $state<'build' | 'prompt' | 'view'>('build');
	peopleError = $state<string>();
	busyPeople = $state(false);
	/** The last invite sent: the phone shows its link to copy (Invite & build together, step 3). */
	invited = $state<InviteSent | null>(null);
	async peopleAct(name: 'invite' | 'role' | 'remove' | 'access', fields: Record<string, string>, done: string): Promise<boolean> {
		this.busyPeople = true;
		this.peopleError = undefined;
		const result = await pageAction(`/projects/${this.project.id}`, name, fields);
		this.busyPeople = false;
		if (!result.ok) {
			if (name === 'invite') this.peopleError = result.message || m.action_failed();
			else if (result.message) toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message });
			return false;
		}
		if (name === 'invite') {
			const sent = result.data as { invited?: string; added?: boolean; path?: string | null; expires_at?: string | null };
			this.invited = { email: sent.invited ?? this.inviteEmail.trim(), added: sent.added ?? false, path: sent.path ?? null, expiresAt: sent.expires_at ?? null };
			this.inviteEmail = '';
		}
		toasts.show({ kind: 'success', title: done });
		await invalidate('app:workspace-project');
		return true;
	}
	envName = $state('');
	envValue = $state('');
	envError = $state<{ field?: string; message: string }>();
	savingEnv = $state(false);
	saveSecret = async (): Promise<boolean> => {
		this.savingEnv = true;
		this.envError = undefined;
		const result = await pageAction(`/projects/${this.project.id}`, 'secret', { name: this.envName, value: this.envValue });
		this.savingEnv = false;
		if (!result.ok) {
			this.envError = { field: result.field, message: result.message || m.action_failed() };
			return false;
		}
		toasts.show({ kind: 'success', title: m.env_saved({ name: this.envName.trim().toUpperCase() }) });
		this.envName = '';
		this.envValue = '';
		await invalidate('app:workspace-project');
		return true;
	}
	async removeSecret(name: string) {
		const result = await pageAction(`/projects/${this.project.id}`, 'unsecret', { name });
		if (!result.ok) return void (result.message && toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message }));
		toasts.show({ kind: 'success', title: m.env_removed({ name }) });
		await invalidate('app:workspace-project');
	}

	// ── Share and Export ──────────────────────────────────────────────────────────────────────
	sharing = $state(false);
	showcased = $state(false);
	async setShown(next: boolean) {
		const before = this.showcased;
		this.showcased = next;
		const result = await pageAction(`/projects/${this.project.id}`, 'showcase', { showcase: String(next) });
		if (!result.ok) {
			this.showcased = before;
			if (result.message) toasts.show({ kind: 'error', title: m.action_failed(), detail: result.message });
		}
	}
	exporting = $state(false);
	summary = $state<ExportSummary | null>(null);
	openExport = async () => {
		this.exporting = true;
		const response = await fetch(`/projects/${this.project.id}/export/summary`).catch(() => null);
		this.summary = response?.ok ? ((await response.json()) as ExportSummary) : null;
	}
	pushing = $state(false);
	repo = $state('');
	createRepo = $state(true);
	privateRepo = $state(true);
	pushError = $state<string>();
	sendingPush = $state(false);
	push = async () => {
		this.sendingPush = true;
		this.pushError = undefined;
		const result = await pageAction(`/projects/${this.project.id}`, 'github', { repo: this.repo, create: String(this.createRepo), private: String(this.privateRepo) });
		this.sendingPush = false;
		if (!result.ok) return void (this.pushError = result.message || m.action_failed());
		this.pushing = false;
		toasts.show({ kind: 'success', title: m.github_pushed({ repo: this.repo.trim() }) });
	}

	view = $state<PaneView>('preview');
	device = $state<'desktop' | 'phone'>('desktop');
	reloads = $state(0);
	sheet = $state<ToolSheet | null>(null);
	/** A `?open=` the workspace was opened with; the layout on show acts on it once. */
	opened = $state<OpenRequest | null>(null);

	// ── Code ──────────────────────────────────────────────────────────────────────────────────
	workspaceView = $state<'chat' | 'pane'>('chat');
	codePath = $state<string | null>(null);
	codeText = $state<string | null>(null);
	/** The file's lines as the server highlighted them; null until it's read (or when it couldn't be). */
	codeLines = $state<CodeLine[] | null>(null);
	codeBinary = $state(false);
	quote = $state<ReturnType<typeof quoteLines> | null>(null);
	readonly tree = $derived(fileTree(this.project.files, this.codePath));
	/** The phone's Code screen is open on a file (the desktop's Code view reads `view`). */
	phoneFile = $state(false);
	readonly codeOpen = $derived(this.view === 'code' || this.phoneFile);
	openFile = (path: string) => {
		this.codeText = null;
		this.codeLines = null;
		this.codePath = path;
	}
	async readFile(path: string) {
		const response = await fetch(`/projects/${this.project.id}/file?path=${encodeURIComponent(path)}`).catch(() => null);
		const file = response?.ok ? ((await response.json()) as { content?: string; binary: boolean; lines?: CodeLine[] }) : null;
		if (path !== this.codePath) return;
		this.codeBinary = file?.binary ?? false;
		this.codeText = file && !file.binary ? (file.content ?? '') : null;
		this.codeLines = file?.lines ?? null;
	}
	askAbout = (from: number, to: number) => {
		if (!this.codePath || this.codeText === null) return;
		this.quote = quoteLines(this.codePath, this.codeText.split('\n'), from, to);
		this.workspaceView = 'chat';
	}
	askAboutFile = () => {
		if (!this.codePath) return;
		this.quote = { file: this.codePath, range: m.code_whole_file(), snippet: '', text: m.code_about_file({ file: this.codePath }) };
		this.workspaceView = 'chat';
	}


	// ── Building together ─────────────────────────────────────────────────────────────────────
	live = $state<ProjectLive | null>(null);
	readonly where = $derived({ preview: m.workspace_preview(), code: m.workspace_code(), terminal: m.workspace_terminal() });
	readonly myName = $derived(this.data.person?.name ?? '');
	/** Faces are told apart by their initials: yours and everyone else's, made distinct together. */
	readonly faces = $derived(distinctInitials([this.myName, ...(this.live?.people ?? []).map((person) => person.name)]));
	/** A teammate's colour comes from their name, as in the chat. */
	readonly peers = $derived((this.live?.people ?? []).map((person, index) => ({ ...person, initials: this.faces[index + 1], peer: peerFor(person.name) })));
	following = $state<string | null>(null);
	readonly followed = $derived(this.peers.find((person) => person.user_id === this.following) ?? null);
	/** Following: go where they go (the terminal only where this workspace has one). */
	readonly followedView = $derived(this.followed ? ((this.followed.view ?? 'preview') as PaneView) : null);
	readonly followedPath = $derived(this.followed?.path ?? null);
	pointerClock = $state(Date.now());
	readonly pointers = $derived(this.live ? pointersIn(this.live.pointers, this.view, this.pointerClock).map((pointer) => ({ ...pointer, peer: peerFor(pointer.name) })) : []);
	trackPointer = (event: PointerEvent) => {
		const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
		this.live?.point(this.view, (event.clientX - box.left) / box.width, (event.clientY - box.top) / box.height);
	}

	// ── Asks in line ──────────────────────────────────────────────────────────────────────────
	asks = $state<QueuedAsk[]>([]);
	#readingQueue: Promise<void> | null = null;
	#queueAgain = false;
	loadQueue(): Promise<void> {
		if (this.#readingQueue) {
			this.#queueAgain = true;
			return this.#readingQueue;
		}
		this.#readingQueue = (async () => {
			do {
				this.#queueAgain = false;
				const response = await fetch(`/projects/${this.project.id}/queue`).catch(() => null);
				// The line is a nicety: when it can't be read, the last one stays until the chat moves on.
				if (response?.ok) this.asks = ((await response.json()) as { asks: QueuedAsk[] }).asks;
			} while (this.#queueAgain);
			this.#readingQueue = null;
		})();
		return this.#readingQueue;
	}
	readonly viewer = $derived(this.deploy?.role === 'viewer');
	readonly line = $derived(askLine(this.asks));
	readonly movable = $derived(this.viewer ? -1 : movableAsk(this.asks));
	async letFirst(index: number) {
		const ask = this.asks[index];
		if (!ask) return;
		const response = await fetch(`/projects/${this.project.id}/queue/${encodeURIComponent(ask.id)}`, { method: 'POST' }).catch(() => null);
		const body = response ? ((await response.json().catch(() => null)) as { asks?: QueuedAsk[]; message?: string } | null) : null;
		if (response?.ok && body?.asks) return void (this.asks = body.asks);
		toasts.show({ kind: 'error', title: m.queue_move_failed(), detail: body?.message || undefined });
		void this.loadQueue();
	}

	// ── Requests to publish ───────────────────────────────────────────────────────────────────
	readonly me = $derived(this.people?.me ?? null);
	readonly role = $derived(this.deploy?.role ?? null);
	requestRows = $state<PublishRequestRow[]>([]);
	async loadRequests() {
		const response = await fetch(`/projects/${this.project.id}/requests`).catch(() => null);
		if (!response?.ok) return;
		const next = ((await response.json()) as { requests: PublishRequestRow[] }).requests;
		for (const answer of answeredRequests(this.requestRows, next, this.me)) {
			toasts.show(
				answer.status === 'approved'
					? { kind: 'success', title: m.publish_request_approved({ name: answer.by }) }
					: { kind: 'info', title: m.publish_request_declined({ name: answer.by }), detail: m.publish_request_answer_note() },
			);
		}
		this.requestRows = next;
	}
	readonly theirs = $derived(this.role === 'admin' ? openRequests(this.requestRows, this.me) : []);
	readonly mine = $derived(this.role === 'member' ? myOpenRequest(this.requestRows, this.me) : null);
	mineOpen = $state(false);
	asking = $state(false);
	askPublish = async () => {
		this.asking = true;
		const result = await pageAction(`/projects/${this.project.id}`, 'ask');
		this.asking = false;
		if (!result.ok) return void (result.message && toasts.show({ kind: 'error', title: m.publish_ask_failed(), detail: result.message }));
		toasts.show({ kind: 'success', title: m.publish_asked_toast() });
		await this.loadRequests();
	}
	answering = $state<string | null>(null);
	async answerRequest(row: PublishRequestRow, action: 'approve' | 'decline' | 'cancel') {
		if (this.answering) return;
		this.answering = row.id;
		const result = await pageAction(`/projects/${this.project.id}`, 'answer', { request: row.id, action });
		this.answering = null;
		const name = row.requested_by_name.split(/\s+/)[0];
		if (!result.ok) {
			// Approving publishes: where the publish can't start, the request stays open and the backend says why.
			if (result.message) toasts.show({ kind: 'error', title: action === 'approve' ? m.deploy_couldnt_start() : m.publish_request_answer_failed(), detail: result.message });
		} else if (action === 'approve') {
			toasts.show({ kind: 'success', title: m.publish_request_publishing({ name }) });
		} else if (action === 'decline') {
			toasts.show({ kind: 'success', title: m.publish_request_declined_done({ name }) });
		} else {
			this.mineOpen = false;
			toasts.show({ kind: 'success', title: m.publish_request_cancelled() });
		}
		await this.loadRequests();
		if (result.ok && action === 'approve') await this.refresh();
	}
	review = () => {
		this.view = 'preview';
		this.workspaceView = 'pane';
	}
}
