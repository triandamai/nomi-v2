# When the project runs in a sandbox

This project's commands and dev server run in a sandbox on Nomi's servers (a small Linux machine
that holds its files), not in the person's browser. Read this with the preview guide: this says
what changes, and the preview guide still says what the project may use.

## What changes

- **The project doesn't need to be open.** `run_command` doesn't need the project open in the
  person's browser: it wakes the sandbox and waits for it. A cold start creates or resumes the
  sandbox, copies the files in and starts the dev server, so the first command can take a minute
  or more.
- **When it's still starting, or couldn't start.** If `run_command` says the sandbox is still
  starting, or couldn't start (it gives the reason), don't call it again in a loop. Finish the
  work and complete the task, as the tool says: Nomi installs and checks the project once the
  sandbox is up, and any errors come back to you as a new task. Don't tell the person to open the
  project to make commands run.
- **Files and the dev server.** Every file you write is copied into the sandbox. Nomi starts the
  dev server itself (`npm run dev`, on port 5173), and when `package.json` changes it runs
  `npm install` and restarts the dev server on its own. Still don't start the dev server
  yourself.
- **When the install or dev server fails.** It comes back to you as a failed check with the
  reason, the same as in the browser. The sandbox keeps running, so your commands still work and
  you can look into it.
- **Idle sandboxes stop.** A sandbox stops after a while with nothing to do. Its files are kept;
  anything running in it besides the dev server isn't. The next command wakes it again.

## What doesn't change

The project can go back to the browser: an admin can turn sandboxes off, and the person can
choose to run it in their browser instead. So everything the preview guide says about what the
project may use still holds here:
- no native modules;
- keep the Rolldown lines in `package.json`;
- `"check": "echo ok"`;
- don't run `npm run build` during development.

The person still has no shell. Never ask them to type a command.
