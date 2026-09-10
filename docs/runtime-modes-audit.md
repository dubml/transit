# Standalone / Kubernetes 模式边界与验收

状态：实施中。下列目标尚未全部完成，不能据此宣称 Kubernetes 模式可用。

## 目标与完成依据

| 要求 | 当前核实结果 | 完成证据要求 |
|---|---|---|
| 显式区分本机与 Kubernetes | 原启动参数无 mode；默认开启 xDS | 两种模式的启动、无效组合和默认行为测试 |
| Standalone 独立运行 | 现有静态文件单次加载可复用 | 不访问 Kubernetes/dubbod，文件与管理修改持久化、更新实际路由 |
| Kubernetes 独立输入 | 当前仅解析引用的 Secret | Gateway API、TransitService 与关联资源监听、删除、重连、状态反馈 |
| 自有 xDS Server | 当前构建配置为 build_server(false) | 原生 xDS 转换、订阅、ACK/NACK、断线恢复和代理端到端测试 |
| 保留 dubbod 委托能力 | 已有 xDS Client 与扩展协议 | 显式外部 xDS 来源、兼容回归；不同时运行两个配置写入来源 |
| 模式决定管理写入边界 | 现有管理设置仅写本地文件 | 集群配置写回声明资源，数据面不产生本地覆盖配置 |
| 两种模式共用代理能力 | 现有 HTTP、LLM、MCP、A2A 实现可复用 | 现有业务回归与两种模式下真实请求验证 |
| Store 承载结论可核查 | 有正确性测试，无规模容量证明 | 本次不宣称容量上限、不替换 Store；记录已有保护及未验证边界 |

## 操作记录

1. 核对工作区与启动代码。现有 OAuth、账户、Configuration 和命名修改已经在工作区；保留，不重复实现、不归入本次新增成果。
2. 核对配置来源。`main.rs` 分别启动 xDS 和加载静态配置；`apply_config()` 将静态输入也归到 xDS 来源。需要显式管理权与互斥校验。
3. 核对 Kubernetes 和协议入口。现有 Secret 解析器不等同于 Kubernetes 控制器；xDS 当前仅生成客户端。因此不能只加两个模式名称就宣称完成。
4. 增加共用模式描述类型，将运行环境、进程职责和配置来源分别表示。职责不是新增运行模式。
5. 按用户提供的 `agentgateway` 仓库核对部署。其 `controller/cmd/agentgateway/main.go` 是独立 Go 控制器；控制器 Helm 清单部署独立镜像和 xDS Service，数据面清单通过 `XDS_ADDRESS` 接入。`controller/pkg/syncer/krtxds/xds.go` 使用 krt 派生资源。这里采用独立 `transit-controller` 与 Rust `transit`，不把两种职责伪装成同一进程内的两个运行模式。

## 保留的设计边界

- krt 式输入输出核心承接资源集合、校验、关联与派生；代理读取已经发布的运行视图。
- Store 容量验证与动态分片属于独立决定。现有全量快照重建是待测更新成本，不能直接等同于已证实的性能故障。
- 同一资源只能有一个配置权威；外部 xDS 与本地文件不能隐式同时写入。
- 参考 agentgateway，以 Go/krt 控制器与 Rust 数据面分别部署；Rust 代理的运行模式仍只有 standalone 与 kubernetes。
- 参考仓库及另一位 AI 负责的命名工作不在本次修改范围。

## 验收记录

待补充。尚未运行本次新增实现的测试，尚未部署 Kubernetes 资源。
