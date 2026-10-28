export const it = {
  sectionsLabel: 'Sezioni della pulizia',
  sections: {
    overview: 'Panoramica',
    cleanup: 'Pulizia',
    schedule: 'Pianificazione',
    history: 'Cronologia',
  },
  settings: 'Impostazioni',
  close: 'Chiudi',
  cancel: 'Annulla',
  save: 'Salva',
  increase: 'Aumenta',
  decrease: 'Diminuisci',
  retry: 'Riprova',
  loading: 'Caricamento in corso',
  loadFailed: 'Impossibile leggere i dati',
  overview: {
    systemDisk: 'Disco di sistema',
    space: '{free} liberi su {total}',
    dataDisks: 'Dischi dati',
    dataDisksNote:
      'Dischi interni del NAS, in sola lettura. La pulizia non modifica mai i dischi dati e i dischi esterni.',
    noDisks: 'Nessun disco dati collegato.',
    last: 'Ultima pulizia',
    lastText: '{date}, liberati {size}',
    never: 'Nessuna pulizia eseguita finora.',
    toCleanup: 'Vai alla pulizia',
    categories: {
      system: 'Sistema',
      apps: 'App',
      caches: 'Cache',
      logs: 'Log',
      packages: 'Pacchetti scaricati',
      temporary: 'Temporanei',
      free: 'Libero',
    },
  },
  groups: {
    apt: 'Pacchetti Debian (APT)',
    kernel: 'Kernel',
    logs: 'Log',
    temporary: 'File temporanei',
    nas: 'NAS',
    caches: 'Cache delle app',
    trash: 'Cestini',
  },
  items: {
    'apt-archives': {
      name: 'Pacchetti scaricati',
      description:
        'Copie dei pacchetti Debian scaricati durante installazioni e aggiornamenti. Non servono al funzionamento del sistema e vengono scaricate di nuovo quando necessario.',
      touches: '/var/cache/apt/archives',
    },
    'apt-obsolete': {
      name: 'Pacchetti scaricati obsoleti',
      description:
        'Pacchetti scaricati che non corrispondono più a nessuna versione disponibile nelle sorgenti di Debian e non possono essere riutilizzati.',
      touches: '/var/cache/apt/archives',
    },
    'apt-orphans': {
      name: 'Pacchetti orfani',
      description:
        'Pacchetti installati automaticamente come dipendenze di altri pacchetti già rimossi. Nessun pacchetto installato li richiede più.',
      touches: 'Pacchetti Debian installati',
    },
    'apt-configs': {
      name: 'Configurazioni rimaste',
      description:
        'File di configurazione lasciati dai pacchetti rimossi, senza effetto sul sistema in uso.',
      touches: '/etc',
    },
    'old-kernels': {
      name: 'Kernel vecchi',
      description:
        'Versioni del kernel Linux meno recenti di quella in uso e della precedente. Restano installati il kernel in uso, il precedente e quelli più recenti.',
      touches: '/boot e /usr/lib/modules',
    },
    'rotated-logs': {
      name: 'Log ruotati',
      description:
        'Copie archiviate dei log di sistema, create a ogni rotazione e spesso compresse. I log in uso restano invariati.',
      touches: '/var/log',
    },
    journal: {
      name: 'Journal di sistema',
      description:
        'Riduce il journal di sistema alla dimensione massima impostata ({size}), eliminando le voci più vecchie.',
      touches: '/var/log/journal',
    },
    temporary: {
      name: 'File temporanei',
      description:
        'File non modificati da più di {days} giorni nelle cartelle temporanee. I file dei programmi in esecuzione restano invariati.',
      touches: '/tmp e /var/tmp',
    },
    'nas-backups': {
      name: 'Copie degli aggiornamenti',
      description:
        'Copie di sicurezza create prima di ogni aggiornamento del NAS. La copia più recente resta sempre, per consentire il ritorno alla versione precedente.',
      touches: '/var/lib/nas-os/backups',
    },
    'nas-leftovers': {
      name: 'Residui degli aggiornamenti',
      description:
        'Pacchetti di aggiornamento scaricati e versioni di app lasciati da un aggiornamento non completato, più vecchi di un giorno.',
      touches: '/var/lib/nas-os',
    },
  },
  risky: 'Voce delicata',
  riskyTitle: 'Attenzione',
  riskyNotes: {
    'apt-orphans':
      'Non più richiesti dal sistema, ma possono servire a programmi installati manualmente. L’anteprima mostra l’elenco completo prima della rimozione.',
    'apt-configs':
      'Reinstallando uno di questi pacchetti si riparte dalla configurazione predefinita.',
    'old-kernels':
      'Senza un kernel di riserva, un aggiornamento difettoso può impedire l’avvio. Il kernel precedente resta comunque installato.',
  } as Record<string, string>,
  trashDescription: 'Elementi nel cestino di {app}. Dopo lo svuotamento non sono più recuperabili.',
  trashRiskyNote:
    'Gli elementi nel cestino sono ancora ripristinabili. Lo svuotamento li elimina definitivamente.',
  cacheDescription: 'Dati temporanei di {app}, ricreati automaticamente dall’app quando servono.',
  cleanup: {
    intro: 'Selezione delle voci da pulire. L’anteprima calcola lo spazio senza eliminare nulla.',
    tree: 'Voci da pulire',
    expand: 'Apri {name}',
    collapse: 'Chiudi {name}',
    preview: 'Anteprima',
    clean: 'Pulisci',
    taken: '{taken} di {total}',
    previewing: 'Calcolo dello spazio in corso',
    chooseOne: 'È necessaria almeno una voce.',
    touches: 'Componente',
    frees: 'Spazio liberabile',
    notMeasured: 'Valore disponibile dopo l’anteprima.',
    entries: 'Elementi',
    more: 'e altri {count}',
    failed: 'Non misurabile ({reason})',
    total: 'Totale liberabile {size}',
    confirmTitle: 'Avviare la pulizia?',
    confirmMessage: 'Verranno pulite {count} voci. L’operazione non è reversibile.',
    confirmRisky: 'Voci delicate incluse',
    running: 'Pulizia in corso, {done} di {total}',
    done: 'Pulizia completata, liberati {size}.',
    doneWithErrors: 'Pulizia completata, liberati {size}. {count} voci non pulite.',
  },
  schedule: {
    intro:
      'Le pulizie pianificate vengono eseguite all’orario del NAS anche con l’app chiusa. Un orario mancato con il NAS spento viene saltato.',
    add: 'Aggiungi pulizia',
    empty: 'Nessuna pulizia pianificata',
    emptyNote:
      'Le pulizie pianificate vengono eseguite in automatico agli orari scelti. Il pulsante Aggiungi pulizia ne crea una nuova.',
    name: 'Nome',
    enabled: 'Attiva',
    when: 'Quando',
    next: 'Prossima',
    off: 'Disattivata',
    items: '{count} voci',
    newTitle: 'Nuova pianificazione',
    editTitle: 'Modifica pianificazione',
    frequency: 'Frequenza',
    time: 'Ora',
    weekday: 'Giorno della settimana',
    day: 'Giorno del mese',
    month: 'Mese',
    what: 'Cosa pulire',
    riskyIncluded: 'Questa pulizia include voci delicate.',
    toggle: 'Pulizia {name} attiva',
    run: 'Avvia ora {name}',
    edit: 'Modifica {name}',
    delete: 'Elimina {name}',
    deleteTitle: 'Eliminare la pulizia pianificata?',
    deleteMessage: 'La pulizia pianificata {name} verrà eliminata.',
    nameRequired: 'È necessario un nome per la pulizia.',
    itemsRequired: 'È necessaria almeno una voce.',
  },
  frequencies: {
    daily: 'Ogni giorno',
    weekly: 'Ogni settimana',
    monthly: 'Ogni mese',
    yearly: 'Ogni anno',
  },
  scheduleSummary: {
    daily: 'Ogni giorno alle {time}',
    weekly: 'Ogni {weekday} alle {time}',
    monthly: 'Il giorno {day} di ogni mese alle {time}',
    yearly: 'Ogni anno il {day} {month} alle {time}',
  },
  history: {
    empty: 'Nessuna pulizia eseguita',
    emptyNote:
      'Le pulizie completate, manuali e pianificate, compaiono qui con lo spazio liberato da ogni voce.',
    manual: 'A mano',
    freed: 'Liberati {size}',
    failed: 'Non pulita ({reason})',
  },
  settingsModal: {
    temporaryDays: 'Età dei file temporanei (giorni)',
    temporaryDaysHint: 'Vengono rimossi solo i file temporanei più vecchi di questo valore.',
    journalMb: 'Dimensione massima del journal (MiB)',
    journalHint: 'Dimensione a cui viene ridotto il journal di sistema.',
    notify: 'Notifica a fine pulizia',
    notifyHint: 'Notifica del NAS, senza email.',
  },
};
