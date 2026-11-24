---
work: phone-tabs
status: verified
verified_by: ux-lead rules (nomi-ux-rules), run in-session
verified_on: 19 Nov 2026
---

# UX contract — phone-tabs

On a phone (`phoneView`, below 768), at 390 in dark and light. The desktop doesn't change.

Ground rules:
- **Pen 1:1** from the frame ids in the brief. Where the pen's text is data (names, counts, times,
  titles), the real data goes in the same slot.
- **Own titles.** Home, Inbox and You draw their own top; the shell's top bar is hidden
  (`ownTitle`). The tab bar stays.
- **Targets.** 44 minimum. Where the pen draws a smaller control (the 32 To pill, the 36 Send), the
  touch area is drawn outside it so the card keeps the pen's size.
- **Sheets** are `BottomSheet`s: who a message goes to, the model, a timeline row's menu, New
  reminder.
- **Honest values.** A row or card shows a value only when the app has it; nothing is invented.

## Decisions (made from the UX rules, not asked)
| # | Decision | Why |
|---|---|---|
| D1 | You uses the settings nav's groups (Account: Profile, General · AI: Crew, Models & providers, Memory & context · Workspace: Workspaces, Connections). Plan & usage is the plan card, not a row. The pen was changed to match. | One information architecture across desktop and phone; the pen's "YOUR CREW" mixed AI settings with Connections. |
| D2 | "Log out", not the pen's "Sign out". Pen changed. | The account menu and the settings nav already say Log out. |
| D3 | Settings rows show a value only for the crew's size and the number of workspaces. | Those are in the shell; Models, Memory and Connections would each need a load for a decoration. |
| D4 | The Inbox shows the week ahead after Today on a phone. | Desktop shows it in the Coming up column, which a phone doesn't have; without it a reminder set for tomorrow vanished. |
| D5 | The all-clear line is "Nothing needs you · N coming up today"; with needs it reuses the desktop meta ("N need you · M coming up this week"). | Same numbers as the desktop; the pen's two lines. |
| D6 | A settings page opened from You gets a back link to You before "Settings › Page". | It's a screen inside the You tab; without it the only way back was the tab. |
| D7 | `/settings` is You on a phone; a desktop still opens Profile (server redirect by the browser's guess, client redirect when the window widens). | The tab needs its own screen; desktop Settings has a nav already. |

## Copy
| Where | en | id |
|---|---|---|
| To pill (name) | Send to {name}, change | Kirim ke {name}, ganti |
| To sheet title | Send to | Kirim ke |
| Home section | Today · updated {time} · Your crew | Hari ini · diperbarui {time} · Kru kamu |
| Inbox all clear line | Nothing needs you · {coming} coming up today | Tidak ada yang perlu kamu urus · {coming} lagi hari ini |
| Inbox all clear body | Approvals, invites and anything the crew can’t finish alone land here first. | Persetujuan, undangan, dan apa pun yang tidak bisa kru selesaikan sendiri muncul di sini lebih dulu. |
| You person line | @{username} · Profile | @{username} · Profil |
| Back from a settings page | Back to You | Kembali ke Kamu |
