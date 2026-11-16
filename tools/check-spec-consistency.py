#!/usr/bin/env python3
"""协议规范内部一致性检查。

把历次修订过程中发现并修正过的缺陷固化为自动检查，防止回归。

检查项:
    1. 端序：spec 08 必须声明小端，且不得残留"网络字节序"表述
    2. session_id：JSON 示例中必须为字符串，不得为数字
    3. file_hash：必须为字符串（Blake3 hex），不得为 u64
    4. Topic 层级：Server 订阅过滤器必须匹配 5 层
    5. proto 对齐：proto/ 的字段宽度须与 spec 08 一致
    6. 版本号：README / spec 00 / CHANGELOG 的协议版本须一致
    7. 消息清单：06 消息表 / 22 Topic 列表 / 99 术语表 三方交叉（名称 + 方向）
    8. 消息载荷：06 每个消息都必须有 JSON 载荷，且 type 字段与消息名一致
    9. 状态机：10 章每条转移的触发消息必须存在；会话类消息必须在状态机中出现
   10. Topic 规范形式：Server 订阅过滤器只能写成 5 层的 `+/#` 形式
   11. 非法行为 ↔ Topic 清单互斥：禁止 Client 发布的消息不得出现在 C→S 清单中
   12. Markdown 代码围栏必须成对，否则渲染被破坏
   13. 关键载荷字段守卫：accept 必须带数据面端口与 chunk_size；missing_chunks 必须带 mode
   14. resume 语义守卫：next_chunk_id 必须是「第一个尚未收到的块」
   15. window 默认值在各章必须一致
   16. 引用块规范性守卫：规范性要求必须写在正文，引用块仅用于历史说明

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


# ------------------------------------------------- 7/8/9. 消息清单解析工具
CS, SC = "C→S", "S→C"


def _dirs(cell: str) -> set[str]:
    """把方向单元格归一化为方向集合。"""
    c = cell.strip()
    if "双向" in c:
        return {CS, SC}
    ci, si = c.find("Client"), c.find("Server")
    if ci >= 0 and si >= 0:
        # `Client→Server` / `Server→Client` 形态：按出现顺序判定
        return {CS} if ci < si else {SC}
    if "C→S" in c:
        return {CS}
    if "S→C" in c:
        return {SC}
    return set()


def _section(text: str, start: str, end: str | None = None) -> str:
    """截取 `start` 与 `end`（下一个二级标题）之间的正文。"""
    i = text.find(start)
    if i < 0:
        return ""
    rest = text[i + len(start):]
    if end is None:
        m = re.search(r"\n## ", rest)
    else:
        m = re.search(rf"\n{re.escape(end)}", rest)
    return rest[: m.start()] if m else rest


def parse_06_messages() -> dict[str, set[str]]:
    """解析 06 章「消息类型」表格：消息名 -> 方向集合。"""
    text = spec("06-control-plane-mqtt.md")
    body = _section(text, "## 消息类型", "## 载荷格式")
    out: dict[str, set[str]] = {}
    for m in re.finditer(r"^\|\s*([a-z][a-z0-9_]*)\s*\|\s*([^|]+?)\s*\|", body, re.M):
        out.setdefault(m.group(1), set()).update(_dirs(m.group(2)))
    return out


def parse_22_topics() -> dict[str, set[str]]:
    """解析 22 章 Topic 列表：action -> 方向集合。"""
    text = spec("22-topic-spec.md")
    out: dict[str, set[str]] = {}
    c2s = _section(text, "### Client → Server", "### Server → Client")
    s2c = _section(text, "### Server → Client", "## 订阅与发布约束")
    for body, dirs in ((c2s, {CS}), (s2c, {SC})):
        for m in re.finditer(r"^- `/jiayi/fsp/(?:client|server)/\{client_id\}/([a-z0-9_]+)`", body, re.M):
            out.setdefault(m.group(1), set()).update(dirs)
    return out


def parse_99_messages() -> dict[str, set[str]]:
    """解析 99 章控制面消息表：消息名 -> 方向集合。"""
    text = spec("99-glossary.md")
    body = _section(text, "## 控制面消息")
    out: dict[str, set[str]] = {}
    for m in re.finditer(r"^\|\s*([^|]+?)\s*\|\s*([^|]+?)\s*\|", body, re.M):
        names = [n.strip() for n in m.group(1).split("/")]
        dirs_raw = [d.strip() for d in m.group(2).split("/")]
        names = [n for n in names if re.fullmatch(r"[a-z][a-z0-9_]*", n)]
        if not names:
            continue
        if len(dirs_raw) == len(names):
            pairs = zip(names, dirs_raw)
        else:
            merged = set()
            for d in dirs_raw:
                merged |= _dirs(d)
            pairs = [(n, "") for n in names]
            for n in names:
                out.setdefault(n, set()).update(merged)
            continue
        for n, d in pairs:
            out.setdefault(n, set()).update(_dirs(d))
    return out


# ---------------------------------------------------------- 7. 消息清单交叉
def check_message_inventory() -> None:
    """06 消息表 / 22 Topic 列表 / 99 术语表 三方交叉：名称与方向必须一致。"""
    m06, m22, m99 = parse_06_messages(), parse_22_topics(), parse_99_messages()
    if not (m06 and m22 and m99):
        return

    for label, a, b in (("22", m06, m22), ("99", m06, m99)):
        only_a = sorted(set(a) - set(b))
        only_b = sorted(set(b) - set(a))
        for n in only_a:
            errors.append(f"[消息清单] 06 章定义的 {n} 在 {label} 章中缺失")
        for n in only_b:
            errors.append(f"[消息清单] {label} 章出现的 {n} 在 06 章消息表中缺失")

    for name in sorted(set(m06) & set(m22) & set(m99)):
        triple = (m06[name], m22[name], m99[name])
        if not (triple[0] == triple[1] == triple[2]):
            detail = "，".join(
                f"{tag}={_fmt_dirs(d)}"
                for tag, d in (("06", m06[name]), ("22", m22[name]), ("99", m99[name]))
            )
            errors.append(f"[消息清单] {name} 的方向在 06/22/99 三处不一致: {detail}")

    notes.append(f"消息清单三方交叉: {len(m06)} 个消息（06/22/99）")


def _fmt_dirs(d: set[str]) -> str:
    if d == {CS, SC}:
        return "双向"
    if d == {CS}:
        return "C→S"
    if d == {SC}:
        return "S→C"
    return "/".join(sorted(d)) or "未声明"


# ------------------------------------------------------- 8. 消息载荷完整性
def check_payloads() -> None:
    """06 章每个消息都必须有 JSON 载荷，且 type 字段与消息名一致。"""
    text = spec("06-control-plane-mqtt.md")
    if not text:
        return
    declared = set(parse_06_messages())
    payload_types: set[str] = set()
    for m in re.finditer(r'"type"\s*:\s*"([a-z][a-z0-9_]*)"', text):
        payload_types.add(m.group(1))

    for n in sorted(declared - payload_types):
        errors.append(f"[载荷] 06 章消息 {n} 没有对应的 JSON 载荷示例")
    for n in sorted(payload_types - declared):
        errors.append(f"[载荷] JSON 载荷的 type={n} 未出现在 06 章消息表中")


# ------------------------------------------------------- 9. 状态机 ↔ 消息
NON_TRANSITION_MESSAGES = {
    "login", "login_ack", "status", "progress_update",
    "missing_chunks", "udp_ready", "asset_recall", "asset_recall_complete",
}


def check_state_machine() -> None:
    """10 章转移表的触发消息必须真实存在；会话类消息必须在状态机中出现。"""
    s10 = spec("10-state-machine.md")
    if not s10:
        return
    known = set(parse_06_messages())

    body = _section(s10, "## 合法状态转移", "## ")
    triggers: set[str] = set()
    for line in body.split("\n"):
        if not line.strip().startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 3 or not re.fullmatch(r"[A-Z_]+", cells[0]) or not re.fullmatch(r"[A-Z_]+", cells[1]):
            continue
        # 最后一列为触发条件（前置列可能是「视角」等说明列）
        for tok in re.split(r"[/、,，]+", cells[-1]):
            tok = tok.strip("（）()「」\"'` ")
            if not tok:
                continue
            # 形如 `upload_start` 的标识符视为消息名；含中文的视为本地事件描述
            if re.fullmatch(r"[a-z][a-z0-9_]*", tok):
                triggers.add(tok)

    for t in sorted(triggers - known):
        errors.append(f"[状态机] 10 章转移表引用了不存在的消息 `{t}`")

    for n in sorted(known - NON_TRANSITION_MESSAGES):
        if not re.search(rf"\b{re.escape(n)}\b", s10):
            errors.append(f"[状态机] 消息 `{n}` 未在 10 章状态机中出现（缺少状态转移或说明）")

    notes.append(f"状态机转移触发消息: {len(triggers)} 个")


# ------------------------------------------------------ 10. Topic 规范形式
def check_topic_canonical() -> None:
    """Server 订阅过滤器的规范写法是 `/jiayi/fsp/client/+/#`。"""
    for name in sorted(p.name for p in SPEC.glob("*.md")):
        text = spec(name)
        if not text:
            continue
        for m in re.finditer(r"(/jiayi/fsp/(?:client|server)/\+)/([^/\s|）)]*)", normative(text)):
            tail = m.group(2)
            if tail == "":
                errors.append(
                    f"[{name[:2]}] Topic 过滤器 {m.group(1)} 仅为 4 层，"
                    f"匹配不到 5 层消息，应写为 {m.group(1)}/#"
                )
            elif tail == "+":
                errors.append(
                    f"[{name[:2]}] Topic 过滤器 {m.group(1)}/+ 的写法非规范，"
                    f"应写为 {m.group(1)}/#"
                )
        # 历史说明段落也统一使用规范写法，避免正文与注释不一致
        for m in re.finditer(r"/jiayi/fsp/(?:client|server)/\+/\+", text):
            errors.append(
                f"[{name[:2]}] 出现非规范 Topic 过滤器 {m.group(0)}，规范写法为 +/#"
            )


# ------------------------------------------- 11. 非法行为 ↔ Topic 列表互斥
def check_illegal_behaviors() -> None:
    """22 章「非法行为」禁止 Client 发布的消息，不得同时出现在 C→S Topic 列表中。"""
    text = spec("22-topic-spec.md")
    if not text:
        return
    body = _section(text, "## 非法行为", "## ")
    forbidden: set[str] = set()
    for line in body.split("\n"):
        # 只认「以 Client 为主语」的禁令行，避免命中说明性文字
        if not re.match(r"^\s*-\s*\**Client\b", line) or "发布" not in line:
            continue
        for m in re.finditer(r"`([a-z][a-z0-9_]*)`", line):
            forbidden.add(m.group(1))

    c2s = _section(text, "### Client → Server", "### Server → Client")
    published = {
        m.group(1)
        for m in re.finditer(r"^- `/jiayi/fsp/client/\{client_id\}/([a-z0-9_]+)`", c2s, re.M)
    }
    for n in sorted(forbidden & published):
        errors.append(
            f"[非法行为] 22 章既禁止 Client 发布 `{n}`，"
            f"又把 `/jiayi/fsp/client/{{client_id}}/{n}` 列入了 Client → Server Topic 清单"
        )


# ------------------------------------------------- 12. Markdown 代码围栏配对
def check_code_fences() -> None:
    """每个规范文件的 ``` 代码围栏必须成对出现，否则 Markdown 渲染会被破坏。"""
    for name in sorted(p.name for p in SPEC.glob("*.md")):
        text = spec(name)
        if not text:
            continue
        fences = sum(1 for line in text.split("\n") if line.lstrip().startswith("```"))
        if fences % 2 != 0:
            errors.append(
                f"[{name[:2]}] 代码围栏数量为奇数（{fences}），存在未闭合的 ```，"
                f"Markdown 渲染会被破坏"
            )


# --------------------------------------------- 13. 关键载荷字段守卫（P0 回归）
REQUIRED_PAYLOAD_FIELDS = {
    "login": {"proto_version", "device_id"},
    "login_ack": {"result", "code", "proto_version"},
    "upload_accept": {"session_id", "chunk_count", "window", "udp_port", "tcp_port", "chunk_size"},
    "download_accept": {"session_id", "chunk_count", "window", "udp_port", "tcp_port", "chunk_size"},
    "missing_chunks": {"mode", "session_id", "chunks"},
    "resume_ack": {"session_id", "next_chunk_id", "udp_port"},
    "upload_complete": {"result", "code", "file_hash"},
    "download_complete": {"result", "code"},
    "session_abort": {"session_id", "reason"},
}


def parse_06_payloads() -> dict[str, set[str]]:
    """解析 06 章 JSON 载荷块：type -> 顶层字段集合。"""
    text = spec("06-control-plane-mqtt.md")
    if not text:
        return {}
    out: dict[str, set[str]] = {}
    for block in re.finditer(r"```json\n(.*?)\n```", text, re.S):
        body = block.group(1)
        m = re.search(r'"type"\s*:\s*"([a-z][a-z0-9_]*)"', body)
        if not m:
            continue
        keys = set(re.findall(r'^\s{2}"([a-z][a-z0-9_]*)"\s*:', body, re.M))
        out.setdefault(m.group(1), set()).update(keys)
    return out


def check_payload_guards() -> None:
    """关键消息必须携带协议赖以工作的字段（防止再次删掉端口/判别字段）。"""
    payloads = parse_06_payloads()
    if not payloads:
        return
    for msg, required in sorted(REQUIRED_PAYLOAD_FIELDS.items()):
        fields = payloads.get(msg)
        if fields is None:
            errors.append(f"[载荷] 06 章缺少消息 {msg} 的 JSON 载荷")
            continue
        missing = sorted(required - fields)
        if missing:
            errors.append(
                f"[载荷] 消息 {msg} 的载荷缺少必需字段: {', '.join(missing)}"
            )


# ---------------------------------------------- 14. next_chunk_id 定义守卫
def check_resume_semantics() -> None:
    """next_chunk_id 必须是「第一个尚未收到的块」，不得回退为 max+1。"""
    s06 = spec("06-control-plane-mqtt.md")
    if not s06:
        return
    rows = [
        line for line in s06.split("\n")
        if line.strip().startswith("| next_chunk_id")
    ]
    if not rows:
        errors.append("[06] resume_ack 的字段表中缺少 next_chunk_id 行")
        return
    for row in rows:
        if "第一个尚未收到" not in row:
            errors.append(
                "[06] next_chunk_id 必须定义为「第一个尚未收到的 chunk_id」，"
                "不得使用 max_received_id + 1（乱序协议下会漏掉空洞块）"
            )


# ------------------------------------------------- 15. window 默认值一致性
def check_window_default() -> None:
    """window 的默认值在各处必须一致。"""
    values: dict[str, set[str]] = {}
    for name in sorted(p.name for p in SPEC.glob("*.md")):
        text = spec(name)
        if not text:
            continue
        for line in text.split("\n"):
            if "window" not in line and "窗口" not in line:
                continue
            if "默认" not in line:
                continue
            if "Mbps" in line or "MB/s" in line:
                continue
            nums: list[str] = []
            for m in re.finditer(r"默认", line):
                tail = line[m.end(): m.end() + 30]
                # 只取「默认」后紧邻的第一个数字，避免把 "（见 11 章）" 之类的引用计入
                found = re.search(r"(?<![\d.])(\d{2,6})(?![\d.])", tail)
                if found:
                    nums.append(found.group(1))
            if nums:
                values.setdefault(name, set()).update(nums)
    allnums = set().union(*values.values()) if values else set()
    if len(allnums) > 1:
        detail = "，".join(f"{k}={sorted(v)}" for k, v in sorted(values.items()))
        errors.append(f"[流控] window 的默认值在各章不一致: {detail}")


# ------------------------------------------- 16. 引用块规范性守卫（文档约定）
HIST_MARKERS = (
    "v1.0.0", "v1.1.0", "v0.2", "草案", "已在", "更正", "此前", "原先", "旧版",
)


def check_quote_blocks() -> None:
    """引用块（`>`）只用于历史变更说明；规范性「必须/不得/应当」必须写在正文。

    这条约定是 load-bearing 的：检查器的 `normative()` 会跳过引用块，
    若规范性要求藏在引用块里，它既不受矛盾性检查保护，读者也无法识别其效力。
    """
    for name in sorted(p.name for p in SPEC.glob("*.md")):
        text = spec(name)
        if not text:
            continue
        block: list[str] = []
        start = 0
        for i, line in enumerate(text.split("\n") + [""]):
            if line.lstrip().startswith(">"):
                if not block:
                    start = i + 1
                block.append(line)
                continue
            if block:
                body = " ".join(x.lstrip("> ").strip() for x in block)
                if (any(k in body for k in ("必须", "不得", "应当"))
                        and not any(h in body for h in HIST_MARKERS)):
                    errors.append(
                        f"[{name[:2]}] 第 {start} 行起的引用块包含规范性「必须/不得/应当」，"
                        f"但不是历史变更说明；规范性要求必须写在正文（引用块仅用于历史说明）"
                    )
                block = []


def main() -> int:
    for fn in (
        check_endianness,
        check_session_id,
        check_file_hash,
        check_topic_levels,
        check_proto_alignment,
        check_version,
        check_message_inventory,
        check_payloads,
        check_state_machine,
        check_topic_canonical,
        check_illegal_behaviors,
        check_code_fences,
        check_payload_guards,
        check_resume_semantics,
        check_window_default,
        check_quote_blocks,
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
