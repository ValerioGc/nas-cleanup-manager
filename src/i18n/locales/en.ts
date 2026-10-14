import type { it } from './it';

export const en: typeof it = {
  sectionsLabel: 'Cleanup sections',
  sections: {
    overview: 'Overview',
    cleanup: 'Cleanup',
    schedule: 'Schedule',
    history: 'History',
  },
  settings: 'Settings',
  close: 'Close',
  cancel: 'Cancel',
  save: 'Save',
  increase: 'Increase',
  decrease: 'Decrease',
  retry: 'Retry',
  loading: 'Loading',
  loadFailed: 'The data could not be read',
  overview: {
    systemDisk: 'System disk',
    space: '{free} free of {total}',
    dataDisks: 'Data disks',
    dataDisksNote:
      'Internal disks of the NAS, read only. The cleanup never changes the data disks and the external disks.',
    noDisks: 'No data disk connected.',
    last: 'Last cleanup',
    lastText: '{date}, {size} freed',
    never: 'No cleanup done yet.',
    toCleanup: 'Go to the cleanup',
    categories: {
      system: 'System',
      apps: 'Apps',
      caches: 'Caches',
      logs: 'Logs',
      packages: 'Downloaded packages',
      temporary: 'Temporary',
      free: 'Free',
    },
  },
  groups: {
    apt: 'Debian packages (APT)',
    kernel: 'Kernel',
    logs: 'Logs',
    temporary: 'Temporary files',
    nas: 'NAS',
    caches: 'App caches',
    trash: 'Trashes',
  },
  items: {
    'apt-archives': {
      name: 'Downloaded packages',
      description:
        'Copies of the Debian packages downloaded during installations and upgrades. The system does not need them and they are downloaded again when needed.',
      touches: '/var/cache/apt/archives',
    },
    'apt-obsolete': {
      name: 'Outdated downloaded packages',
      description:
        'Downloaded packages that match no version available in the Debian sources any more and cannot be reused.',
      touches: '/var/cache/apt/archives',
    },
    'apt-orphans': {
      name: 'Orphan packages',
      description:
        'Packages installed automatically as dependencies of packages already removed. No installed package requires them any more.',
      touches: 'Installed Debian packages',
    },
    'apt-configs': {
      name: 'Leftover configurations',
      description:
        'Configuration files left by removed packages, with no effect on the running system.',
      touches: '/etc',
    },
    'old-kernels': {
      name: 'Old kernels',
      description:
        'Linux kernel versions older than the running one and the previous one. The running kernel, the previous one and the newer ones stay installed.',
      touches: '/boot and /usr/lib/modules',
    },
    'rotated-logs': {
      name: 'Rotated logs',
      description:
        'Archived copies of the system logs, made at each rotation and often compressed. The logs in use stay unchanged.',
      touches: '/var/log',
    },
    journal: {
      name: 'System journal',
      description:
        'Shrinks the system journal to the configured maximum size ({size}), deleting the oldest entries.',
      touches: '/var/log/journal',
    },
    temporary: {
      name: 'Temporary files',
      description:
        'Files not modified for more than {days} days in the temporary folders. The files of running programs stay unchanged.',
      touches: '/tmp and /var/tmp',
    },
    'nas-backups': {
      name: 'Update backups',
      description:
        'Backup copies made before each update of the NAS. The most recent copy always stays, to allow a return to the previous version.',
      touches: '/var/lib/nas-os/backups',
    },
    'nas-leftovers': {
      name: 'Update leftovers',
      description:
        'Downloaded update packages and app versions left by an update that did not finish, older than one day.',
      touches: '/var/lib/nas-os',
    },
  },
  risky: 'Delicate item',
  riskyNotes: {
    'apt-orphans':
      'No longer required by the system, but programs installed by hand may use them. The preview shows the full list before the removal.',
    'apt-configs':
      'Reinstalling one of these packages starts again from its default configuration.',
    'old-kernels':
      'Without a spare kernel, a faulty update can stop the boot. The previous kernel stays installed in any case.',
  },
  trashDescription: 'Items in the trash of {app}. Once emptied they cannot be recovered.',
  trashRiskyNote:
    'The items in the trash can still be restored. Emptying it deletes them permanently.',
  cacheDescription: 'Temporary data of {app}, rebuilt automatically by the app when needed.',
  cleanup: {
    intro:
      'Selection of the items to clean. The preview measures the space without deleting anything.',
    tree: 'Items to clean',
    expand: 'Open {name}',
    collapse: 'Close {name}',
    preview: 'Preview',
    clean: 'Clean',
    taken: '{taken} of {total}',
    previewing: 'Measuring the space',
    chooseOne: 'At least one item is required.',
    touches: 'Component',
    frees: 'Space it frees',
    notMeasured: 'Value available after the preview.',
    entries: 'Entries',
    more: 'and {count} more',
    failed: 'Not measured ({reason})',
    total: 'Total to free {size}',
    confirmTitle: 'Start the cleanup?',
    confirmMessage: '{count} items will be cleaned. The operation cannot be undone.',
    confirmRisky: 'Delicate items included',
    running: 'Cleaning, {done} of {total}',
    done: 'Cleanup done, {size} freed.',
    doneWithErrors: 'Cleanup done, {size} freed. {count} items not cleaned.',
  },
  schedule: {
    intro:
      'Scheduled cleanups run at the time of the NAS, even with the app closed. A time missed while the NAS was off is skipped.',
    add: 'Add cleanup',
    empty: 'No scheduled cleanup',
    emptyNote:
      'Scheduled cleanups run automatically at the chosen times. The Add cleanup button creates a new one.',
    name: 'Name',
    enabled: 'On',
    when: 'When',
    next: 'Next',
    off: 'Off',
    items: '{count} items',
    newTitle: 'New schedule',
    editTitle: 'Edit schedule',
    frequency: 'Frequency',
    time: 'Time',
    weekday: 'Day of the week',
    day: 'Day of the month',
    month: 'Month',
    what: 'What to clean',
    riskyIncluded: 'This cleanup includes delicate items.',
    run: 'Run {name} now',
    edit: 'Edit {name}',
    delete: 'Delete {name}',
    deleteTitle: 'Delete the scheduled cleanup?',
    deleteMessage: 'The scheduled cleanup {name} will be deleted.',
    nameRequired: 'A name for the cleanup is required.',
    itemsRequired: 'At least one item is required.',
  },
  frequencies: {
    daily: 'Every day',
    weekly: 'Every week',
    monthly: 'Every month',
    yearly: 'Every year',
  },
  scheduleSummary: {
    daily: 'Every day at {time}',
    weekly: 'Every {weekday} at {time}',
    monthly: 'On day {day} of every month at {time}',
    yearly: 'Every year on {month} {day} at {time}',
  },
  history: {
    empty: 'No cleanup done',
    emptyNote:
      'Completed cleanups, manual and scheduled, show here with the space freed by each item.',
    manual: 'By hand',
    freed: '{size} freed',
    failed: 'Not cleaned ({reason})',
  },
  settingsModal: {
    temporaryDays: 'Age of temporary files (days)',
    temporaryDaysHint: 'Only temporary files older than this value are removed.',
    journalMb: 'Largest journal size (MiB)',
    journalHint: 'Size the system journal is shrunk to.',
    notify: 'Notify at the end of a cleanup',
    notifyHint: 'Notification of the NAS, without email.',
  },
};
