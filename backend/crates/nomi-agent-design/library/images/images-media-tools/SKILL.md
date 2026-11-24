---
name: images-media-tools
description: "A catalogue of image, video, audio, capture and document tools you cannot run yourself (fal, Venice, Replicate, OpenAI and Google image models, OpenAI Sora video, TTS, music, Remotion, screenshot and PDF/DOCX/PPTX tools): what each does, when a design calls for it, what to write or suggest."
license: "Apache-2.0, Copyright nexu-io open-design contributors (open-design catalogue entries and utilities). Upstream tools are named for reference only and described in our own words: fal-ai-community/skills (MIT), veniceai/skills (MIT), MiniMax-AI/skills (MIT), Leonxlnx/taste-skill (MIT), heygen-com/hyperframes (Apache-2.0); openai/skills, anthropics/skills, op7418/NanoBanana-PPT-Skills, replicate, remotion-dev/remotion and the others listed carry their own terms"
source: "https://github.com/nexu-io/open-design/tree/5fdcc3d/skills (fal-*, venice-*, replicate, imagegen, imagegen-frontend-web, imagegen-frontend-mobile, imagen, nanobanana-ppt, pixelbin-media, image-enhancer, gif-sticker-maker, speech, ai-music-album, remotion, video-hyperframes, video-downloader, youtube-clipper, slack-gif-creator, screenshot, full-page-screenshot, agent-browser, sora, od-next-media-inputs, export-download-debugging, pdf, docx, minimax-docx, minimax-pdf, pptx, pptx-generator); https://github.com/nexu-io/open-design/tree/5fdcc3d/design-templates/audio-jingle; https://github.com/nexu-io/open-design/tree/5fdcc3d/clipper"
---
# Media tools you can't run

A catalogue of the generation, editing, capture and export tools a design may need, so you can describe the asset precisely in your placeholder note or suggest the right kind of tool to the person.

**Be honest:** you cannot call any of these. You draw on the canvas and write notes. When a design needs a photo, an illustration, a voiceover or a finished file, say plainly what is needed, write the brief (see `../images-visual-briefs/SKILL.md`), and tell the person which kind of tool or professional can make it. Never say an image "has been generated" or a file "has been exported". Never invent a price, a quota or a model name; costs and accounts are the person's to check with each provider.

**First choice is always real:** the person's own photos, a photographer, a licensed stock photo marked as a stand-in. Generated images are for illustrations, mood references and drafts; never pass a generated person off as a real customer or a generated product shot as the real product.

## Make an image

| Tool family | What it does | When a design calls for it | What you write or suggest |
|---|---|---|---|
| **fal: generate** | Text-to-image and video from many hosted models | Illustrations, backgrounds, mood references | Your full brief; the ratio from your frame (`4:5, 1080x1350`) |
| **fal: Kling** | A strong image and video model family | Cinematic stills or short clips | Shot brief with camera and motion |
| **fal: realtime** | Fast streaming drafts | Moodboard exploration before a shoot | 3-4 short direction variants |
| **fal: train (LoRA)** | Trains a custom style or character model from examples | A mascot or brand style that must stay consistent across many images | The character or style lock, and that 15-30 consistent reference images are needed |
| **fal: 3D** | 3D models from text or images | A product turntable, an AR preview | Product identity lock; the views needed |
| **Venice: image generate** | Image generation with preset styles | Same as above, privacy-minded provider | Brief + style name if the person picks one |
| **Replicate** | Runs many community models; good when models change often | Any of the above | Brief; let the person pick the model |
| **OpenAI image** | Generate and edit images | UI references, icons, illustrations, social art | Brief; note any text that must appear exactly |
| **Google Gemini image** | Generate images | Same | Brief |
| **Website / app image direction** | Makes one reference image per page section (web) or per screen (mobile) from a design direction | When the person wants a look to react to before you draw | Section list with the one idea each; your palette and type; "reference only, not final art" |
| **Pixelbin** | A large set of image and video APIs, plus page building | Batch edits, background swaps for a catalogue | The edit, applied to which files |
| **NanoBanana deck** | Turns a document into a styled deck with generated images | A quick image-heavy deck draft | The outline and the look; you still design the real deck |
| **Photo to animated sticker** | Turns a photo into a toy-style animated GIF sticker | A WhatsApp sticker pack of the owner or mascot | Character lock, the 8-12 poses and words |

## Edit or fix an image

| Tool | What it does | When | What you write |
|---|---|---|---|
| **fal / Venice image edit** | Remove background or objects, inpaint, style transfer | Cut out a product for a marketplace main image; clear a cluttered counter | `Edit · remove background, keep shadow soft, white #FFFFFF` |
| **fal restore** | Deblur, denoise, fix faces, restore old photos | An old shopfront photo for an "about us" story; a blurry phone shot | What to fix, what must stay untouched |
| **fal upscale / image enhancer** | Raise resolution and sharpness | The person's only photo is small; a screenshot for a deck | Target size from your frame (`at least 2160 px wide for a 1080 frame`) |
| **fal try-on** | Shows clothes on a person | A batik or hijab shop's lookbook draft | Garment photo + model brief; mark results as illustrations |
| **fal vision** | Detects, segments, reads text (OCR), describes images | Reading a menu photo into a price list, checking a product photo's details | What to extract |

## Make video

| Tool | What it does | When | What you write |
|---|---|---|---|
| **HyperFrames renderer** | Renders HTML compositions to MP4 | The way your storyboards become real videos (see `../../video/video-frame-templates/SKILL.md` and `../../video/video-hyperframes/SKILL.md`) | Your key frames with motion notes |
| **Remotion** | Code-driven video in React | Repeatable branded clips, data-to-video | Storyboard + timing; note that Remotion's own licence may require a company licence for business use |
| **OpenAI Sora (video)** | OpenAI's text-to-video model. Not you: same name, different thing | Cinematic b-roll, a mood clip | A shot list: subject, setting, camera move, light, length, ratio |
| **fal / Venice video** | Generate or remix clips; Venice also transcribes | B-roll, style remix | Same shot list |
| **fal video edit** | Restyle, upscale, remove background, add audio | Clean up the person's own phone footage | What to change, what to keep |
| **fal lip-sync** | Talking-head or lip-synced video from audio | An explainer avatar, a dubbed promo | Script, voice, framing; mark as synthetic |
| **YouTube clipper** | Pulls a long video, slices highlights, adds captions | Cutting reels from a cooking class or a podcast | Which moments, caption style, 1080x1920 |
| **Video downloader** | Downloads videos from platforms | Only for the person's own content | Remind: only footage they own or may use |
| **Slack GIF maker** | Short GIFs within Slack's size limits | An internal team emoji or celebration | Loop length, size, 2-3 frames described |

## Voice, music and sound

| Tool | What it does | When | What you write |
|---|---|---|---|
| **Text-to-speech (OpenAI, Venice, MiniMax, ElevenLabs, Fish)** | Spoken audio from a script | A reel voiceover, a narrated explainer, a class audio | The script in the person's language, voice described in words (`perempuan, hangat, logat netral, tempo sedang`), length; voice ids are provider-specific, so describe rather than name one |
| **Music generation (Suno, Udio, Lyria, Venice)** | Jingles, beds, loops | A promo's background bed, a brand sting | Genre, tempo (BPM), instruments, mood, length, where it must hit (`logo at 0:13`) |
| **Sound effects (ElevenLabs SFX, AudioCraft)** | Short effects | A whoosh on a transition, a cash-register ding | Texture, impact, duration |
| **AI music album** | Concept, lyrics, sequencing, export for a whole album | A brand soundtrack project, rarely | Concept and track list |

Audio is a brief, never drawn: put an `Audio` note frame beside the storyboard.

## Capture a web page or screen

| Tool | What it does | When | What you write |
|---|---|---|---|
| **Screenshot** | Captures the screen, a window or a region | The person's existing app or site for a "before" slide | What to capture, at what size |
| **Full-page screenshot** | Captures a whole web page | A portfolio case study, an audit report | The URL (theirs) and width (1440, 390) |
| **Browser automation** | Navigates, fills forms, clicks, screenshots, extracts data | Checking a live site before a redesign | The pages and states to capture |
| **Web clipper (extension)** | Saves images and page content from any site into a library | Collecting references | Only as reference; never reuse others' images or copy a site |
| **Screenshot framing (marketing)** | App-store style screenshots | Store listings | See `../../templates/templates-social-cards/SKILL.md` |

## Convert and export documents

| Tool | What it does | When | What you write |
|---|---|---|---|
| **PDF tools (incl. MiniMax PDF)** | Create, fill, reformat PDFs; styled covers | Price list, e-guide, invoice, one-pager as a file | Your page frames are the design; note page size A4 and bleed |
| **DOCX tools (incl. MiniMax DOCX)** | Create and edit Word files, tracked changes | Proposals and reports people will edit | Styles from your type scale (heading, body, caption) |
| **PPTX tools (incl. PptxGenJS pipelines)** | Build and edit PowerPoint decks | A deck the person must edit in PowerPoint | See `../../decks/decks-templates/SKILL.md` for what survives export |
| **Export troubleshooting** | Diagnoses failed image or file downloads in browsers and apps | When the person says a download is 0 KB or won't save | Suggest they report the browser and steps; you can't fix it from the canvas |
| **Media-input planning** | Gathers and measures the assets a plan needs, keeping each photo's source, author and licence | Before a build, listing every placeholder | A list of placeholders: id, size, subject, source (theirs, stock stand-in, generated illustration) |

## How to put this in your work

1. In the placeholder: the short caption (`Photo · …`). That is enough for most frames.
2. In the handoff or chat, when the person has no photo: one line on the options, in their language: "Foto ini paling bagus dari HP kamu sendiri di dekat jendela. Kalau mau, aku bisa tulis brief untuk fotografer atau untuk tool pembuat gambar."
3. For a set (marketplace images, sticker pack, video scenes): a numbered list of assets with sizes and briefs, so the person or a tool can work through it.

Adapted from open-design (nexu-io/open-design@5fdcc3d, Apache-2.0); upstream tools named for reference and described in our own words: fal-ai-community/skills (MIT), veniceai/skills (MIT), MiniMax-AI/skills (MIT), Leonxlnx/taste-skill (MIT), heygen-com/hyperframes (Apache-2.0), and others under their own terms. Changed: rewritten for the OpenPencil canvas.
