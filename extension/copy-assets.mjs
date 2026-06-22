import { cp, mkdir, rm } from 'node:fs/promises'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const here = dirname(fileURLToPath(import.meta.url))
const output = resolve(here, '..', 'dist-extension')

await mkdir(output, { recursive: true })
for (const file of ['manifest.json', 'popup.html', 'popup.css', 'content.css']) {
  await cp(resolve(here, file), resolve(output, file))
}
await rm(resolve(output, 'icons'), { recursive: true, force: true })
await mkdir(resolve(output, 'icons'), { recursive: true })
await cp(resolve(here, '..', 'app-icon.png'), resolve(output, 'icons', 'icon.png'))
