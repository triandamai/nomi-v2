<script lang="ts">
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FormSection from '$lib/ds/admin/FormSection.svelte';
	import TextArea from '$lib/ds/admin/TextArea.svelte';
	import ConfirmDialog from '$lib/ds/admin/ConfirmDialog.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import Toggle from '$lib/ds/inputs/Toggle.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Support → an article (support: write; view shows it; deleting needs manage): its title, its
	// answer, tags, and whether the help desk answers from it.
	let { data, form } = $props();

	const canWrite = $derived(can(data.access, 'support', 'write'));
	const canManage = $derived(can(data.access, 'support', 'manage'));
	const existing = $derived(data.article?.ok ? data.article.data : null);
	const initial = () => (data.article?.ok ? data.article.data.published : true);
	let published = $state(initial());
	let deleting = $state(false);
	let deleteForm: HTMLFormElement | undefined = $state();
</script>

{#snippet save()}<Button label={existing ? m.admin_save() : m.admin_article_create()} variant="inverse" type="submit" />{/snippet}

{#if data.article && !data.article.ok}
	<PageHeader title={m.admin_page_support()} back={{ href: '/admin/support', label: m.admin_article_back() }} />
	<LoadFailed title={m.admin_article_failed()} problem={data.article.problem} />
{:else}
	<PageHeader title={existing?.title ?? m.admin_article_new()} back={{ href: '/admin/support', label: m.admin_article_back() }} />
	<form method="POST" action="?/save" use:enhance={() => ({ update }) => update({ reset: false })}>
		<FormSection title={m.admin_article_section()} description={m.admin_article_section_desc()} footer={canWrite ? save : undefined}>
			<div class="flex flex-col gap-4">
				<TextField label={m.admin_draft_title()} name="title" value={existing?.title ?? ''} required disabled={!canWrite} autocomplete="off" />
				<TextArea label={m.admin_draft_body()} name="body" value={existing?.body ?? ''} disabled={!canWrite} />
				<TextField label={m.admin_draft_tags()} name="tags" value={existing?.tags.join(', ') ?? ''} helper={m.admin_draft_tags_helper()} disabled={!canWrite} autocomplete="off" />
				<Toggle label={m.admin_article_publish()} bind:checked={published} disabled={!canWrite} />
				<input type="hidden" name="published" value={published ? 'on' : ''} />
			</div>
			<Outcome {form} action="save" done={m.admin_article_saved()} failed={m.admin_article_save_failed()} />
		</FormSection>
	</form>

	{#if existing && canManage}
		{#snippet remove()}<Button label={m.admin_article_delete()} variant="danger" onclick={() => (deleting = true)} />{/snippet}
		<FormSection title={m.admin_article_delete_section()} description={m.admin_article_delete_desc()} footer={remove} danger>
			<Outcome {form} action="delete" failed={m.admin_article_delete_failed()} />
		</FormSection>
		<form bind:this={deleteForm} method="POST" action="?/delete" use:enhance hidden></form>
		<ConfirmDialog
			bind:open={deleting}
			title={m.admin_article_delete_title({ title: existing.title })}
			body={m.admin_article_delete_body()}
			confirmLabel={m.admin_article_delete()}
			cancelLabel={m.admin_cancel()}
			onconfirm={() => {
				deleting = false;
				deleteForm?.requestSubmit();
			}}
		/>
	{/if}
{/if}
