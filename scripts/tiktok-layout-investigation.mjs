import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const TARGET_URL = 'https://www.tiktok.com/@vantaplay_jp';
const OUT_DIR = process.cwd();
const ARTIFACT_DIR = join(OUT_DIR, 'artifacts', 'tiktok-layout-investigation');

mkdirSync(ARTIFACT_DIR, { recursive: true });

function runOsascript(args, env = {}) {
  return execFileSync('osascript', args, {
    encoding: 'utf8',
    env: { ...process.env, ...env },
    maxBuffer: 20 * 1024 * 1024,
  }).trim();
}

function runChromeJs(js) {
  const jxa = [
    'ObjC.import("Foundation");',
    'const env = $.NSProcessInfo.processInfo.environment;',
    'const code = ObjC.unwrap(env.objectForKey("CODE"));',
    'const chrome = Application("Google Chrome");',
    'const win = chrome.windows[0];',
    'const out = win.activeTab.execute({ javascript: code });',
    'if (out !== undefined) console.log(out);',
  ].join(' ');

  return runOsascript(['-l', 'JavaScript', '-e', jxa], { CODE: js });
}

function chromeTitleAndUrl() {
  const script = [
    'tell application "Google Chrome"',
    '  return {title of active tab of front window, URL of active tab of front window}',
    'end tell',
  ].join('\n');
  return runOsascript(['-e', script]);
}

function ensureInvestigationTab(url) {
  const script = [
    'tell application "Google Chrome"',
    '  activate',
    '  tell front window',
    '    make new tab with properties {URL:"' + url + '"}',
    '    set active tab index to (count of tabs)',
    '  end tell',
    'end tell',
  ].join('\n');
  runOsascript(['-e', script]);
}

function main() {
  const summaryPath = join(ARTIFACT_DIR, 'bootstrap-status.txt');
  writeFileSync(
    summaryPath,
    [
      'TikTok layout investigation bootstrap',
      `Target: ${TARGET_URL}`,
      `Chrome tab before bootstrap: ${chromeTitleAndUrl()}`,
    ].join('\n'),
  );

  ensureInvestigationTab(TARGET_URL);

  try {
    const result = runChromeJs(
      "JSON.stringify({ title: document.title, href: location.href, readyState: document.readyState })",
    );
    writeFileSync(join(ARTIFACT_DIR, 'chrome-js-smoke.json'), result + '\n');
    console.log('Chrome JavaScript bridge is enabled.');
    console.log(`Smoke result saved to ${join(ARTIFACT_DIR, 'chrome-js-smoke.json')}`);
  } catch (error) {
    const message = String(error?.stderr || error?.message || error);
    writeFileSync(join(ARTIFACT_DIR, 'chrome-js-error.txt'), message + '\n');
    console.error(message);
    process.exitCode = 1;
  }
}

main();
