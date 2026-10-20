//! Start the server for unit tests.

import { spawn } from 'node:child_process';

let proc: ReturnType<typeof spawn> | undefined

export async function setup() {
  proc = spawn('cargo', [ 'run', '--bin', 'test_server'], { cwd: '..', stdio: 'inherit' })

  const started = Date.now();
  while (!await isUp()) {
    if ((Date.now() - started) > 20 * 1000) throw new Error('Timed out waiting for server')
    await new Promise((r) => setTimeout(r, 500))
  }
}

const isUp = () => fetch('http://127.0.0.1:3000/health').then((r) => r.ok, () => false)

export function teardown() {
  proc?.kill()
}
