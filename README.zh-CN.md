# OpenCase

声明式测试用例管理：测试用例是 git 管理的纯声明文件（`cases/*.md`），agent 负责拆解与执行，**人类独占 review 门**。零数据库，零运行时依赖，一个 Rust 二进制。

[![CI](https://github.com/realmorrisliu/opencase/actions/workflows/ci.yml/badge.svg)](https://github.com/realmorrisliu/opencase/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/realmorrisliu/opencase)](https://github.com/realmorrisliu/opencase/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

English version: [README.md](README.md)

## 目录

- [为什么存在](#为什么存在)
- [核心概念](#核心概念)
- [状态机](#状态机)
- [安装](#安装)
- [快速开始](#快速开始)
- [Case 格式](#case-格式)
- [命令参考](#命令参考)
- [Agent 集成](#agent-集成)
- [参与贡献](#参与贡献)
- [开发](#开发)
- [License](#license)

## 为什么存在

| | 测试管理平台（TestRail 式） | 自动化框架（Playwright 式） | OpenCase |
|---|---|---|---|
| 用例声明层 | 有，但在数据库里，与代码脱节 | 无 | 有，git 文件，随代码走 |
| 执行 | 靠人 | 有 | 混合：manual 归 agent，scripted 归 CI |
| 谁签收 | 混乱 | 无 | 执行记录带日期 + commit |
| 用例 vs 脚本 | 脱节 | 无此概念 | `covered-by` 链接 + 校验 |

## 核心概念

- **纯声明**：case 文件是唯一真相。review 通过 git diff 发生。
- **review 门**：`draft → reviewed` 由 grill 式 review 会话（case-reviewer）驱动——每个 case 逐个呈现给你决策；显而易见的 case（期望逐条可溯源、无冲突无歧义）agent 可自批并在会话摘要说明理由，其余必须你拍板。未 review 的 case 禁止执行；编辑过的 case 自动回落 draft。
- **混合执行**：`manual` 模式由 agent 扮演手动测试工程师（CLI 出 prompt，agent 用浏览器工具执行，结果落盘为带日期的记录）；`scripted` 模式结果归 CI，OpenCase 只校验 `covered-by` 链接不撒谎。
- **失败强制归因**：fail 记录必须归类 `product-bug` / `test-bug` / `environment`，不允许模糊。
- **漂移狩猎**：对 scripted case 重跑 manual，产出"观察 vs 期望"的 diff——抓脚本抓不到的变化。
- **漂移提示**：scriptify 时记录 Steps/Expected 的哈希（`drift-sha`）；`review` 对比并在 case 变更后警告，防止 covered-by 脚本悄悄过期。

## 状态机

```
draft ──review --approve（review 会话）──▶ reviewed
  ▲                                        │
  └──────────review --edit（任何编辑）───────┘
                    │
                    ▼
        run / record / scriptify 的门：reviewed
        record 额外要求：mode=manual
```

## 安装

每个 [release](https://github.com/realmorrisliu/opencase/releases) 都附带了 macOS（Apple Silicon）、Linux（x86_64）、Windows（x86_64）的预编译二进制，**不需要 Rust 工具链**。

**一行安装（macOS / Linux）：**

```bash
curl -fsSL https://raw.githubusercontent.com/realmorrisliu/opencase/main/install.sh | sh
```

固定版本：`VERSION=v0.1.2 sh install.sh`。脚本装到 `/usr/local/bin`（需要时请求 sudo）。Windows 用户从 releases 页下载 `.exe` 放入 PATH。

**源码安装（需要 Rust）：**

```bash
cargo install opencase
```

然后在任意项目仓库里：

```bash
opencase init        # 生成 cases/ 目录和一个示例 case
opencase validate    # 检查所有 case
```

## 快速开始

```bash
opencase init
opencase validate                  # 1 case(s), 0 problem(s)
opencase review                    # 列出待 review 的 case
opencase review <id> --approve     # 审批（由 review 会话驱动）
opencase run <id>                  # 给 agent 的执行 prompt
opencase record <id> --result pass
opencase report                    # 状态/覆盖率/最近执行
```

仓库自带 5 个 dogfood case（`cases/`），用 OpenCase 测试它自己；`validate` 应为 `5 case(s), 0 problem(s)`。

## Case 格式

```markdown
---
id: login-success              # 必填，全局唯一
title: Login succeeds          # 必填
status: draft                   # draft | reviewed
mode: manual                    # manual | scripted
source: Feishu PRD §2.1 (token: xxx)   # 必填，来源溯源
covered-by: tests/login.spec.ts # 可选；scripted 必填
drift-sha: 0c3e6a...           # scriptify 写入；review 用它对比 Steps/Expected 是否漂移
---

## Steps

1. Open the login page
2. Enter a correct account and password
3. Click "Log in"

## Expected

- Redirected to the home page
- Username visible in the top-right corner

## Executions      ← 由 CLI 维护，勿手改

- 2026-08-08 | abc1234 | pass
- 2026-08-09 | def5678 | fail | product-bug | button unresponsive, filed issue #12
```

- frontmatter 刻意扁平：`key: value`，值不含换行、`|`
- 执行记录行：`- YYYY-MM-DD | <commit> | pass|fail [ | category ] [ | note ]`，失败行必须带归因；pass 行的第一个附加字段是 note
- `covered-by` 相对仓库根解析（如 `tests/login.spec.ts`），validate 检查文件存在

## 命令参考

| 命令 | 作用 | 门 |
|---|---|---|
| `init` | 生成 `cases/` 目录和一个示例 case | — |
| `validate` | 校验 schema、状态机、记录行、covered-by | — |
| `review [id]` | 列出 draft + 漂移警告；`--approve` 审批；`--edit` 打开编辑器（reviewed 编辑后回落 draft）。由 case-reviewer 会话驱动，不是给人敲命令的 | 审批由 review 会话决策 |
| `run <id>` | 输出执行 prompt（Steps + Expected + source） | reviewed |
| `record <id> --result pass\|fail [--category] [--commit] [--note]` | 追加带日期的执行记录；fail 必带归因；commit 缺省取 git HEAD | reviewed + manual |
| `report` | markdown 报告：状态、模式、覆盖率、最近执行、draft 清单 | — |
| `scriptify <id> [--covered-by <path>] [--rebaseline]` | 输出转换上下文（Steps + 执行记录），翻转 case 为 scripted；`--rebaseline` 在脚本更新后刷新漂移基线 | reviewed + manual |

全局 `--cases <dir>` 指定用例目录（默认 `cases/`）。

## Agent 集成

agent skills 是 OpenCase 的另一半——真正写、审、执行 case 的是它们。它们遵循 [Agent Skills 开放标准](https://agentskills.io/specification)，且已内嵌进二进制，直接用 CLI 安装（不需要克隆仓库）：

```bash
opencase skill install                 # 默认 pi（~/.agents/skills）
opencase skill install --agent claude  # 或 codex、project（当前目录 .agents/skills）
opencase skill install --force         # 覆盖本地修改
```

如果你的 harness 自带 skill 安装器（如 `npx skills add realmorrisliu/opencase` 或 `gh skill install realmorrisliu/opencase`），指向本仓库也可以——skills 在 `skills/` 目录下，每个都是独立的 `SKILL.md`。

- **case-writer**：从多源需求（飞书 PRD、spec 文档、grill/to-prd 产物）拆解 happy path / 边界 / 异常流，带溯源写 case，不审批
- **case-reviewer**：grill 式 review 会话——逐 case 呈现给你决策（approve / 修改 / 跳过）；显而易见的 case 自批并在会话摘要留痕；一次一个，不批量倾倒；人不在场不启动、不自批
- **case-executor**：run → 真实执行 → 对照 Expected → record 归因；product-bug 用 `gh issue create` 起草；归因不确定默认 test-bug 并询问；含漂移狩猎流程

典型循环：

```
case-writer 根据 PRD 写 case → case-reviewer 开 review 会话（你决策）
→ case-executor 执行并记录 → report 看状态 → scriptify 把稳定 case 交给 CI
```

## 参与贡献

见 [CONTRIBUTING.md](CONTRIBUTING.md)——设计决策都落在 [`docs/PRD.md`](docs/PRD.md)，改动格式、状态机、门之前先读它。工具刻意保持薄，优先最小改动。安全问题按 [SECURITY.md](SECURITY.md) 私下报告。社区规范：[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。

## 开发

```bash
cargo test    # 单元测试 + 端到端冒烟（tests/smoke.rs 驱动真实二进制走全链路）
```

## License

MIT — 见 [LICENSE](LICENSE)。

---

[CONTRIBUTING.md](CONTRIBUTING.md) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) · [SECURITY.md](SECURITY.md)
