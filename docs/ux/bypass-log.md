# UX gate bypass log

Every `NOMI_UX_BYPASS=1` edit gets a line. ux-lead reviews these weekly and opens
a proposal for anything that should have gone through the gates.

| Date | Who | Files | Why it couldn't wait | Follow-up proposal |
|---|---|---|---|---|
| 2026-11-17 | Trian | `frontend-new/src/lib/ds/agents/{crew,mascots,shapes}.ts`, `frontend-new/src/lib/ds/tokens/tokens.{css,json}`, `frontend/src/lib/components/ds/agents/{crew,mascots}.ts`, `frontend/src/lib/components/m3/shapes.ts`, `frontend-new/src/routes/design-system/(docs)/get-started/content/+page.svelte` | Sora (the design agent) is on for everyone since the crew switch landed, and showed as a bare initial; she is added the way Iris was, with no frame in Nomi.pen | Draw "Mascot / Sora" and the crew-sora colour (indigo #7F8FF4) in Nomi.pen, or correct them |
| 2026-11-17 | Trian | `frontend/src/lib/components/ProjectPreview.svelte` | The sandbox preview showed "refused to connect": the cross-origin isolated project page can't frame the sandbox's address unless the frame is credentialless. One attribute; no screen, state or copy changes | None needed |
| 2026-11-17 | Trian | `frontend/src/routes/admin/(protected)/{projects,sandboxes,projects/[id]/issues}/`, `frontend/src/lib/{adminProjects.ts,permissions.ts}`, `frontend/src/lib/components/AdminSidebar.svelte`, `frontend/messages/{en,id}.json`, `frontend-new/src/lib/admin/{access.ts,labels.ts}`, `frontend-new/messages/{en,id}.json` | Admin had no list of projects, sandboxes or a project's issues. Gate 1 couldn't run (no pencil MCP or Nomi.pen in the session); the owner approved bypassing. Adds Admin → Projects, Sandboxes and a project's issues (old app) and the "projects" admin area in both apps | Design the admin Projects, Sandboxes and issues pages in Nomi.pen and correct copy and layout; build them in frontend-new |
