# Contract of nas-cleanup-manager with the NAS launcher

The contract every application shares with the launcher (package, manifest, permissions, frame
protocol, launcher socket and providers) is `docs/development/external_apps/APPS.md` in the
nas-launcher repository, and the bridge is the one of `@valeriogc/nas-base-app`. This file keeps
only what is specific to nas-cleanup-manager, which builds and releases on its own.

## Identity

- id `nas-cleanup-manager`, category `system`
- permissions `cleanup` true

## Specific to this application

`permissions.cleanup: true` (nas-cleanup-manager) lets an application ask the launcher, on its socket, for the cleanups of a fixed list (`POST /api/apps/self/cleanup` with `{ action: 'overview' | 'preview' | 'clean', item?, options: { temporaryDays, journalMb } }`) and read and empty the caches the applications declare (`GET /api/caches`, `DELETE /api/caches/{app}/{id}`). It sends only identifiers and two numbers: the paths and the commands are the launcher's, run as root by `nas-os-cleanup.service`.

The commands that check the repository are in [DEVELOPMENT.md](DEVELOPMENT.md).
