---
work: you-flow
status: designed
flows: Flow / Mobile — You & Settings (dark EtSi2, light UBr1i); Flow / You — Settings (Desktop) (dark ZhFyw, light d95Bu)
---

# You & Settings, desktop and phone

You is the phone's tab for the person: who they are, their plan, and every settings page. On a
desktop the same pages sit behind the account menu and the settings nav, and are built and wired
to the backend (Profile, General, Plan & usage, Crew, Models & providers, Memory & context,
Workspaces and a workspace, Connections). This work item makes the whole flow complete on both:

- **Phone:** each settings page as a phone screen, not the desktop page squeezed. Audit at 390
  (23 Nov 2026), against the real backend: no sideways scroll, but top bars collide
  ("Settings › Plan & usage" under "Export usage CSV"; "Connect a provider" over the title),
  label | control rows wrap into narrow columns (Models' defaults, General), Models' providers are a
  table, 1–8 targets per page under 32px, and Plan & usage is one long column.
- **Desktop:** the dialogs the app built without a design (change email and its code, change
  password, delete account, manage a connection) drawn over their pages.

## Phone, the way phones work
- You is the hub; each page is a screen pushed from it, back to You, with a deeper level where a
  task needs one (a crew member, a provider, a kind of memory, a workspace).
- Grouped rows instead of label | control columns: a row shows its value and opens what changes
  it: a **sheet** to pick (language, theme, time zone, who chats start with, a model, a role), a
  **pushed screen** to type (email, password, an API key). Switches sit at the row's end and
  save as they flip; a toast at the top says so.
- The main action sits at the bottom of a sheet or screen, above the keyboard while a field is
  open; 44pt targets; nothing important only in the top bar.
- Danger comes last, alone in its group, and asks first.
- Every page has its loading, empty, couldn't-load and saved states.
