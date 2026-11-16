#!/usr/bin/env node
/**
 * 把规范源文件与图复制进站点构建产物。
 *
 * 为什么需要：规范源文件位于仓库的 `docs/spec/` 与 `docs/diagrams/`，
 * 而 Pages 的部署目录只包含 `site/dist/`。若不复制，
 * 部署后 `https://<user>.github.io/jiayi-fsp/spec/00-overview.md` 会 404。
 *
 * 用 node 而非 `cp` —— 保证 Windows / macOS / Linux 行为一致。
 */

import { cp, mkdir, rm } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const siteRoot = resolve(here, '..');
const repoRoot = resolve(siteRoot, '..');
const dist = join(siteRoot, 'dist');

/** 需要随站点一起发布的目录：源目录 → 产物内的目标目录名。 */
const PAYLOADS = [
  { from: join(repoRoot, 'docs', 'spec'), to: 'spec' },
  { from: join(repoRoot, 'docs', 'diagrams'), to: 'diagrams' },
];

if (!existsSync(dist)) {
  console.error(`找不到构建产物目录 ${dist}，请先运行构建。`);
  process.exit(1);
}

for (const { from, to } of PAYLOADS) {
  if (!existsSync(from)) {
    console.warn(`跳过：源目录不存在 ${from}`);
    continue;
  }
  const target = join(dist, to);
  await rm(target, { recursive: true, force: true });
  await mkdir(target, { recursive: true });
  await cp(from, target, { recursive: true });
  console.log(`已复制 ${from} → ${target}`);
}

console.log('规范源文件复制完成。');
