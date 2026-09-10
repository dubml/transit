# Configuration：实现、依据与验收

本次范围是四张参考图中的接入与认证、TLS、远程管理，以及仅显示本项目版本和连接状态的紧凑信息栏。接手时配置页仅支持查看、复制运行配置 JSON；OAuth 账户管理和本地管理会话已经存在。

## 逐步记录

1. **核实当前源码。** 检查工作区状态、`ui/ui.html` 的配置页与 `renderConfiguration`，以及 `crates/ui/src/lib.rs` 的路由。确认需要增加可保存的接入配置接口，已有账户功能继续复用。
2. **读取 CLIProxyAPI 的实际实现。** 对照下表中的配置定义、鉴权、监听、重载和面板下载代码。截图里的 API 密钥是客户端调用网关的凭证；管理密钥用于配置与账户管理；两者不能混用。
3. **建立接入配置存储。** 新增独立的 `AccessSettings`。保存时校验字段、版本冲突、证书与密钥，通过同目录临时文件原子替换；文件权限为 `0600`。管理密钥保存为带随机盐的 PBKDF2-HMAC-SHA256 哈希，不在读取接口中回传。
4. **接入启动过程。** 新增 `--access-config` / `TRANSIT_ACCESS_CONFIG`。默认文件为 `$XDG_CONFIG_HOME/transit/access.json`，没有该环境变量时使用 `~/.config/transit/access.json`。首次读取采用启动参数；文件保存后，其接入配置在后续启动时优先。路由仍来自原有静态配置或 xDS。
5. **接入真实认证。** `/admin/*`、`/debug/*` 使用统一管理认证。客户端 API 密钥支持 Bearer、`X-Api-Key`、`X-Goog-Api-Key`、`key` / `auth_token` 查询参数。路由认证完成后移除已消费的网关凭证，再注入上游所需凭证。
6. **实现紧凑表单。** 增加主机、端口、认证目录、API 密钥增删改与复制、TLS 证书路径、远程管理开关与管理密钥。折叠区域、暗亮主题、中英文、未保存草稿、冲突和断线状态均接入实际接口；保留运行配置 JSON 查看入口。
7. **实现 TLS 与面板分发。** 网关和管理监听器共享配置的 TLS 证书；支持 HTTP/1.1、HTTP/2。可选 GitHub 面板仓库下载 `management.html`，每 3 小时检查；停用自动更新时仍允许首次补齐缺失文件。下载失败保留现有或内置面板。
8. **修复联调中发现的问题。** HTTPS 浏览器请求使用 HTTP/2 `:authority`，不能只读取 `Host`；已补齐同源校验。原账户页 `llmReload` 未携带管理凭证，新增统一认证后会返回 401；已改用统一请求方法。切换语言时保留草稿、折叠状态、版本和连接信息。复查还发现后台刷新不能更新未保存草稿的基准修订号；已单独保存 `baseRevision`，避免覆盖另一页面的更改。
9. **验证实际行为。** Rust 测试验证配置存储、权限边界和面板缓存；浏览器测试启动真实网关与本地上游，检查保存、密钥轮换、代理认证、重启、HTTPS、目录切换和面板关闭。
10. **回查范围与依据。** 下表逐项对应截图；没有把配置保存成功当作监听参数立即生效，也没有把本地模拟提供商测试称为真实第三方 OAuth 登录。

## 字段作用与源码依据

| 要求 | 实际作用 / 生效时机 | CLIProxyAPI 依据 | 本项目实现 |
|---|---|---|---|
| 主机、端口 | 设置网关监听地址；保存后重启生效 | `config.example.yaml:1`、`internal/api/server.go:250` | `crates/proxy/src/access_settings.rs`、`crates/app/src/main.rs` |
| 认证目录 | 加载、保存 OAuth 账户，支持 `~/`；更换后重启加载新目录 | `config.example.yaml:35`、`internal/config/config.go` | `AccessSettings`、现有 `LlmAccounts` |
| API 密钥列表 | 验证客户端请求；保存后立即生效；空列表关闭此层校验 | `internal/access/config_access/provider.go:60`、`internal/api/handlers/management/config_lists.go:145` | `AccessSettings::authenticate_api`、`server/mod.rs` |
| TLS 开关、证书、私钥 | 校验 PEM、有效期与证书私钥匹配；重启后两个监听器使用 HTTPS | `internal/config/config_types.go:183`、`internal/api/server.go:273` | `tls_config`、`validate_key_pair`、`serve_router` |
| 允许远程访问 | 判断实际连接对端，不能用转发头冒充本机；保存后立即生效 | `internal/api/handlers/management/handler.go:264` | `crates/ui/src/access.rs::guard` |
| 管理密钥 | 保护配置、调试数据及账户操作；轮换后旧密钥失效 | `handler.go:300`、`internal/config/config_load.go:113` | `AccessSettings::authorized/save`、统一管理中间件 |
| 禁用控制面板 | 网页及静态资源返回 404；管理 API 保留 | `internal/api/server_management.go:292` | `access::guard` |
| 禁用面板自动更新 | 禁止后台更新；已有缓存仍可用，缺失时可首次下载 | `internal/managementasset/updater.go:60` | `crates/ui/src/panel.rs` |
| 面板仓库 | GitHub release 的网页资源来源，不更新网关二进制 | `internal/managementasset/updater.go:30` | `PanelAssets`、`scripts/package-management.py` |
| 本项目版本、连接状态 | 读取运行中网关的版本和管理 API 连接结果；不显示其他产品版本或构建时间 | 参考图 4 | `/healthz`、`UiServer::with_version`、`ui/configuration.js` |

参考仓库：`/Users/mfordjody/sdk/code/CLIProxyAPI`。本次没有修改该仓库。

## 验证

- `cargo test --workspace`：234 项通过、2 项忽略，覆盖 21 个测试套件。
- `cargo test -p transit-ui --lib`：15 项通过，包含 release 资源选择、摘要与兼容性检查、禁用更新时的缓存读取。
- `tests/configuration-workspace.cjs`：真实进程验证 API 密钥增删改、复制、匿名管理访问拒绝、版本冲突、管理密钥轮换、旧密钥失效、路由与网关双层认证、重启恢复、HTTPS、认证目录切换、关闭面板后管理 API 仍可使用；检查桌面与手机布局。
- `tests/llm-workspace.cjs`：账户导入、额度、重置幂等、模型规则、禁用/删除、重启持久化、空配置 OAuth 面板回归通过。提供商 HTTP 为本地测试服务。
- `tests/ui-smoke.cjs`：原有 8 个业务页面在 1440 / 1024 / 768 / 390px、暗亮主题下通过，无 JavaScript 错误。
- 本地运行验收：`http://127.0.0.1:15021/#configuration` 显示实际版本 `v0.1.0` 和“已连接”。已有 1 个账户文件的内容哈希保持一致；未替用户保存实际接入配置、密钥或 TLS 设置。
- 严格 Clippy 未通过：现有 `core/config.rs` 的大型枚举及 `core/ledger.rs` 的可派生实现、参数数量告警阻断检查。全仓默认格式检查仍有当前工作区的其他格式差异。这两项不计为通过。

## 容易遗漏的边界

- **保存和应用不同。** API 密钥、管理权限和面板开关立即生效；监听地址、TLS、认证目录需要重启。接口同时返回保存值、当前生效值和 `restart_required`。
- **管理端口独立。** 此项目使用 `--ui-addr` 指定管理监听器，表单中的主机和端口控制网关监听器。允许远程管理仍需管理监听器绑定外部可达地址；不能与网关监听地址冲突。
- **目录不会迁移账户。** 更换目录只切换后续加载位置，不移动、删除原目录文件；配置文件不能放在认证文件目录中，以免被当作 OAuth JSON 加载。
- **本机启动方式沿用项目既有行为。** 未设置管理密钥时，同源回环连接可取得进程内临时管理会话；远程访问必须设置管理密钥。CLIProxyAPI 的“空管理密钥关闭管理 API”没有直接替换本项目已有本机会话机制。设置、清除管理密钥时会轮换临时会话。
- **密钥用途不同。** API 密钥用于调用网关，管理密钥用于修改网关；二者都不是 OpenAI / Anthropic 账户密码。健康检查和原有指标采集接口维持既有用途。
- **并发保存不能静默覆盖。** API 修订号不符或磁盘文件被外部修改时拒绝覆盖。另一个页面保存后重新加载即可；直接编辑配置文件后需重启网关。
- **面板资源必须兼容本项目 API。** 留空使用随程序发布的内置面板。可执行 `python3 scripts/package-management.py /path/to/management.html` 构建自包含文件，并将它作为指定仓库的 release 资源；产物携带 `transit-management-api=1` 标识。CLIProxyAPI 的网页依赖另一套接口，不能直接作为本项目面板使用。
- **面板下载不会升级程序。** 自动更新只替换网页资源；限制 GitHub HTTPS、下载大小，提供摘要时校验摘要。不兼容或失败的下载不会覆盖现有面板。
- **HTTPS 联调采用临时自签名证书。** 已验证实际加密监听与浏览器保存；没有替用户更改正式运行实例的证书、密钥或访问权限。

## 完成回查

四张截图中的可见配置项均有对应的前端输入、后台校验、持久化和实际执行路径。版本与连接状态来自运行实例。错误状态、空密钥列表、TLS/HTTP2、会话失效、同源校验、远程限制、草稿保留和原有账户刷新均已纳入检查。测试结果与尚未通过的全仓检查分别记录，没有用编译成功代替端到端验收。
