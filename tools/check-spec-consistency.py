#!/usr/bin/env python3
"""协议规范内部一致性检查。

把 v1.0.0 修订过程中发现并修正过的缺陷固化为自动检查，防止回归。

检查项:
    1. 端序：spec 08 必须声明小端，且不得残留"网络字节序"表述
    2. session_id：JSON 示例中必须为字符串，不得为数字
    3. file_hash：必须为字符串（Blake3 hex），不得为 u64
    4. Topic 层级：Server 订阅过滤器必须匹配 5 层
    5. proto 对齐：proto/ 的字段宽度须与 spec 08 一致
    6. 版本号：README / spec 00 / CHANGELOG 的协议版本须一致

退出码:
    0  全部通过
    1  存在不一致
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPEC = ROOT / "docs" / "spec"

errors: list[str] = []
notes: list[str] = []


def read(rel: str) -> str:
    p = ROOT / rel
    if not p.exists():
        errors.append(f"[缺失] 文件不存在: {rel}")
        return ""
    return p.read_text(encoding="utf-8").replace("\r\n", "\n")


def spec(name: str) -> str:
    p = SPEC / name
    if not p.exists():
        errors.append(f"[缺失] 规范文件不存在: docs/spec/{name}")
        return ""
    return p.read_text(encoding="utf-8").replace("\r\n", "\n")


def normative(text: str) -> str:
    """只保留规范性正文，过滤引用块（历史变更说明通常写在引用块里）。

    以 `>` 开头的行视为非规范性说明，不参与矛盾性检查。
    """
    return "\n".join(line for line in text.split("\n") if not line.lstrip().startswith(">"))


# ---------------------------------------------------------------- 1. 端序
def check_endianness() -> None:
    s08 = spec("08-packet-format.md")
    if not s08:
        return
    if "网络字节序" in normative(s08):
        errors.append("[08] 仍存在自相矛盾的“网络字节序”表述，应统一为小端序（Little-Endian）")
    if "小端" not in s08:
        errors.append("[08] 未声明小端序")
    s07 = spec("07-data-plane-udp.md")
    if "8 字节小端编码" in normative(s07):
        errors.append("[07] 探测包 payload 仍写作 8 字节；session_id 为 u32，应为 4 字节")


# ------------------------------------------------------------ 2. session_id
def check_session_id() -> None:
    s06 = spec("06-control-plane-mqtt.md")
    if not s06:
        return
    # JSON 示例中 session_id 必须是字符串
    for m in re.finditer(r'"session_id"\s*:\s*([^,\n}]+)', s06):
        val = m.group(1).strip()
        if not val.startswith('"') and val not in ("null",):
            errors.append(f'[06] session_id 必须以字符串传输，发现: "session_id": {val}')
    if "以**字符串**形式传输" not in s06 and "以字符串" not in s06:
        errors.append("[06] 未明确 session_id 的字符串传输约束")


# -------------------------------------------------------------- 3. file_hash
def check_file_hash() -> None:
    for name in sorted(p.name for p in SPEC.glob("*.md")):
        text = spec(name)
        for m in re.finditer(r'"file_hash"\s*:\s*"([^"]*)"', text):
            t = m.group(1)
            if "u64" in t or "u32" in t or "int" in t:
                errors.append(f'[{name[:2]}] file_hash 类型应为字符串（Blake3 hex），发现: "{t}"')


# ------------------------------------------------------------ 4. Topic 层级
def check_topic_levels() -> None:
    for name in ("06-control-plane-mqtt.md", "21-acl-and-auth.md", "22-topic-spec.md"):
        text = normative(spec(name))
        if not text:
            continue
        # 形如 /jiayi/fsp/client/+ 但后面没跟 / 的订阅写法是 4 层，匹配不到消息
        for m in re.finditer(r'(/jiayi/fsp/(?:client|server)/\+)(?![/#])', text):
            errors.append(
                f"[{name[:2]}] Topic 过滤器 {m.group(1)} 仅为 4 层，"
                f"匹配不到 5 层消息，应写为 {m.group(1)}/# 或 {m.group(1)}/+"
            )


# ------------------------------------------------------------ 5. proto 对齐
def check_proto_alignment() -> None:
    rs = read("proto/fsp_packet.rs")
    if not rs:
        return
    s08 = spec("08-packet-format.md")
    expected = {
        "session_id": "u32",
        "chunk_id": "u32",
        "payload_len": "u16",
        "flags": "u16",
    }
    for field, ty in expected.items():
        m = re.search(rf"pub {field}\s*:\s*(\w+)", rs)
        if not m:
            errors.append(f"[proto] fsp_packet.rs 缺少字段 {field}")
        elif m.group(1) != ty:
            errors.append(f"[proto] fsp_packet.rs 的 {field} 为 {m.group(1)}，spec 08 要求 {ty}")
    if "file_id_hash" in rs:
        errors.append("[proto] fsp_packet.rs 仍含 file_id_hash，spec 08 中不存在该字段")
    m = re.search(r"pub const HEADER_SIZE:\s*usize\s*=\s*([^;]+);", rs)
    if m:
        expr = re.sub(r"[^0-9+\-*/() ]", "", m.group(1))
        try:
            if eval(expr, {"__builtins__": {}}) != 12:  # noqa: S307 - 固定表达式
                errors.append(f"[proto] HEADER_SIZE 表达式求值为 {eval(expr)}, spec 08 要求 12")
        except Exception:
            errors.append(f"[proto] 无法解析 HEADER_SIZE 表达式: {m.group(1)!r}")
    else:
        errors.append("[proto] fsp_packet.rs 缺少 HEADER_SIZE 常量")
    if s08 and "crc32" not in s08:
        errors.append("[08] 未定义 crc32")


# -------------------------------------------------------------- 6. 版本号
def check_version() -> None:
    versions = {}
    readme = read("README.md")
    readme_zh = read("README.zh-CN.md")
    overview = spec("00-overview.md")
    changelog = read("CHANGELOG.md")

    m = re.search(r"Status:\s*v?(\d+\.\d+\.\d+)", readme)
    versions["README.md"] = m.group(1) if m else None
    m = re.search(r"状态：\*\*v?(\d+\.\d+\.\d+)", readme_zh)
    versions["README.zh-CN.md"] = m.group(1) if m else None
    m = re.search(r"协议版本：\*\*v?(\d+\.\d+\.\d+)", overview)
    versions["docs/spec/00"] = m.group(1) if m else None
    m = re.search(r"协议版本：v?(\d+\.\d+\.\d+)", changelog)
    versions["CHANGELOG.md"] = m.group(1) if m else None

    known = {v for v in versions.values() if v}
    if len(known) > 1:
        detail = "，".join(f"{k}={v or '未找到'}" for k, v in versions.items())
        errors.append(f"[版本号] 各处协议版本不一致: {detail}")
    elif known:
        notes.append(f"协议版本号一致: {known.pop()}")


def main() -> int:
    for fn in (
        check_endianness,
        check_session_id,
        check_file_hash,
        check_topic_levels,
        check_proto_alignment,
        check_version,
    ):
        fn()

    for n in notes:
        print(f"ℹ️  {n}")

    if errors:
        print(f"\n❌ 规范一致性检查失败（{len(errors)} 项）：\n", file=sys.stderr)
        for e in errors:
            print(f"   • {e}", file=sys.stderr)
        return 1

    print("✅ 规范一致性检查通过")
    return 0


if __name__ == "__main__":
    sys.exit(main())
