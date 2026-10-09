<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { contentUrl, formatBytes, previewUrl, type Attachment } from '$lib/attachments';
	import AttachmentIcon from './AttachmentIcon.svelte';

	// The files of a sent message: images as previews (tap for the full file), video and audio
	// as players, everything else as a card that downloads it. Older messages carried text files
	// inline; those open as a preview of their text.
	let { files }: { files: Attachment[] } = $props();

	let openText = $state<number | null>(null);
	const images = $derived(files.filter((f) => f.id && f.kind === 'image'));
	const others = $derived(files.filter((f) => !(f.id && f.kind === 'image')));
	let broken = $state<Record<string, boolean>>({});
</script>

{#if images.length > 0}
	<div class="images" class:images--grid={images.length > 1}>
		{#each images as image (image.id)}
			<a class="image attachment-link" href={contentUrl(image.id!)} target="_blank" rel="noopener" aria-label={m.msg_open_file({ name: image.name })}>
				{#if broken[image.id!]}
					<span class="image__fallback"><AttachmentIcon kind="image" size={22} />{image.name}</span>
				{:else}
					<img src={previewUrl(image.id!)} alt={image.name} loading="lazy" onerror={() => (broken = { ...broken, [image.id!]: true })} />
				{/if}
			</a>
		{/each}
	</div>
{/if}

{#if others.length > 0}
	<div class="files">
		{#each others as file, i (file.id ?? `${file.name}-${i}`)}
			{#if file.id && file.kind === 'video'}
				<figure class="media">
					<!-- svelte-ignore a11y_media_has_caption -->
					<video controls preload="metadata" src={contentUrl(file.id)}></video>
					<figcaption>{file.name}</figcaption>
				</figure>
			{:else if file.id && (file.kind === 'audio' || file.kind === 'voice')}
				<figure class="media media--audio" class:media--voice={file.kind === 'voice'}>
					<figcaption><AttachmentIcon kind={file.kind} size={16} />{file.name}</figcaption>
					<audio controls preload="metadata" src={contentUrl(file.id)}></audio>
				</figure>
			{:else if file.id}
				<a class="card attachment-link" href={contentUrl(file.id)} download={file.name}>
					<span class="card__icon"><AttachmentIcon kind={file.kind} size={20} /></span>
					<span class="card__text">
						<span class="card__name">{file.name}</span>
						{#if file.size}<span class="card__meta">{formatBytes(file.size)}</span>{/if}
					</span>
				</a>
			{:else}
				<button type="button" class="card" aria-expanded={openText === i} onclick={() => (openText = openText === i ? null : i)}>
					<span class="card__icon"><AttachmentIcon kind={file.kind === 'voice' ? 'voice' : 'text'} size={20} /></span>
					<span class="card__text">
						<span class="card__name">{file.name}</span>
						<span class="card__meta">{formatBytes(new Blob([file.text ?? '']).size)}</span>
					</span>
				</button>
			{/if}
		{/each}
	</div>
	{#if openText !== null && others[openText]?.text !== undefined}
		<pre class="preview">{others[openText].text}</pre>
	{/if}
{/if}

<style>
	.images {
		display: grid;
		gap: 4px;
		max-width: min(360px, 100%);
		margin-bottom: 8px;
		border-radius: var(--md-sys-shape-corner-large);
		overflow: hidden;
	}
	.images--grid {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
	.image {
		display: block;
		background: color-mix(in srgb, currentColor 10%, transparent);
	}
	.image img {
		display: block;
		width: 100%;
		max-height: 320px;
		object-fit: cover;
	}
	.images--grid .image img {
		aspect-ratio: 1;
	}
	.image:focus-visible {
		outline: 2px solid currentColor;
		outline-offset: -2px;
	}
	.image__fallback {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 16px;
		font-size: 0.8125rem;
	}
	.files {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
		margin-bottom: 4px;
	}
	.media {
		margin: 0;
		max-width: min(360px, 100%);
	}
	.media video {
		display: block;
		width: 100%;
		max-height: 300px;
		border-radius: var(--md-sys-shape-corner-large);
		background: #000;
	}
	.media figcaption {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 4px;
		font-size: 0.75rem;
		opacity: 0.8;
	}
	.media--audio {
		display: flex;
		flex-direction: column;
		gap: 4px;
		width: min(320px, 100%);
		padding: 8px 10px;
		border-radius: var(--md-sys-shape-corner-large);
		background: color-mix(in srgb, currentColor 10%, transparent);
	}
	.media--audio figcaption {
		margin: 0;
		font-weight: 650;
	}
	.media--audio audio {
		width: 100%;
		height: 36px;
	}
	.card {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		max-width: min(320px, 100%);
		min-height: 48px;
		padding: 6px 14px 6px 8px;
		border: none;
		border-radius: var(--md-sys-shape-corner-large);
		background: color-mix(in srgb, currentColor 12%, transparent);
		color: inherit;
		font: inherit;
		text-align: left;
		text-decoration: none;
		cursor: pointer;
	}
	.card:focus-visible {
		outline: 2px solid currentColor;
		outline-offset: 2px;
	}
	.card__icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex: none;
		width: 36px;
		height: 36px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: color-mix(in srgb, currentColor 14%, transparent);
	}
	.card__text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	.card__name {
		font-size: 0.875rem;
		font-weight: 650;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.card__meta {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		opacity: 0.75;
	}
	.preview {
		margin: 8px 0 0;
		max-height: 260px;
		overflow: auto;
		padding: 10px 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: color-mix(in srgb, currentColor 10%, transparent);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
</style>
