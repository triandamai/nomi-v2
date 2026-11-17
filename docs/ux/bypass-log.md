# UX gate bypass log

Every `NOMI_UX_BYPASS=1` edit gets a line. ux-lead reviews these weekly and opens
a proposal for anything that should have gone through the gates.

| Date | Who | Files | Why it couldn't wait | Follow-up proposal |
|---|---|---|---|---|
| 2026-11-17 | Trian | `frontend-new/src/lib/ds/agents/{crew,mascots,shapes}.ts`, `frontend-new/src/lib/ds/tokens/tokens.{css,json}`, `frontend/src/lib/components/ds/agents/{crew,mascots}.ts`, `frontend/src/lib/components/m3/shapes.ts`, `frontend-new/src/routes/design-system/(docs)/get-started/content/+page.svelte` | Sora (the design agent) is on for everyone since the crew switch landed, and showed as a bare initial; she is added the way Iris was, with no frame in Nomi.pen | Draw "Mascot / Sora" and the crew-sora colour (indigo #7F8FF4) in Nomi.pen, or correct them |
| 2026-11-17 | Trian | `frontend/src/lib/components/ProjectPreview.svelte` | The sandbox preview showed "refused to connect": the cross-origin isolated project page can't frame the sandbox's address unless the frame is credentialless. One attribute; no screen, state or copy changes | None needed |
| 2026-11-17 | Trian | `frontend-new/src/lib/projects/{SandboxWait.svelte,SandboxWait.svelte.test.ts,sandbox.ts,sandbox.svelte.ts}`, `frontend-new/src/routes/(app)/projects/[id]/+page.svelte`, `frontend-new/messages/{en,id}.json` | frontend-new's preview said "Waking the sandbox" while the crew was still building, because it started a sandbox the backend wouldn't start, and showed no progress. Gate 1 couldn't run (no pencil MCP or Nomi.pen in the session); the owner approved bypassing. Adds a "Waiting for the first build" state (the old app's wording) and the last line the sandbox printed under the waking/starting states | Run gate 1 on Code, terminal & phone preview (uR6dq): draw the first-build wait and the progress line, and correct the copy |
