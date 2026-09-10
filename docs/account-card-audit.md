# OAuth 账户卡片：实现与验收

日期：2026-09-09。范围包括紧凑卡片的五项要求，以及后续标准图标、ChatGPT 标志、模型禁用与别名要求。Git 基线为 `555ce8e`；保留已有 OAuth 实现及之前的 `docs/oauth-login-audit.md`、测试脚本修改。未提交 Git commit。

## 先核实已有内容，再继续剩余部分

| 核实对象 | 实际状态 | 依据 |
| --- | --- | --- |
| OAuth 登录、回调、认证文件、模型规则、上传下载 | 已存在，保留并回归验证 | `accounts.rs`、`oauth_login.rs`、`ui/llm.js`，真实网关浏览器测试 |
| 账户额度、启停、删除、健康观测 | 本次持续任务前半段已经补齐，后续保留 | Git 工作区差异、`account_management.rs`、`server/mod.rs`、管理接口测试 |
| 最新要求的紧凑标题、信息行、无框额度条、下方逐卡重置、整体缩小 | 收到最新截图时尚未完成；本次继续完成 | 旧卡片仍有大图标、CODEX 标签、独立邮箱、长文件名、健康区和全局重置按钮；逐项检查当前 DOM 与截图 |
| 指定某张重置卡 | 原实现未完成，只发送 `redeem_request_id`；现已贯通 `credit_id` | OpenAI 官方客户端、管理接口参数、供应商模拟服务收到的请求体 |
| 重置结果判断 | 原实现将所有 HTTP 2xx 当成成功；现已修复 | 官方响应类型，已知四类结果与未知结果测试 |

## 对照来源

- CLIProxyAPI 本地仓库 `/Users/mfordjody/sdk/code/CLIProxyAPI`：`internal/api/handlers/management/auth_files.go`、`auth_files_fields.go`、`auth_files_crud.go`、`quota.go`。读取账户摘要、启停、删除和本地冷却清除实现；该仓库未修改。
- 其管理前端是另一个仓库。按 `panel-github-repository` 配置读取 [Management Center，固定提交 ed5f1c4](https://github.com/router-for-me/Cli-Proxy-API-Management-Center/tree/ed5f1c48e11ba7335f1e8f676f228c280196af85)：`src/features/quota/providers/codex/data.ts`、`CodexQuotaBody.tsx`、`src/utils/quota/{constants,resetCredits}.ts`。
- 参考前端以 `100 - used_percent` 表示剩余额度；查询分别使用 `wham/usage` 和 `wham/rate-limit-reset-credits`。CLIProxyAPI 自身的管理 `/reset-quota` 清除本地冷却，不等于消费供应商重置卡。
- 指定重置卡的依据：[OpenAI 官方客户端](https://github.com/openai/codex/blob/main/codex-rs/backend-client/src/client/rate_limit_resets.rs) 中的 `consume_rate_limit_reset_credit_by_id`，请求同时携带 `redeem_request_id` 和 `credit_id`；其 [合同测试](https://github.com/openai/codex/blob/main/codex-rs/backend-client/src/client/rate_limit_resets_tests.rs) 验证指定卡的请求体。
- 结果码依据：[OpenAI 官方响应类型](https://github.com/openai/codex/blob/main/codex-rs/backend-client/src/types.rs) 中的 `ConsumeRateLimitResetCreditCode`：`reset`、`already_redeemed`、`nothing_to_reset`、`no_credit`。本项目对未知结果保留待确认请求，不报告成功。

## 最新五项要求与逐步执行依据

1. **压缩标题与套餐信息。** 小图标右侧一行显示 `codex-邮箱-hash`，超长省略并通过 title 显示完整名称。hash 是由账户 ID 计算的稳定七位显示标识，不改存储 ID。下一行显示套餐、续费时间、主动重置次数，去掉胶囊外框。套餐取真实供应商数据，缺失续费时间显示“—”。依据：`ui/llm.js::llmSubscriptionCard`、`llmQuotaMarkup`；浏览器断言标题格式与实际截图。
2. **改成无外框额度条。** 每行保留窗口名、剩余百分比、重置时间及相对时间；条形区取消边框与内层卡片。仍支持五小时、周、月和附加窗口，剩余不足 20% 使用警示色，未来一小时内的时间提示红色。依据：`ui/llm.css` 的 `.llm-quota-panel` 与 `.llm-quota-track`；浏览器验证边框为 `0px`，已用 99% 显示剩余 1%。
3. **移除长文件名显示及冗余区块。** 不再显示 `codex-UGJle…MK8.json` 那一行；同时移除重复供应商标签、独立邮箱大标题、健康条、文件大小/修改时间和后端信息行。认证文件保留，否则账户会失效；下载与设置仍可用。依据：DOM 中 `.llm-file-name`、`.llm-file-meta`、`.llm-health`、`.llm-binding` 数量为 0；更新实际服务前后认证文件哈希一致。
4. **重置卡移到额度条下方，逐卡操作。** 每张卡一行，展示序号、到期时间和“使用重置”；无额外全局重置按钮。点击后确认所选账户和该卡到期时间，再发送该卡的 `credit_id`。无卡时不显示操作按钮；缺少卡编号或已过期时不允许消费。依据：`llmQuotaMarkup`、`llmResetQuota`；DOM 顺序与全局按钮缺失断言，模拟上游实际收到第二张卡 `credit-1`。
5. **补齐前后端参数与准确结果。** `QuotaReset.credit_id` → `LlmAccounts::reset_account_quota` → 供应商消费接口，使用同一编号。`reset` 才表示本次重置；已兑换、无需重置、无卡各有对应提示。未知响应不转换为成功。依据：`crates/ui/src/llm.rs`、`account_management.rs::reset_result_code`；Rust 单测和实际 HTTP 测试。
6. **避免中断后消费另一张卡。** 每次请求先保存私有回执，绑定请求编号与卡编号；同卡重试复用编号，不同卡或同一编号换卡返回冲突；已完成结果保留以供重放。页面刷新、会话丢失和网关重启后通过账户摘要恢复待确认卡，即使上游已将它从可用列表移除也能重试确认。依据：`prepare_reset_receipt`、`pending_reset_credit_id`；模拟消费生效后返回 502，再重启网关、清空 sessionStorage、刷新页面并重试，记录仍只有一次消费。
7. **缩小整体尺寸与工具栏。** 卡片最大宽度 360 CSS px，14px 内边距，小图标和紧凑工具栏；保留模型、刷新额度、下载、设置、删除和启停功能。依据：`ui/llm.css`；真实账户桌面为 360×176、390px 视口为 358×176；三条额度加两张重置卡约 360×388，测试上限 430px，无横向溢出。
8. **修复精简导致的搜索回归。** 首次 smoke 测试在第 90 行报 `0 !== 1`，原因是被移除的信息行原本承担了账户编号搜索。现把账户 ID、后端、邮箱和供应商放入 `data-search`，原有搜索行为保留。依据：`ui/ui.html::applyPageFilter`、`llmSubscriptionCard`；相同测试重跑通过。
9. **更新实际运行页面。** 核实原有进程参数和账户目录，重新构建并以相同参数替换本机网关。保留一个真实账户、原配置和账号绑定；`http://127.0.0.1:15021/healthz` 正常。实际供应商返回 Free、月额度剩余 1%、可用重置卡 0、无续费时间。依据：进程参数、重启前后文件哈希和真实浏览器只读查询；没有消费真实重置卡。
10. **回头检查源码与结果。** 搜索确认无旧长文件名显示节点、无全局重置按钮和额度面板边框；检查每个按钮仍接真实接口；核对卡片截图、移动端、错误结果及中断重试。清除工具生成的训练日志与本任务临时测试目录；保留用户运行中的服务、构建产物和原有修改。

## 前后端对应

| 展示或操作 | 接口 / 数据源 | 验证 |
| --- | --- | --- |
| 名称、套餐、续费、待确认卡 | `GET /debug/llm` | 白名单摘要，不向页面暴露 token |
| 查询额度、重置卡明细 | `POST /admin/llm/accounts/:id/quota` | 真实供应商 HTTP；按账户隔离；失败保留观测 |
| 使用指定重置卡 | `POST /admin/llm/accounts/:id/reset-quota`，`redeem_request_id` + `credit_id` | 指定第二张卡、准确结果、冲突、跨重启幂等 |
| 启停 | `PATCH .../:id/status`，修订号 + disabled | 版本冲突 409；重启保留；禁用不转发 |
| 删除 | `DELETE .../:id`，修订号 | 认证、持久化、批量及部分失败处理 |
| 模型、设置、替换文件 | `GET/PUT .../:id` | 规则进入实际代理请求；保留账户绑定 |
| 下载、刷新 token | `.../:id/download`、`.../:id/refresh` | 原有认证与凭证轮换测试保留 |

## 最终验证结果

- `rtk cargo test --workspace`：**232 passed，2 ignored，22 suites，退出码 0**。忽略项为原有本机性能/延迟基准。
- `rtk cargo build --bins`：通过；构建 `transit` 单一二进制入口。
- `tests/llm-workspace.cjs`：最终构建 + 浏览器 + 本地供应商 HTTP 通过。覆盖指定卡 ID、未知结果、换卡冲突、无卡/无需重置、待确认卡跨页面刷新和进程重启、账户启停/删除/批量、真实代理与原有 OAuth 链路。
- `tests/ui-smoke.cjs`：八个页面、1440/1024/768/390px、主题、键盘、搜索、错误恢复全部通过。
- 真实页面：中文深色、360×176 和 358×176，无脚本错误或横向溢出。认证文件仍为 1 个，更新时内容不变；未改变账号套餐或真实额度。
- `git diff --check` 通过；CLIProxyAPI 仓库干净。未修改无关源码；之前的 OAuth 验收文件修改保留。

## 可能出错或被忽略的边界

- **参考图的 Plus 不是当前账户套餐。** 当前实际返回 Free；不会为了外观填充 Plus、续费日期、五小时/周窗口或重置卡。多窗口与逐卡 UI 由明确的模拟数据验证，截图与真实账户截图分开保存。
- **未进行真实消耗性重置。** 真实供应商仅查询额度；消费协议和幂等由真实 HTTP 的本地供应商模拟验证。未重新执行真实 Claude 账号登录。
- **上游协议和网络可能变化。** 401 尝试刷新令牌后重试一次；未知消费结果保留请求编号供确认，403/网络失败不伪装为零额度。重置成功但刷新失败单独提示。
- **旧版遗留 pending 回执没有卡 ID。** 不擅自把它绑定到新点击的卡；拒绝新消费并报告冲突，需要核实旧请求结果。损坏回执或不可写存储同样不绕过保护。当前真实账户没有这类回执。
- **落盘回执必须可写。** 旧 `.reset.tmp` 残留或存储错误会明确阻止重置，应核实文件后处理；不通过删除回执来自动重发可能已经成功的消费。
- **单目录按单网关进程使用。** 锁在进程内；不支持多个网关共享同一账户目录并发消费。额度观测在进程内存，重启后重新查询；启停与回执持久化。
- **真实代理路由未被本次 UI 请求创建。** 本机当前没有后端配置，账户保留旧绑定 `codex-team-sub`；账户登录、管理和额度查询可工作，实际转发仍需对应后端。独立联调配置已验证模型规则和禁用账户不转发。
- **展示 ID 不是存储 ID。** 七位 hash 用于区分同邮箱卡片，不用于认证或安全判断；完整原 ID 仍可搜索和通过设置查看。
- **通读范围说明。** 没有声称逐个打开两个仓库所有文件；此记录只对实际核实的源码、协议、测试和运行结果负责。

## 最终漏项检查

| 用户要求 | 状态 | 复查依据 |
| --- | --- | --- |
| 单行 codex-邮箱-hash、套餐、续费 | 完成 | 标题正则、真实 Free 截图、信息行 |
| 额度条无外框 | 完成 | computedStyle 边框 0px、截图 |
| 不再显示长 .json 文件名 | 完成 | 旧 DOM 节点 0 个、认证文件保留 |
| 额度条下逐卡重置，无额外按钮 | 完成 | DOM 顺序、credit_id 请求体、无全局按钮 |
| 缩小整体卡片 | 完成 | 桌面和手机实测尺寸、多卡场景上限断言 |
| 步骤、结论依据、风险、回头检查 | 完成 | 本文源码指针、协议来源和测试记录 |

## 后续模型管理与图标要求

- 工具栏采用统一 24×24 viewBox、圆端点、1.8 线宽；模型使用分层符号，刷新/下载/设置/删除统一尺寸和对齐。ChatGPT 标志来自上文固定提交的 OpenAI SVG，按主题着色并移除紫色底框，许可保存在 `ui/THIRD_PARTY_NOTICES.md`。依据：`llmIcon`、`llmChatGptLogo`、`.llm-provider-mark` 与页面截图。
- 点击模型按钮打开“OAuth 模型禁用”和“OAuth 模型别名”两个区域。规则作用于当前账户；模型列表由新增的认证管理 `GET /admin/llm/accounts/:id/models` 查询供应商实际目录，失败允许手动输入，不伪造模型清单。Codex 查询方式依据 [官方 ModelsClient](https://github.com/openai/codex/blob/main/codex-rs/codex-api/src/endpoint/models.rs) 的 `models?client_version` 接口。
- 禁用支持精确选择和大小写不敏感的 `*` 通配符；判断原模型与别名对应的源模型，不能通过别名绕过禁用。依据：`OAuthAccount::resolve_model`、`model_pattern_matches` 和实际代理 HTTP 测试。
- 新增别名默认不保留原名；勾选“保留原名”后两种名称都能使用。旧认证文件缺少 `keep_original` 时默认保留原行为。依据：`ModelRule` 的 serde 默认值和路由测试。
- 订阅后端的 `GET /v1/models` 合并当前后端启用账户的目录，过滤后端不支持或账户禁用的模型，并按别名/保留原名规则输出。此目录不会影响其它 API 或本地模型后端的原有模型列表路径。
- 实际联调用例验证：禁用 `gpt-hidden`、通配符排除 `gpt-reserve-test`，把 `gpt-test` 改为 `friendly` 后目录只含 `friendly`；被禁用模型与旧名称的请求未到达上游；别名请求实际发送 `gpt-test`；启用“保留原名”后目录包含两者；再禁用源模型后目录为空且别名请求也被拒绝。保存、重启后规则仍在。
- 当前账户仍没有代理后端配置；本次只读验证真实模型目录，不擅自替用户禁用或重命名任何真实模型。模型目录范围、别名与禁用生效已由独立联调配置验证。
- 本次结尾工作区完成 `transit` 全面改名。早先构建与 232 项 Rust 测试通过；最后整库编译与实际服务更新在改名引用一致后再次验证。

## 本任务最终验收补充

- 真实页面 `http://127.0.0.1:15021` 已更新为包含模型管理和 ChatGPT 标志的已验证构建。实际读取到 **5 个供应商模型**；只打开管理面板检查，没有保存真实账户禁用/别名规则，也没有消耗真实重置卡。
- ChatGPT 图形继承当前主题颜色，标志容器 computedStyle 背景为透明；工具栏五个 SVG 均为 16×16。截图检查确认无紫色框、图形居中、按钮含用途标签。
- 模型弹窗在 1440px 与 390px 视口中均无横向溢出；手机自动纵向排列两个区域。
- 最新 `ui/llm.js`、`ui/llm.css` 通过 `UI_SOURCE_ASSETS=1` 在已验证网关上运行完整浏览器联调：静态资源取当前磁盘源码，其余管理接口、凭证持久化、模型路由和供应商 HTTP 请求仍走真实网关进程。`/tmp/transit-model-management-source-test.log` 的本任务结果已复制到交付截图目录。该模式验证最后的通配符计数反馈。
- 用户明确要求改名由另一个 AI 处理。本任务不继续改名工作，也不把其未完成构建算成本任务模型功能的成功证明；模型与图标以本节的真实页面和完整功能联调为依据。
- 最后逐项核对：紧凑卡片、名称格式、无框额度条、移除文件名、逐卡重置、标准图标、无框 ChatGPT 标志、禁用原名/别名、通配符、修改模型名称、保留原名开关、目录过滤、保存与重启持久化均有源码及测试/渲染依据。
