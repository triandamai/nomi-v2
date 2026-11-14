<script lang="ts">
	import { Play, Search } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import type { KeySource, ReadPageSettings, SearchTest, SpeakSettings, ToolEntry, WebSearchSettings } from '$lib/admin/api';
	import { full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import Select from '$lib/ds/admin/Select.svelte';
	import SecretField from '$lib/ds/admin/SecretField.svelte';
	import Notice from '$lib/ds/admin/Notice.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Tools (tools: view; write changes): every tool, grouped by who uses it, each switched on or off
	// where it can be; then the settings of the ones that have them: web search (and trying it), how
	// much of a page is read, and the voice (and hearing it). Keys are written, never read back;
	// where a key comes from (saved here, the environment, or nowhere) is said.
	let { data, form } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'tools', 'write'));
	const groups = $derived(data.tools.ok ? data.tools.data : []);
	const tool = (name: string): ToolEntry | undefined => groups.flatMap((group) => group.tools).find((one) => one.name === name);
	const search = $derived(tool('web_search')?.settings as WebSearchSettings | undefined);
	const reading = $derived(tool('read_web_page')?.settings as ReadPageSettings | undefined);
	const voice = $derived(tool('speak')?.settings as SpeakSettings | undefined);

	const keyStatus = (source: KeySource, env: string) => (source === 'saved' ? m.admin_key_saved() : source === 'env' ? m.admin_key_env({ env }) : m.admin_key_missing({ env }));

	const initialProvider = () => (tool('web_search')?.settings as WebSearchSettings | undefined)?.provider ?? 'tavily';
	let provider = $state(initialProvider());
	const providerInfo = $derived(search?.providers.find((one) => one.id === provider));

	const initialBoost = () => (tool('speak')?.settings as SpeakSettings | undefined)?.use_speaker_boost ?? true;
	let boost = $state(initialBoost());

	const tried = $derived(form?.action === 'testSearch' && form.ok ? (form.data as SearchTest) : null);

	// Hearing the voice: the recording comes from the voice test endpoint and plays here.
	let line = $state('');
	let hearing = $state(false);
	let heard = $state<string | null>(null);
	let hearFailed = $state<string | null>(null);
	async function hear() {
		hearing = true;
		hearFailed = null;
		const response = await fetch('/admin/tools/voice-test', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ text: line }) }).catch(() => null);
		hearing = false;
		if (!response?.ok) {
			hearFailed = ((await response?.json().catch(() => null)) as { error?: string } | null)?.error || m.admin_unreachable();
			return;
		}
		if (heard) URL.revokeObjectURL(heard);
		heard = URL.createObjectURL(await response.blob());
	}
</script>

{#snippet save()}<Button label={m.admin_save()} variant="inverse" type="submit" />{/snippet}

<PageHeader title={m.admin_page_tools()} intro={m.admin_tools_intro()} />

{#if data.tools.ok}
	<Outcome {form} action="toggle" failed={m.admin_tool_toggle_failed()} />
	{#each groups as group (group.key)}
		<section aria-label={group.label} class="flex flex-col gap-2">
			<SectionLabel label={group.label} count={group.tools.length} />
			<ul class="flex flex-col rounded-md outline -outline-offset-1 outline-border">
				{#each group.tools as entry (entry.name)}
					<li class={['flex flex-wrap items-center gap-x-4 gap-y-1.5 border-b border-border px-4 py-3 last:border-b-0', !entry.enabled && 'opacity-60']}>
						<div class="flex min-w-0 flex-1 basis-80 flex-col gap-0.5">
							<span class="font-mono text-ui text-text">{entry.name}</span>
							<span class="text-caption text-text-2">{entry.description}</span>
							{#if entry.used_by.length > 0}<span class="font-mono text-label text-text-3">{m.admin_tool_used_by({ agents: entry.used_by.join(', ') })}</span>{/if}
						</div>
						<span class="flex flex-wrap items-center gap-1.5">
							{#if entry.required}<Tag kind="neutral" label={m.admin_tool_always_on()} />{/if}
							{#if entry.agent_only}<Tag kind="neutral" label={m.admin_tool_agent_only()} />{/if}
							{#if !entry.ready}<Tag kind="neutral" label={m.admin_tool_not_ready()} />{/if}
						</span>
						{#if canWrite && !entry.required && !entry.agent_only}
							<form method="POST" action="?/toggle" use:enhance>
								<input type="hidden" name="name" value={entry.name} />
								<input type="hidden" name="enabled" value={entry.enabled ? '' : 'on'} />
								<Button
									label={entry.enabled ? m.admin_model_switch_off() : m.admin_model_switch_on()}
									variant="ghost"
									type="submit"
									aria-label={entry.enabled ? m.admin_tool_turn_off({ tool: entry.name }) : m.admin_tool_turn_on({ tool: entry.name })}
								/>
							</form>
						{:else}
							<span class="font-mono text-label text-text-2">{entry.enabled ? m.admin_model_on() : m.admin_model_off()}</span>
						{/if}
					</li>
				{/each}
			</ul>
		</section>
	{/each}

	{#if search}
		<form method="POST" action="?/saveSearch" use:enhance={() => ({ update }) => update({ reset: false })}>
			<FormSection title={m.admin_search_section()} description={m.admin_search_desc()} footer={canWrite ? save : undefined}>
				<div class="grid gap-4 sm:grid-cols-2">
					<Select label={m.admin_search_provider()} name="provider" bind:value={provider} options={search.providers.map((one) => ({ value: one.id, label: one.id }))} mono disabled={!canWrite} />
					<TextField
						label={m.admin_search_results()}
						name="max_results"
						value={String(search.max_results)}
						helper={m.admin_search_results_helper({ max: String(search.max_results_limit) })}
						mono
						disabled={!canWrite}
						autocomplete="off"
					/>
					{#if provider === 'searxng'}
						<div class="sm:col-span-2">
							<TextField label={m.admin_search_base_url()} name="base_url" value={search.base_url ?? ''} helper={m.admin_search_base_url_helper()} mono disabled={!canWrite} autocomplete="off" />
						</div>
					{/if}
					{#if providerInfo?.needs_key}
						<div class="sm:col-span-2">
							<SecretField
								label={m.admin_search_key()}
								name="api_key"
								set={providerInfo.key_source !== 'missing'}
								status={keyStatus(providerInfo.key_source, providerInfo.env_var)}
								forget={providerInfo.key_source === 'saved' ? { name: 'clear_api_key', label: m.admin_key_forget() } : undefined}
								disabled={!canWrite}
							/>
						</div>
					{/if}
				</div>
				<Outcome {form} action="saveSearch" done={m.admin_search_saved()} failed={m.admin_search_failed()} />
			</FormSection>
		</form>
		{#if canWrite}
			<form method="POST" action="?/testSearch" use:enhance={() => ({ update }) => update({ reset: false })} class="flex flex-col gap-3">
				<div class="flex flex-wrap items-end gap-3">
					<div class="min-w-60 flex-1"><TextField label={m.admin_search_query()} name="query" placeholder={m.admin_search_query_placeholder()} autocomplete="off" /></div>
					<Button label={m.admin_search_try()} variant="secondary" iconStart={Search} type="submit" />
				</div>
				{#if tried}
					{#if tried.ok}
						<Notice kind="success" text={m.admin_search_tried({ count: full(tried.results.length, language), provider: tried.provider })} />
						<ol class="flex flex-col gap-2">
							{#each tried.results as result, index (index)}
								<li class="flex flex-col gap-0.5">
									<span class="text-ui font-medium text-text">{result.title}</span>
									<span class="truncate font-mono text-label text-text-3">{result.url}</span>
									<span class="text-caption text-text-2">{result.snippet}</span>
								</li>
							{/each}
						</ol>
					{:else}
						<Notice kind="error" text={m.admin_search_try_failed()} detail={tried.error ?? undefined} />
					{/if}
				{/if}
				<Outcome {form} action="testSearch" failed={m.admin_search_try_failed()} />
			</form>
		{/if}
	{/if}

	{#if reading}
		<form method="POST" action="?/saveReadPage" use:enhance={() => ({ update }) => update({ reset: false })}>
			<FormSection title={m.admin_read_section()} description={m.admin_read_desc()} footer={canWrite ? save : undefined}>
				<div class="max-w-60">
					<TextField label={m.admin_read_chars()} name="max_chars" value={String(reading.max_chars)} helper={m.admin_read_chars_helper()} mono disabled={!canWrite} autocomplete="off" />
				</div>
				<Outcome {form} action="saveReadPage" done={m.admin_read_saved()} failed={m.admin_read_failed()} />
			</FormSection>
		</form>
	{/if}

	{#if voice}
		<form method="POST" action="?/saveVoice" use:enhance={() => ({ update }) => update({ reset: false })}>
			<FormSection title={m.admin_voice_section()} description={m.admin_voice_desc()} footer={canWrite ? save : undefined}>
				<div class="grid gap-4 sm:grid-cols-2">
					{#if data.voices && data.voices.voices.length > 0}
						<Select
							label={m.admin_voice_voice()}
							name="voice_id"
							value={voice.voice_id}
							options={data.voices.voices.map((one) => ({ value: one.voice_id, label: one.name }))}
							disabled={!canWrite}
						/>
					{:else}
						<TextField label={m.admin_voice_voice_id()} name="voice_id" value={voice.voice_id} mono disabled={!canWrite} autocomplete="off" />
					{/if}
					<input type="hidden" name="voice_name" value={data.voices?.voices.find((one) => one.voice_id === voice.voice_id)?.name ?? voice.voice_name ?? ''} />
					<Select
						label={m.admin_voice_model()}
						name="model_id"
						value={voice.model_id}
						options={(data.voices?.models.length ? data.voices.models : voice.models).map((one) => ({ value: one.model_id, label: one.name }))}
						disabled={!canWrite}
					/>
					{#if !data.voices}<div class="sm:col-span-2"><Notice kind="info" text={m.admin_voice_voices_unavailable()} /></div>{/if}
					<TextField label={m.admin_voice_stability()} name="stability" value={String(voice.stability)} helper={m.admin_voice_zero_one()} mono disabled={!canWrite} autocomplete="off" />
					<TextField label={m.admin_voice_similarity()} name="similarity_boost" value={String(voice.similarity_boost)} helper={m.admin_voice_zero_one()} mono disabled={!canWrite} autocomplete="off" />
					<TextField label={m.admin_voice_style()} name="style" value={String(voice.style)} helper={m.admin_voice_zero_one()} mono disabled={!canWrite} autocomplete="off" />
					<TextField label={m.admin_voice_speed()} name="speed" value={String(voice.speed)} helper={m.admin_voice_speed_helper()} mono disabled={!canWrite} autocomplete="off" />
					<Select label={m.admin_voice_format()} name="output_format" value={voice.output_format} options={voice.output_formats.map((one) => ({ value: one, label: one }))} mono disabled={!canWrite} />
					<TextField
						label={m.admin_voice_chars()}
						name="max_chars"
						value={String(voice.max_chars)}
						helper={m.admin_voice_chars_helper({ max: full(voice.max_chars_limit, language) })}
						mono
						disabled={!canWrite}
						autocomplete="off"
					/>
					<div class="sm:col-span-2">
						<Toggle label={m.admin_voice_boost()} bind:checked={boost} disabled={!canWrite} />
						<input type="hidden" name="use_speaker_boost" value={boost ? 'on' : ''} />
					</div>
					<div class="sm:col-span-2">
						<SecretField
							label={m.admin_voice_key()}
							name="api_key"
							set={voice.key_source !== 'missing'}
							status={keyStatus(voice.key_source, voice.env_var)}
							forget={voice.key_source === 'saved' ? { name: 'clear_api_key', label: m.admin_key_forget() } : undefined}
							disabled={!canWrite}
						/>
					</div>
				</div>
				<Outcome {form} action="saveVoice" done={m.admin_voice_saved()} failed={m.admin_voice_failed()} />
			</FormSection>
		</form>
		{#if canWrite}
			<div class="flex flex-col gap-3">
				<div class="flex flex-wrap items-end gap-3">
					<div class="min-w-60 flex-1"><TextField label={m.admin_voice_line()} bind:value={line} placeholder={m.admin_voice_line_placeholder()} autocomplete="off" /></div>
					<Button label={hearing ? m.admin_voice_saying() : m.admin_voice_hear()} variant="secondary" iconStart={Play} loading={hearing} onclick={hear} />
				</div>
				{#if hearFailed}<Notice kind="error" text={m.admin_voice_hear_failed()} detail={hearFailed} />{/if}
				<!-- svelte-ignore a11y_media_has_caption -->
				{#if heard}<audio controls autoplay src={heard} aria-label={m.admin_voice_recording()}></audio>{/if}
			</div>
		{/if}
	{/if}
{:else}
	<LoadFailed title={m.admin_tools_failed()} problem={data.tools.problem} />
{/if}
