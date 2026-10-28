# nas-cleanup-manager

External application for the NAS launcher, "Cleanup": it frees space on the system disk by removing
temporary files, logs, Debian packages no longer useful, old kernels, leftovers of the NAS updates
and app caches, by hand or with scheduled cleanups. It never touches the data disks, `/home`, the
data and databases of the apps, the quarantine, or the files of the backup rules.

## Sections

- **Overview**: the system disk in a bar split by category (system, apps, caches, logs, downloaded
  packages, temporary, free), the data disks read only, the last cleanup and a button leading to
  Cleanup.
- **Cleanup**: a tree of switches on the model of BleachBit. The switch of a group takes or leaves
  all its items and shows half way when only some are chosen; groups open and close. On the
  right, for the highlighted item, what it does, which component it touches and how much it
  frees. **Preview** computes the space and the list of files or packages without deleting
  anything; **Clean** asks for confirmation with the summary and shows the progress, which also
  appears among the operations of the launcher. The choice is saved for the next time.
- **Schedule**: cleanups with a name, frequency (daily, weekly, monthly, yearly), time, on or off,
  each with its own tree of items. A time missed because the NAS was off is skipped.
- **History**: the last 100 cleanups, by hand or scheduled, with the space freed item by item.
- **Settings**: age of the temporary files, maximum size of the journal, notification at the end of
  a cleanup (never an email).

Delicate items (orphan packages, leftover configurations, old kernels) carry a warning sign, are
off by default and say why when they are turned on. Of the copies of the NAS updates the last one
always stays, which allows going back to the previous version; of the kernels, the one in use, the
previous one and the newer ones stay. The apt package lists (`/var/lib/apt/lists`) stay out:
without them, installing an app that asks for Debian packages would fail until the next
`apt update`.

While a scheduled cleanup is on, the app stays running with its window closed and starts with the
NAS (`run.background`), at low priority: it is stopped by turning the scheduled cleanups off or
with `sudo systemctl stop nas-app@nas-cleanup-manager`.

## Local start

Requires Node.js 24, pnpm and Rust/Cargo.

```bash
pnpm install
pnpm run build
pnpm run backend:dev
```

In a second terminal:

```bash
pnpm run dev
```

Vite serves the UI on `http://127.0.0.1:1420/` and forwards `api/...` to the backend over a Unix
socket. Without the socket of the launcher (`NAS_LAUNCHER_SOCKET`) the backend measures and cleans
nothing and says so: the E2E tests put a fake launcher next to it
(`src/tests/e2e/fixtures/launcher.mjs`).

For structure, tests and checks see [DEVELOPMENT.md](DEVELOPMENT.md). The contract with the
launcher is in [APPS.md](APPS.md); packaging and release are in [NAS.md](NAS.md).
