import { createServer } from 'node:http';
import { rmSync } from 'node:fs';

const socket = process.env.NAS_LAUNCHER_SOCKET;
if (!socket) {
  throw new Error('NAS_LAUNCHER_SOCKET is required');
}
const MB = 1024 * 1024;
const sizes = {
  'apt-archives': 120 * MB,
  'apt-obsolete': 10 * MB,
  'apt-orphans': 40 * MB,
  'apt-configs': 0,
  'old-kernels': 300 * MB,
  'rotated-logs': 25 * MB,
  journal: 200 * MB,
  temporary: 3 * MB,
  'nas-backups': 80 * MB,
  'nas-leftovers': 0,
};
let previews = 64 * MB;

function cleanup(request) {
  if (request.action === 'overview') {
    return {
      total: 64 * 1024 * MB,
      free: 20 * 1024 * MB,
      categories: [
        { id: 'packages', bytes: 130 * MB },
        { id: 'logs', bytes: 900 * MB },
        { id: 'temporary', bytes: 3 * MB },
        { id: 'apps', bytes: 700 * MB },
      ],
      disks: [{ name: 'Dati', total: 2048 * 1024 * MB, free: 1024 * 1024 * MB }],
    };
  }
  const bytes = sizes[request.item] ?? 0;
  const entries = request.item === 'old-kernels' ? ['linux-image-6.1.0-9-amd64'] : [];
  return { item: request.item, bytes, entries, more: 0 };
}

const server = createServer((request, response) => {
  let body = '';
  request.on('data', (chunk) => (body += chunk));
  request.on('end', () => {
    const route = `${request.method} ${request.url}`;
    let status = 204;
    let answer = null;
    if (route === 'POST /api/apps/self/cleanup') {
      status = 200;
      answer = cleanup(JSON.parse(body));
    } else if (route === 'GET /api/caches') {
      status = 200;
      answer = [
        {
          app: 'nas-file-manager',
          id: 'previews',
          label: 'Miniature di Explorer',
          bytes: previews,
          items: 12,
        },
        {
          app: 'nas-file-manager',
          id: 'trash',
          label: 'Cestino di Explorer',
          bytes: 5 * MB,
          items: 3,
          kind: 'trash',
        },
      ];
    } else if (route === 'DELETE /api/caches/nas-file-manager/previews') {
      previews = 0;
    }
    const text = answer === null ? '' : JSON.stringify(answer);
    response.writeHead(status, { 'Content-Type': 'application/json' });
    response.end(text);
  });
});

rmSync(socket, { force: true });
server.listen(socket);
for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => server.close(() => process.exit(0)));
}
