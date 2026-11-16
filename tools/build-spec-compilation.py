#!/usr/bin/env python3
"""从 docs/spec/ 生成单文件协议规范合集。

单一事实来源是 `docs/spec/` 下的分章节文件；本脚本仅做机械拼接，
**不添加任何规范内容**。

用法:
    python3 tools/build-spec-compilation.py           # 生成（覆盖 doc/ 下的合集）
    python3 tools/build-spec-compilation.py --check   # 校验合集是否为最新（CI 用）

退出码:
    0  一致 / 生成成功
    1  合集已过期（--check 模式），或生成失败
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC_DIR = ROOT / "docs" / "spec"
OUTPUT = ROOT / "doc" / "JIAYI-FSP-协议规范合集.md"

HEADER = """# jiayi-fsp 协议规范合集

> **本文件由 `tools/build-spec-compilation.py` 从 `docs/spec/` 自动生成，请勿手动编辑。**
>
> 单一事实来源是 `docs/spec/` 下的分章节文件。本文件仅供整体阅读与归档分发；
> 若存在差异，一律以 `docs/spec/` 的分章节原文件为准。
>
> 协议版本：v1.1.0

---

"""


def chapter_files() -> list[Path]:
    """按编号排序返回规范章节文件。"""
    files = [p for p in SPEC_DIR.glob("*.md") if p.is_file()]
    # 文件名形如 00-overview.md / 99-glossary.md
    return sorted(files, key=lambda p: (p.name[:2].isdigit() is False, p.name[:2], p.name))


def shift_headings(text: str) -> str:
    """把章节内的标题整体下沉一级（# -> ##, ## -> ### ...）。

    分章节文件以 `#` 作为章标题；合集中章标题为 `##`，其下小节顺次下沉。
    """
    out = []
    for line in text.split("\n"):
        m = re.match(r"^(#{1,5}) (.*)$", line)
        if m:
            out.append("#" + m.group(1) + " " + m.group(2))
        else:
            out.append(line)
    return "\n".join(out)


def build() -> str:
    parts = [HEADER]
    for path in chapter_files():
        body = path.read_text(encoding="utf-8").replace("\r\n", "\n").strip()
        parts.append(shift_headings(body))
        parts.append("\n---\n")
    return "\n".join(parts).rstrip() + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="生成 JIAYI-FSP 单文件协议规范合集")
    parser.add_argument("--check", action="store_true", help="只校验，不写入；合集过期时退出码为 1")
    args = parser.parse_args()

    if not SPEC_DIR.is_dir():
        print(f"错误：找不到规范目录 {SPEC_DIR}", file=sys.stderr)
        return 1

    chapters = chapter_files()
    if not chapters:
        print(f"错误：{SPEC_DIR} 下没有规范文件", file=sys.stderr)
        return 1

    content = build()

    if args.check:
        if not OUTPUT.exists():
            print(f"❌ 合集文件不存在：{OUTPUT}", file=sys.stderr)
            return 1
        current = OUTPUT.read_text(encoding="utf-8").replace("\r\n", "\n")
        if current != content:
            print("❌ 合集与 docs/spec/ 不一致，请运行：", file=sys.stderr)
            print("     python3 tools/build-spec-compilation.py", file=sys.stderr)
            return 1
        print(f"✅ 合集与 docs/spec/ 一致（{len(chapters)} 章）")
        return 0

    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(content, encoding="utf-8")
    print(f"✅ 已生成 {OUTPUT.relative_to(ROOT)}（{len(chapters)} 章，{len(content.splitlines())} 行）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
