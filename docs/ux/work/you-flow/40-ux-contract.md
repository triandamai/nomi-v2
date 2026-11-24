---
work: you-flow
status: verified
verified_by: ux-lead rules (nomi-ux-rules), run in-session
verified_on: 23 Nov 2026
---

# UX contract: you-flow

Build exactly this. Frame ids are in Nomi.pen; the light twins mirror the dark ones.

## Phone (Flow / Mobile — You & Settings, EtSi2 / UBr1i), below 768
| # | Step | What it is |
|---|---|---|
| 1 | You | The hub (#347): person, plan card, groups, Log out. |
| 2 | General | Back to You; groups Account & sign-in (Email, Password, Google), Language & region (Language, Time zone), Appearance (Theme), Chat (New chats start with; Enter sends switch), Notifications (Reminders switch, Quiet hours), each row with its value. |
| 3 | Language sheet | A row's choices as a sheet, the chosen one ticked; picking sets it and closes. Theme, time zone, who chats start with and quiet hours work the same. |
| 4 | Change email | Pushed screen, tab bar hidden: the new address, what happens next, Send code above the keyboard. |
| 5 | Enter the code | Six boxes (paste works), Resend counting down, Change email once all six are in. |
| 6 | Delete account | Sessions then Delete account alone and last; a sheet: what goes, a workspace to hand over, the password or "Email me a code instead", Delete account (danger), Keep my account. |
| 7 | Profile | Avatar, name, @username and joined; Change photo; Details (Display name, Username) each a pushed screen with its own Save; Activity tiles and 13 weeks of token activity; the public switches; Showcase. No unsaved-changes bar. |
| 8 | Plan & usage | Plan card (allowance meter, resets, Compare plans); this month as four tiles; day by day; by model; Export usage CSV as a row. |
| 9 | Compare plans | Sheet: each plan's allowance and price, yours marked, one action each (Upgrade, Join the waitlist). |
| 10 | Crew | Core and Specialists, each member with what they do and their state. |
| 11 | A crew member | Who, what they're doing now, Model (Nomi, Koda) as a picker row, when they step in. |
| 12 | Models & providers | Defaults as picker rows; providers as rows with how many models are in the picker; Connect a provider. |
| 13 | A provider | Whose key, models as switches (the chats' default marked), Disconnect last. |
| 14 | Connect a provider | Sheet with search and the providers. |
| 15 | The key | Pushed screen: the key field, where to find one, Check and connect above the keyboard. |
| 16 | Memory & context | About you (size against the budget, Edit), the five kinds as rows with counts; learning switches and budgets; Export. |
| 17 | A kind of memory | Each memory with who learned it and its strength; ⋯ opens Correct, Confirm, Forget (Undo). |
| 18 | Workspaces | Each with your role and people, the one projects come from marked; Create; New projects go to. |
| 19 | A workspace | Invite first; people with roles (a role you may change opens a sheet; Remove asks); open invites with Copy link; Leave last. |
| 20 | Invite people | Sheet: email, Member or Admin with what each can do, Send invite. |
| 21 | Connections | Google Workspace, GitHub, Hevy, each with its crew member and state. |
| 22 | Manage a connection | Sheet: the account, what the crew member may use (switches), Disconnect. |
| 23 | States | A saved change says so in a toast at the top; a page that couldn't load says what happened and offers Try again. |

## Desktop (Flow / You — Settings (Desktop), ZhFyw / d95Bu)
| # | Step | Dialog |
|---|---|---|
| 1 | Change email | New email, Cancel, Send code. |
| 2 | Enter the code | Six boxes, Resend in 0:42, Back, Change email. |
| 3 | Change password | Current, new (8+), again; "Forgot it? Reset it by email"; Cancel, Change password. |
| 4 | Delete account | What goes, hand-over warning, password, "Email me a code instead"; Keep my account, Delete account. |
| 5 | Manage a connection | The account, access switches, Disconnect, Done. |

## Decisions (made from the UX rules)
- D1 Sub-pages keep the tab bar with a back bar to You (no drawer: You is the hub, #347).
- D2 Typing gets a pushed screen with the action above the keyboard; picking gets a sheet.
- D3 Changes save as they're made (switches, sheet picks); a toast at the top confirms. Profile's
  fields save from their own screens, so there's no unsaved-changes bar on a phone.
- D4 Nothing the backend can't do is drawn: the plans sheet says plans are changed by Nomi's team
  for now; the Google access switches are what Tara may use; delete asks for the password or a code.
- D5 Copy is the app's own messages where they exist; the pen's sample data (Kopi Kita, Siti) is
  replaced by the person's real data.

## Built (phone), and where the build differs from the pen
Every settings page has its phone screen, wired to the backend's own loads and actions, each
covered by `frontend-new/e2e/backend/phone-you.e2e.ts` against the real backend.
- D6 Profile shows the last 13 weeks, not a year: a year's cells at 390 are too small to read
  or tap. The busiest day under them is counted over the same 13 weeks.
- D7 Plans that can't be bought yet say Coming soon in the price's place (the desktop's "10×"
  room note stays on the desktop). Other months come off the back bar's ⋯ as a sheet of the
  last six; Export CSV is a row that saves the month shown.
- D8 Models' key screen opens "Get a key" in a new tab, so a half-typed key isn't lost.
  Connect a provider is a sheet with search; the key screen is `?connect=<id>` (Back is Back).
- D9 Memory's budgets are picked from a sheet of sizes (the shared text field has no number
  keyboard, and its props are frozen); Forget hides the memory at once and sends it after
  6 seconds unless Undo is tapped.
- D10 Google's access switches pick what to ask for; Google must confirm a change, so a changed
  selection shows Update access on Google (the backend can't change scopes without consent).
  GitHub's and Hevy's Manage show the account, what it may do, and Disconnect.
- D11 A settings page that couldn't load keeps its back bar and title, says why with the
  status, and offers Try again (step 23). Toasts sit at the top on every phone screen.
- D12 Setting rows wrap a long label to two lines and cap a value at 40% of the row, rather
  than cutting the label ("New chats start with").
