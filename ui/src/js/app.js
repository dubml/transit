    const I18N = {
      en: {
        overview: 'Overview',
        core: 'Core',
        base: 'Core',
        ai: 'AI',
        setup: 'Setup',
        configuration: 'Configuration',
        mcp: 'MCP',
        a2a: 'A2A',
        resources: 'Resources',
        services: 'APIs',
        search: 'Search',
        themeLight: 'Light',
        themeDark: 'Dark',
        apiServices: 'API services',
        mcpPlugins: 'MCP plugins',
        agents: 'Agents',
        llmProviders: 'LLM providers',
        overviewSubtitle: 'Monitor your gateway\'s configuration, gateway assets, and runtime status',
        viewServices: 'View APIs',
        manageMcp: 'Manage MCP',
        agentRegistryLink: 'Agent registry',
        llmRouting: 'LLM routing',
        protocolDistribution: 'Protocol Distribution',
        protocolsInUse: 'Listener and backend protocols in use',
        policyOverview: 'Policy Overview',
        policyOverviewSub: 'Security, traffic and budget policies',
        quickActions: 'Quick Actions',
        quickActionsSub: 'Common management tasks',
        addService: 'Register Service / Route',
        manageMcpServers: 'Manage MCP Servers',
        configurePolicy: 'Configure Policy',
        testPrompt: 'Test Prompt (Playground)',
        budgetQuota: 'Budget & Quota',
        traffic: 'Traffic',
        totalPolicies: 'Total Policies',
        apiServicesSub: 'Upstream clusters & HTTP routes',
        mcpPluginsSub: 'Tool servers & SSE connections',
        agentsSub: 'A2A protocol & multi-agent routing',
        llmProvidersSub: 'Model routing & fallback strategies',
        registeredServices: 'Active Services',
        upstreamClusters: 'Upstream Clusters',
        routingRules: 'Routing Rules',
        mcpServers: 'Plugin Servers',
        exposedTools: 'Exposed Tools',
        sseTransport: 'Transport Mode',
        registeredAgents: 'Registered Agents',
        commPaths: 'A2A Channels',
        agentProtocol: 'Transport Protocol',
        managedModels: 'Managed Models',
        upstreamProviders: 'Upstream Providers',
        failoverStatus: 'Auto Failover',
        runtimeObservability: 'Runtime & Telemetry',
        runtimeObservabilitySub: 'Health status and live metrics',
        gatewayStatus: 'Gateway Status',
        p99Latency: 'P99 Latency',
        telemetrySpans: 'Telemetry Spans',
        viewObservability: 'View Observability',
        type: 'Type',
        name: 'Name',
        detail: 'Detail',
        endpoints: 'Endpoints',
        domain: 'Domain',
        routes: 'Routes',
        tls: 'TLS',
        mtls: 'mTLS',
        plaintext: 'Plaintext',
        secLevel: 'Security',
        mode: 'Mode',
        protocol: 'Protocol',
        pickService: 'Select a service',
        unhealthy: 'unhealthy',
        kind: 'Kind',
        provider: 'Provider',
        id: 'ID',
        baseUrl: 'Base URL',
        endpoint: 'Endpoint',
        tools: 'Tools',
        agent: 'Agent',
        noConfig: 'No configuration',
        apiService: 'API service',
        mcpPlugin: 'MCP plugin',
        llmProvider: 'LLM provider',
        operations: 'Operations',
        observability: 'Observability',
        security: 'Security',
        'cost-control': 'Efficiency',
        auth: 'Auth',
        rate: 'Rate',
        tokens: 'Tokens',
        apiUsd: 'API USD',
        chatgptCredits: 'OpenAI credits',
        usage: 'Usage',
        rateCard: 'Rate card',
        model: 'Model',
        tier: 'Tier',
        context: 'Context',
        requests: 'Requests',
        prompt: 'Prompt',
        cached: 'Cached',
        completion: 'Completion',
        complete: 'Complete',
        inputUsd: 'API in /1M',
        cachedUsd: 'API cached /1M',
        outputUsd: 'API out /1M',
        inputCredits: 'Credits in /1M',
        cachedCredits: 'Credits cached /1M',
        outputCredits: 'Credits out /1M',
        flame: 'Daily tokens',
        noLocalUsage: 'No local usage',
        localCli: 'Local CLI',
        localInput: 'Local input',
        localCache: 'Local cache read',
        localCacheWrite: 'Local cache write',
        localOutput: 'Local output',
        localTotal: 'Local total',
        source: 'Source',
        cacheWrite: 'Cache write',
        vendor: 'Vendor',
        localApiUsd: 'Local API USD',
        localFx: 'Local FX',
        fxCountry: 'Country FX',
        costFilters: 'Filters',
        billing: 'Billing',
        subscription: 'Subscription',
        allModels: 'All models',
        unpublished: 'unpublished',
        family: 'Family',
        llm: 'LLM',
        llmTrafficMgmt: 'LLM',
        configSyncHealthy: 'Config Sync: Healthy',
        last24h: 'Last 24h',
        successRate: 'Success Rate',
        p95Latency: 'P95 Latency',
        tokenUsage: 'Token Usage',
        estimatedCost: 'Estimated Cost',
        client: 'CLIENT',
        gateway: 'GATEWAY',
        policy: 'POLICY',
        modelProvider: 'MODEL PROVIDER',
        fallbackChain: 'FALLBACK CHAIN',
        routingStrategy: 'Routing Strategy',
        fallbackOrder: 'Fallback Order',
        onErrorOrTimeout: 'On error or timeout',
        requestDetail: 'REQUEST DETAIL',
        selection: 'SELECTION',
        latencyBreakdown: 'LATENCY BREAKDOWN (ms)',
        retryTimeout: 'RETRY / TIMEOUT',
        policyResult: 'POLICY RESULT',
        result: 'RESULT',
        viewInTraces: 'View in Traces',
        models: 'MODELS',
        status: 'Status',
        weight: 'Weight',
        healthy: 'Healthy',
        degraded: 'Degraded',
        throttled: 'Throttled',
        rpm: 'RPM',
        tpm: 'TPM',
        errorRate: 'Error Rate',
        costUsd: 'Cost (USD)',
        routingPolicySummary: 'ROUTING POLICY SUMMARY',
        alerts: 'ALERTS',
        legend: 'LEGEND',
        legPrimary: 'Primary Route (Selected)',
        legWeighted: 'Weighted Route',
        legDegraded: 'Degraded',
        legFallbackPath: 'Fallback Path',
        legFallbackChain: 'Fallback Chain',
        viewAllAlerts: 'View all alerts',
        vsPrev24hReq: 'vs prev 24h 1.05M',
        vsPrev24hSucc: 'vs prev 24h 98.81%',
        vsPrev24hLat: 'vs prev 24h 1.62s',
        vsPrev24hTok: 'vs prev 24h 2.25B',
        vsPrev24hCost: 'vs prev 24h $3,550.42',
        mcpMgmt: 'MCP Management',
        mcpSubtitle: 'Manage MCP servers, tools, and secure agent access',
        mcpServers: 'MCP Servers',
        mcpServer: 'MCP Server',
        invocations: 'Invocations',
        policyDenials: 'Policy Denials',
        credDelegations: 'Credential Delegations',
        highRisk: 'High Risk',
        medRisk: 'Medium',
        lowRisk: 'Low',
        server: 'Server',
        transport: 'Transport',
        lastSeen: 'Last Seen',
        actions: 'Actions',
        recentInvocations: 'Recent Invocations',
        viewAllInvocations: 'View all invocations →',
        caller: 'Caller',
        toolInventory: 'Tool Inventory',
        allowedCallers: 'Allowed Callers',
        toolPolicy: 'Tool Policy',
        allowlist: 'Allowlist',
        denylist: 'Denylist',
        paramValidation: 'Parameter Validation',
        credDelegation: 'Credential Delegation',
        lastInvocation: 'Last Invocation',
        callerIdentity: 'Caller Identity',
        delegatedIdentity: 'Delegated Identity',
        arguments: 'Arguments',
        policyDecision: 'Policy Decision',
        duration: 'Duration',
        resultStatus: 'Result Status',
        error: 'Error',
        allStatuses: 'All Statuses',
        timeRange: 'Time',
        last1h: 'Last 1 hour',
        last7d: 'Last 7 days',
        overviewTab: 'Overview',
        toolsTab: 'Tools',
        policyTab: 'Policy',
        authTab: 'Auth',
        credentialsTab: 'Credentials',
        metricsTab: 'Metrics',
        tool: 'Tool',
        success: 'Success',
        errors: 'Errors',
        time: 'Time',
        a2aOps: 'A2A Operations',
        a2aSubtitle: 'Monitor and operate agent-to-agent communications',
        totalA2aReq: 'Total A2A Requests',
        activeTasks: 'Active Tasks',
        completedTasks: 'Completed Tasks',
        agentRegistry: 'Agent Registry',
        viewAllAgents: 'View all agents →',
        a2aTasks: 'A2A Tasks',
        taskDetail: 'Task Detail',
        taskSummary: 'Task Summary',
        sourceAgent: 'Source Agent',
        sourceIdentity: 'Source Identity',
        representedIdentity: 'Represented Identity',
        targetAgent: 'Target Agent',
        capability: 'Capability',
        retries: 'Retries',
        timeline: 'Timeline',
        identities: 'Identities',
        delegation: 'Delegation',
        metadata: 'Metadata',
        payload: 'Payload',
        observabilityTitle: 'Observability',
        obsSubtitle: 'Trace, metrics, and logs for requests across the gateway',
        distributedTrace: 'Distributed Trace',
        recentTraces: 'Recent Traces',
        refresh: 'Refresh',
        securityControl: 'Security Control',
        secSubtitle: 'Monitor security posture, policies, and risk events across the gateway',
        blockedRequests: 'Blocked Requests',
        authFailures: 'Auth Failures',
        policyDenials: 'Policy Denials',
        highRiskToolCalls: 'High-risk Tool Calls',
        credExposureEvents: 'Credential Exposure Events',
        topRiskTypes: 'Top Risk Types',
        securityDecisionChain: 'Security Decision Chain',
        securityEvents: 'Security Events',
        'cost-control': 'Efficiency',
        costControlTitle: 'Efficiency',
        costSubtitle: 'Real-time spend, token & prompt cache accounting, tool efficiency, and optimization evidence',
        costBreakdown: 'Cost Breakdown',
        totalSpend: 'Total Spend (Actual)',
        costPerReq: 'Cost per Request',
        tokenCost: 'Token Cost',
        mcpCost: 'MCP / API Cost',
        networkCost: 'Network Cost',
        projectedSpend: 'Projected Spend (7d)',
        budgetStatus: 'Budget & Quota Status'
      },
      zh: {
        overview: '总览',
        core: 'Core',
        base: 'Core',
        ai: 'AI',
        setup: 'Setup',
        configuration: '配置',
        mcp: 'MCP',
        a2a: 'A2A',
        resources: '资源',
        services: 'APIs',
        search: '搜索',
        themeLight: '亮色',
        themeDark: '暗色',
        apiServices: 'API 服务',
        mcpPlugins: 'MCP 插件',
        agents: '智能体',
        llmProviders: 'LLM 供应商',
        overviewSubtitle: '监控网关配置、网关资产与实时运行状态',
        viewServices: '查看 APIs',
        manageMcp: '管理 MCP',
        agentRegistryLink: '智能体注册表',
        llmRouting: '模型路由',
        protocolDistribution: '协议分布',
        protocolsInUse: '正在使用的监听与后端协议',
        policyOverview: '策略总览',
        policyOverviewSub: '安全、流量及预算配额策略',
        quickActions: '快捷操作',
        quickActionsSub: '常用网关管理与调试动作',
        addService: '注册服务 / 路由',
        manageMcpServers: '管理 MCP 服务器',
        configurePolicy: '配置策略',
        testPrompt: '测试提示词 (演练场)',
        budgetQuota: '预算与配额',
        traffic: '流量策略',
        totalPolicies: '策略总数',
        apiServicesSub: '上游网关集群与 HTTP 路由',
        mcpPluginsSub: '工具服务器与 SSE 传输端点',
        agentsSub: 'A2A 协议与多智能体协作网络',
        llmProvidersSub: '多模型智能路由与容灾策略',
        registeredServices: '接入服务数',
        upstreamClusters: '上游集群',
        routingRules: '路由规则',
        mcpServers: '插件服务器',
        exposedTools: '开放工具数',
        sseTransport: '传输模式',
        registeredAgents: '注册智能体',
        commPaths: 'A2A 协同链路',
        agentProtocol: '传输协议',
        managedModels: '纳管模型数',
        upstreamProviders: '模型供应商',
        failoverStatus: '故障转移',
        runtimeObservability: '运行与遥测',
        runtimeObservabilitySub: '网关健康状态与核心性能指标',
        gatewayStatus: '网关状态',
        p99Latency: 'P99 延迟',
        telemetrySpans: '遥测跨度速率',
        viewObservability: '查看可观测看板',
        type: '类型',
        name: '名称',
        detail: '详情',
        endpoints: '端点',
        domain: '域名',
        routes: '路由',
        tls: 'TLS',
        mtls: 'mTLS',
        plaintext: '明文',
        secLevel: '安全',
        mode: '模式',
        protocol: '协议',
        pickService: '选择服务',
        unhealthy: '不健康',
        kind: '类型',
        provider: '供应商',
        id: 'ID',
        baseUrl: '基址',
        endpoint: '端点',
        tools: '工具',
        agent: '智能体',
        noConfig: '没有配置',
        apiService: 'API 服务',
        mcpPlugin: 'MCP 插件',
        llmProvider: 'LLM 供应商',
        operations: '运维',
        observability: '可观测性',
        security: '安全',
        'cost-control': '效率',
        auth: '鉴权',
        rate: '限流',
        tokens: 'Token',
        apiUsd: 'API 美元',
        chatgptCredits: 'OpenAI credits',
        usage: '用量',
        rateCard: '价目表',
        model: '模型',
        tier: '档位',
        context: '上下文',
        requests: '请求量',
        prompt: '输入',
        cached: '缓存',
        completion: '输出',
        complete: '完整',
        inputUsd: 'API 输入 /1M',
        cachedUsd: 'API 缓存 /1M',
        outputUsd: 'API 输出 /1M',
        inputCredits: 'Credits 输入 /1M',
        cachedCredits: 'Credits 输出 /1M',
        outputCredits: 'Credits 输出 /1M',
        flame: '每日 token',
        noLocalUsage: '没有本机用量',
        localCli: '本机 CLI',
        localInput: '本机输入',
        localCache: '本机缓存读',
        localCacheWrite: '本机缓存写',
        localOutput: '本机输出',
        localTotal: '本机合计',
        source: '来源',
        cacheWrite: '缓存写',
        vendor: '厂商',
        localApiUsd: '本机 API 美元',
        localFx: '本机外汇',
        fxCountry: '国家汇率',
        costFilters: '筛选',
        billing: '计费',
        subscription: '订阅',
        allModels: '全部模型',
        unpublished: '未公布',
        family: '分类',
        llm: 'LLM',
        llmTrafficMgmt: 'LLM',
        configSyncHealthy: '配置同步: 正常',
        last24h: '最近 24 小时',
        successRate: '成功率',
        p95Latency: 'P95 延迟',
        tokenUsage: 'Token 用量',
        estimatedCost: '预估成本',
        client: '客户端',
        gateway: '网关',
        policy: '策略',
        modelProvider: '模型供应商',
        fallbackChain: '降级回退链',
        routingStrategy: '路由策略',
        fallbackOrder: '降级顺序',
        onErrorOrTimeout: '错误或超时时',
        requestDetail: '请求详情',
        selection: '模型选择',
        latencyBreakdown: '延迟分解 (毫秒)',
        retryTimeout: '重试与超时',
        policyResult: '策略结果',
        result: '执行结果',
        viewInTraces: '在 Traces 中查看',
        models: '模型列表',
        status: '状态',
        weight: '权重',
        healthy: '健康',
        degraded: '降级',
        throttled: '限流',
        rpm: '每分请求',
        tpm: '每分 Token',
        errorRate: '错误率',
        costUsd: '成本 (美元)',
        routingPolicySummary: '路由策略摘要',
        alerts: '告警',
        legend: '图例',
        legPrimary: '主选路由 (当前匹配)',
        legWeighted: '加权路由',
        legDegraded: '性能降级',
        legFallbackPath: '降级路径',
        legFallbackChain: '降级链',
        viewAllAlerts: '查看全部告警',
        vsPrev24hReq: '较前 24h 1.05M',
        vsPrev24hSucc: '较前 24h 98.81%',
        vsPrev24hLat: '较前 24h 1.62s',
        vsPrev24hTok: '较前 24h 2.25B',
        vsPrev24hCost: '较前 24h $3,550.42',
        mcpMgmt: 'MCP 服务治理',
        mcpSubtitle: '管理 MCP 服务端、工具以及安全的 Agent 访问',
        mcpServers: 'MCP 服务端',
        mcpServer: 'MCP 服务端',
        invocations: '调用量',
        policyDenials: '策略拦截',
        credDelegations: '凭证委托',
        highRisk: '高风险',
        medRisk: '中风险',
        lowRisk: '低风险',
        server: '服务端',
        transport: '传输协议',
        lastSeen: '最近活跃',
        actions: '操作',
        recentInvocations: '近期调用记录',
        viewAllInvocations: '查看全部调用记录 →',
        caller: '调用方',
        toolInventory: '工具清单',
        allowedCallers: '授权调用方',
        toolPolicy: '工具策略',
        allowlist: '白名单',
        denylist: '黑名单',
        paramValidation: '参数校验',
        credDelegation: '凭证委托',
        lastInvocation: '最近一次调用详情',
        callerIdentity: '调用方身份',
        delegatedIdentity: '委托身份',
        arguments: '调用参数',
        policyDecision: '策略判定',
        duration: '耗时',
        resultStatus: '执行状态',
        error: '错误信息',
        allStatuses: '全部状态',
        timeRange: '时间范围',
        last1h: '最近 1 小时',
        last7d: '最近 7 天',
        overviewTab: '总览',
        toolsTab: '工具',
        policyTab: '策略',
        authTab: '认证',
        credentialsTab: '凭证',
        metricsTab: '指标',
        tool: '工具',
        success: '成功',
        errors: '错误',
        time: '时间',
        a2aOps: 'A2A 智能体协同',
        a2aSubtitle: '监控与治理智能体间（Agent-to-Agent）跨域通信与协同',
        totalA2aReq: 'A2A 请求总量',
        activeTasks: '活跃任务',
        completedTasks: '已完成任务',
        agentRegistry: '智能体注册表',
        viewAllAgents: '查看全部智能体 →',
        a2aTasks: 'A2A 协同任务',
        taskDetail: '任务详情',
        taskSummary: '任务摘要',
        sourceAgent: '发起智能体',
        sourceIdentity: '发起方身份',
        representedIdentity: '代表用户身份',
        targetAgent: '目标智能体',
        capability: '能力方法',
        retries: '重试次数',
        timeline: '时序轨迹',
        identities: '身份认证',
        delegation: '凭证委托',
        metadata: '元数据',
        payload: '报文负载',
        observabilityTitle: '可观测性治理',
        obsSubtitle: '全网关请求链路追踪、指标监控与日志洞察',
        distributedTrace: '全链路分布式追踪',
        recentTraces: '近期调用链路',
        refresh: '刷新',
        securityControl: '安全防控治理',
        secSubtitle: '全网关安全态势感知、策略执行与风险事件监控',
        blockedRequests: '已拦截请求',
        authFailures: '认证失败数',
        policyDenials: '策略阻断数',
        highRiskToolCalls: '高危工具调用',
        credExposureEvents: '凭证暴露事件',
        topRiskTypes: '高危风险分布',
        securityDecisionChain: '安全决策链路',
        securityEvents: '安全风险事件',
        'cost-control': '效率',
        costControlTitle: '效率',
        costSubtitle: '实时费用、Token与缓存账本、执行效率与优化节省证据',
        costBreakdown: '成本分摊明细',
        totalSpend: '总支出 (实际)',
        costPerReq: '单次请求均成本',
        tokenCost: 'Token 消耗成本',
        mcpCost: 'MCP / API 成本',
        networkCost: '网络传输成本',
        projectedSpend: '预估 7 日支出',
        budgetStatus: '预算与配额状态'
      }
    };

    const $ = (id) => document.getElementById(id);
    const state = {
      lang: document.documentElement.lang === 'zh' ? 'zh' : 'en',
      theme: document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light',
      query: '',
      tab: 'overview',
      queries: {},
      showExamples: false,
      paused: false,
      loading: false,
      endpointErrors: {},
      syncedAt: {},
      config: { clusters: [], backends: [], providers: [] },
      cost: { rate_card: [], usage: [], api_usd: '$0', chatgpt_credits: '0' },
      costLedgerTab: 'spend',
      costRoute: 'all-routes',
      costModelFilter: 'all-models',
      costSelectedEventId: null,
      country: localStorage.getItem('transit-country') || 'United States',
      model: localStorage.getItem('transit-model') || 'all',
      billing: localStorage.getItem('transit-billing') || 'subscription',
      family: localStorage.getItem('transit-family') || 'chatgpt'
    };

    function t(key) {
      return I18N[state.lang][key] || I18N.en[key] || key;
    }

    function backendsOf(config, type) {
      return (config.backends || []).filter((backend) => backend.type === type);
    }

    function esc(value) {
      return String(value ?? '').replace(/[&<>"']/g, (ch) => ({ '&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;', "'":'&#39;' }[ch]));
    }

    function uiText(en, zh) { return state.lang === 'zh' ? zh : en; }

    function notify(message) {
      let toast = $('ui-toast');
      if (!toast) {
        toast = document.createElement('div');
        toast.id = 'ui-toast';
        toast.className = 'ui-toast';
        toast.setAttribute('role', 'status');
        toast.innerHTML = '<span></span><button type="button" aria-label="Dismiss">×</button>';
        toast.querySelector('button').onclick = () => toast.classList.add('hidden');
        document.body.appendChild(toast);
      }
      toast.querySelector('span').textContent = message;
      toast.classList.remove('hidden');
      clearTimeout(state.toastTimer);
      state.toastTimer = setTimeout(() => toast.classList.add('hidden'), 7000);
    }

    function unavailableAction() {
      notify(uiText('This control is not connected yet. No request was sent and no gateway configuration was changed.', '此操作尚未接入后端，未发送请求，也未修改网关配置。'));
    }

    async function copyText(value) {
      try {
        await navigator.clipboard.writeText(value);
        notify(uiText('Copied to clipboard.', '已复制到剪贴板。'));
      } catch {
        notify(uiText('Clipboard access is unavailable. Select and copy the displayed text.', '无法访问剪贴板，请选中显示的文本手动复制。'));
      }
    }

    function emptyRow(columns, message) {
      return '<tr><td colspan="' + columns + '" class="ui-empty">' + esc(message) + '</td></tr>';
    }

    function updatePageChrome() {
      const descriptions = {
        overview: ['Gateway resources at a glance. Select a module to inspect its configuration.', '查看网关资源概况，进入各模块检查配置与运行情况。'],
        services: ['', ''],
        llm: ['', ''],
        mcp: ['', ''],
        a2a: ['', ''],
        observability: ['Filter gateway requests and inspect the available decision evidence.', '筛选网关请求，查看可用的决策记录与关联信息。'],
        security: ['Audit decisions first. Expand policies and identities when needed.', '优先查看安全决策，需要时展开策略与身份目录。'],
        'cost-control': ['Inspect spend, token usage and execution efficiency in separate ledgers.', '分别查看费用、Token 消耗与执行效率账本。'],
        configuration: ['', '']
      };
      const pair = descriptions[state.tab] || descriptions.overview;
      $('page-description').textContent = uiText(...pair);
      if (state.tab !== 'llm' && $('page-actions')) {
        $('page-actions').innerHTML = '';
      }
      const aiPage = ['mcp', 'a2a'].includes(state.tab);
      if ($('examples-toggle')) {
        $('examples-toggle').classList.toggle('hidden', !aiPage);
        $('examples-toggle').textContent = uiText('Example data', '示例数据');
        $('examples-toggle').setAttribute('aria-pressed', String(state.showExamples));
      }
      if ($('poll-toggle')) {
        $('poll-toggle').textContent = state.paused ? uiText('Resume updates', '恢复更新') : uiText('Pause updates', '暂停更新');
        $('poll-toggle').setAttribute('aria-pressed', String(state.paused));
      }
      if ($('reload-data')) {
        $('reload-data').textContent = uiText('Sync now', '立即同步');
        $('reload-data').disabled = state.loading;
      }
      const scopes = { llm: ['/debug/llm'], services: ['/debug/services'], observability: ['/debug/observability'], security: ['/debug/security/posture', '/debug/security/events', '/debug/security/identities'], 'cost-control': ['/debug/cost'] };
      const relevant = ['/debug/config', ...(scopes[state.tab] || [])];
      const failures = relevant.filter(url => state.endpointErrors[url]).map(url => state.endpointErrors[url]);
      const syncText = state.loading ? uiText('Syncing…', '同步中…') : failures.length ? uiText('Update incomplete', '更新不完整') : state.paused ? uiText('Updates paused', '自动更新已暂停') : state.syncedAt[state.tab] ? uiText('Synced ', '已同步 ') + state.syncedAt[state.tab] : uiText('Awaiting data', '等待数据');
      if ($('sync-state')) $('sync-state').textContent = syncText;
      $('error').textContent = failures.length ? uiText('Some data could not be updated. Retaining the last available values. ', '部分数据更新失败，保留上次可用结果。') + failures.join(' · ') : '';
      $('error').classList.toggle('hidden', !failures.length);
      const notice = $('data-notice');
      notice.classList.toggle('hidden', state.tab === 'mcp' || (!aiPage && state.tab !== 'observability'));
      notice.dataset.kind = 'config';
      notice.textContent = aiPage
        ? uiText('Configuration view · Unreported metrics are shown as “—”. Runtime controls are not connected.', '配置视图 · 未上报的指标显示为「—」，运行控制尚未接入。')
        : uiText('Decision-derived view · The displayed spans are estimated from security decisions, not measured distributed traces.', '决策记录视图 · 当前 Span 由安全决策耗时估算，并非实测的分布式追踪。');
    }

    function applyPageFilter() {
      const page = $('tab-' + state.tab);
      if (!page) return;
      const selector = state.tab === 'llm' ? '.llm-account-card' : state.tab === 'overview' ? '.ov-kpi-card' : '.table-wrap tbody tr';
      const items = Array.from(page.querySelectorAll(selector)).filter(el =>
        !el.closest('.svc-drawer, .obs-trace-drawer, .sec-drawer, [style*="display:none"], [style*="display: none"]') && !el.querySelector('.ui-empty'));
      const q = state.query.trim().toLowerCase();
      let matches = 0;
      items.forEach(el => {
        const matched = !q || (el.textContent + ' ' + (el.dataset.search || '')).toLowerCase().includes(q);
        el.classList.toggle('ui-search-hidden', !matched);
        if (matched) matches++;
      });
      $('page-search-empty').classList.toggle('hidden', !q || matches > 0);
      $('page-search-empty').textContent = q && !matches ? uiText('No matches on this page. Clear the search or adjust the filters.', '当前页面没有匹配项，请清空搜索或调整筛选条件。') : '';
      page.querySelectorAll('.table-wrap').forEach(el => {
        el.tabIndex = 0;
        el.setAttribute('role', 'region');
        el.setAttribute('aria-label', uiText('Scrollable data table', '可滚动数据表格'));
      });
      page.querySelectorAll('tbody tr[onclick], tr[data-mcp-server], tr[data-a2a-agent], tr[data-svc-id], tr[data-trace-idx], tr[data-decision-idx]').forEach(el => {
        el.tabIndex = 0;
        el.setAttribute('aria-label', uiText('Inspect ', '查看 ') + (el.querySelector('td')?.textContent || ''));
      });
    }

    function setFilterOptions(id, values, current, label) {
      const select = $(id);
      const options = [...new Set(values)];
      const selected = options.includes(current) ? current : 'all';
      const key = JSON.stringify([options, state.lang]);
      if (select.dataset.options !== key) {
        select.innerHTML = '<option value="all">' + esc(label) + '</option>' + options.map(value => '<option value="' + esc(value) + '">' + esc(value === 'unknown' || value === '—' ? uiText('Not reported', '未上报') : value) + '</option>').join('');
        select.dataset.options = key;
      }
      select.value = selected;
      return selected;
    }

    window.accountRuntimeStates = window.accountRuntimeStates || {};
    window.mcpServerRuntimeStates = window.mcpServerRuntimeStates || {};
    window.mcpToolRuntimeStates = window.mcpToolRuntimeStates || {};
    window.mcpCatalogRiskFilter = 'all';

    window.handleMcpServerAction = function() {
      unavailableAction();
    };

    // MCP Gateway Runtime Controller
    window.mcpCapActiveTab = window.mcpCapActiveTab || 'tools';
    window.mcpSelectedTransport = window.mcpSelectedTransport || 'streamable-http';
    window.mcpEditingServer = null;

    window.openMcpAddModal = function() {
      const dlg = $('mcp-add-dialog');
      if (!dlg) return;
      window.mcpEditingServer = null;
      if ($('mcp-dialog-title')) $('mcp-dialog-title').textContent = uiText('Add MCP server', '添加 MCP 服务');
      if ($('mcp-input-name')) {
        $('mcp-input-name').value = '';
        $('mcp-input-name').disabled = false;
      }
      if ($('mcp-input-endpoint')) $('mcp-input-endpoint').value = '';
      selectMcpTransport('streamable-http');
      dlg.classList.remove('hidden');
    };

    window.closeMcpAddModal = function() {
      const dlg = $('mcp-add-dialog');
      if (dlg) dlg.classList.add('hidden');
    };

    window.openMcpEditModal = function(name) {
      const dlg = $('mcp-add-dialog');
      if (!dlg) return;
      const config = state.config || {};
      const mcpBackends = backendsOf(config, 'mcp');
      const all = [...mcpBackends, ...(state.localMcpServers || [])];
      const target = all.find(s => s.name === name);
      if (!target) return;
      window.mcpEditingServer = name;
      if ($('mcp-dialog-title')) $('mcp-dialog-title').textContent = uiText('Edit MCP server', '编辑 MCP 服务');
      if ($('mcp-input-name')) {
        $('mcp-input-name').value = target.name;
        $('mcp-input-name').disabled = true;
      }
      if ($('mcp-input-endpoint')) $('mcp-input-endpoint').value = target.endpoint || '';
      let tKey = 'streamable-http';
      if (target.transport === 'Command Line' || target.endpoint?.startsWith('cmd:') || target.command) tKey = 'cli';
      else if (target.transport === 'Legacy SSE' || target.endpoint?.includes('/sse') || target.protocol === 'sse') tKey = 'legacy-sse';
      selectMcpTransport(tKey);
      dlg.classList.remove('hidden');
    };

    window.selectMcpTransport = function(type) {
      window.mcpSelectedTransport = type;
      document.querySelectorAll('#mcp-add-dialog .mcp-seg-btn').forEach(b => {
        b.classList.toggle('active', b.dataset.transport === type);
      });
      const label = $('mcp-label-endpoint');
      const input = $('mcp-input-endpoint');
      const hint = $('mcp-hint-endpoint');
      if (type === 'cli') {
        if (label) label.textContent = 'COMMAND';
        if (input) input.placeholder = 'npx -y @modelcontextprotocol/server-filesystem /path';
        if (hint) hint.textContent = 'The command to spawn the MCP server process';
      } else if (type === 'legacy-sse') {
        if (label) label.textContent = 'URL (SSE)';
        if (input) input.placeholder = 'http://localhost:8080/sse';
        if (hint) hint.textContent = 'The SSE endpoint URL of the legacy MCP server';
      } else {
        if (label) label.textContent = 'URL';
        if (input) input.placeholder = 'http://localhost:8080/mcp';
        if (hint) hint.textContent = 'The endpoint URL of the MCP server';
      }
    };

    window.handleSaveMcpServer = function(e) {
      if (e) e.preventDefault();
      const name = ($('mcp-input-name')?.value || '').trim();
      const endpoint = ($('mcp-input-endpoint')?.value || '').trim();
      const transport = window.mcpSelectedTransport || 'streamable-http';
      if (!name || !endpoint) return;
      state.localMcpServers = state.localMcpServers || [];
      const transportLabel = transport === 'cli' ? 'Command Line' : (transport === 'legacy-sse' ? 'Legacy SSE' : 'Streamable HTTP');
      if (window.mcpEditingServer) {
        const existing = state.localMcpServers.find(s => s.name === window.mcpEditingServer);
        if (existing) {
          existing.endpoint = endpoint;
          existing.transport = transportLabel;
        } else {
          state.localMcpServers.push({ name, endpoint, transport: transportLabel, tools: [] });
        }
      } else {
        const existing = state.localMcpServers.find(s => s.name === name);
        if (existing) {
          existing.endpoint = endpoint;
          existing.transport = transportLabel;
        } else {
          state.localMcpServers.push({ name, endpoint, transport: transportLabel, tools: [] });
        }
      }
      state.mcpSelectedServer = name;
      closeMcpAddModal();
      renderMcp();
    };

    window.handleMcpDeleteServer = function(name) {
      if (!name) return;
      if (state.localMcpServers) {
        state.localMcpServers = state.localMcpServers.filter(s => s.name !== name);
      }
      state.deletedMcpServers = state.deletedMcpServers || new Set();
      state.deletedMcpServers.add(name);
      if (state.mcpSelectedServer === name) {
        state.mcpSelectedServer = '';
      }
      renderMcp();
    };

    window.handleMcpRemoveTarget = function() {
      if (!state.mcpSelectedServer) return;
      if (state.localMcpServers) {
        state.localMcpServers = state.localMcpServers.filter(s => s.name !== state.mcpSelectedServer);
      }
      state.mcpSelectedServer = '';
      renderMcp();
    };

    window.handleMcpHandshakeTest = function(serverName) {
      unavailableAction();
    };

    window.filterMcpCapTab = function(tab, btn) {
      window.mcpCapActiveTab = tab;
      document.querySelectorAll('#mcp-capabilities-panel .tool-filter-btn').forEach((b) => b.classList.toggle('active', b === btn));
      renderMcp();
    };

    window.selectMcpServer = function(name) {
      state.mcpSelectedServer = name;
      renderMcp();
    };

    window.selectMcpInvocation = function(idx) {
      state.mcpSelectedInvIdx = idx;
      renderMcp();
    };

    // A2A Global Runtime Controller
    window.a2aAgentRuntimeStates = window.a2aAgentRuntimeStates || {};
    window.a2aRoleFilter = window.a2aRoleFilter || 'all';
    window.a2aStatusFilter = window.a2aStatusFilter || 'all';
    window.a2aTeamFilter = window.a2aTeamFilter || 'all';
    window.a2aSearchQuery = window.a2aSearchQuery || '';
    window.a2aTaskTab = window.a2aTaskTab || 'all';
    window.a2aSubtab = window.a2aSubtab || 'gateways';
    window.a2aEditingGateway = null;
    window.a2aEditingRoute = null;

    window.switchA2aSubtab = function(subtab) {
      window.a2aSubtab = subtab;
      const isGateways = subtab === 'gateways';
      const btnGw = $('a2a-tab-btn-gateways');
      const btnRt = $('a2a-tab-btn-routes');
      const viewGw = $('a2a-view-gateways');
      const viewRt = $('a2a-view-routes');
      const actionBtn = $('a2a-main-action-btn');

      if (btnGw) {
        btnGw.classList.toggle('active', isGateways);
        btnGw.setAttribute('aria-selected', isGateways ? 'true' : 'false');
      }
      if (btnRt) {
        btnRt.classList.toggle('active', !isGateways);
        btnRt.setAttribute('aria-selected', !isGateways ? 'true' : 'false');
      }
      if (viewGw) viewGw.classList.toggle('hidden', !isGateways);
      if (viewRt) viewRt.classList.toggle('hidden', isGateways);

      if (actionBtn) {
        if (isGateways) {
          actionBtn.onclick = openA2aAddGatewayModal;
          actionBtn.innerHTML = '<svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;"><path d="M8 3v10M3 8h10"/></svg><span>Add gateway</span>';
        } else {
          actionBtn.onclick = openA2aAddRouteModal;
          actionBtn.innerHTML = '<svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" style="vertical-align:middle;"><path d="M8 3v10M3 8h10"/></svg><span>Add route</span>';
        }
      }
    };

    // Gateway Modal
    window.openA2aAddGatewayModal = function() {
      window.a2aEditingGateway = null;
      const dlg = $('a2a-add-gateway-dialog');
      if (!dlg) return;
      if ($('a2a-gw-modal-title')) $('a2a-gw-modal-title').textContent = uiText('Add gateway', '添加网关');
      if ($('a2a-gw-name')) {
        $('a2a-gw-name').value = '';
        $('a2a-gw-name').disabled = false;
      }
      if ($('a2a-gw-port')) $('a2a-gw-port').value = '26080';
      if ($('a2a-gw-default')) $('a2a-gw-default').checked = true;
      if ($('a2a-gw-protocol')) $('a2a-gw-protocol').value = 'http';
      if ($('a2a-gw-multiple-listeners')) $('a2a-gw-multiple-listeners').checked = false;
      updateA2aGatewayYaml();
      dlg.classList.remove('hidden');
    };

    window.closeA2aAddGatewayModal = function() {
      const dlg = $('a2a-add-gateway-dialog');
      if (dlg) dlg.classList.add('hidden');
    };

    window.openA2aEditGatewayModal = function(name) {
      window.a2aEditingGateway = name;
      const dlg = $('a2a-add-gateway-dialog');
      if (!dlg) return;
      const gwList = state.customGateways || [];
      const gw = gwList.find(g => g.name === name);
      if (!gw) return;
      if ($('a2a-gw-modal-title')) $('a2a-gw-modal-title').textContent = uiText('Edit gateway', '编辑网关');
      if ($('a2a-gw-name')) {
        $('a2a-gw-name').value = gw.name;
        $('a2a-gw-name').disabled = true;
      }
      if ($('a2a-gw-port')) $('a2a-gw-port').value = gw.port;
      if ($('a2a-gw-default')) $('a2a-gw-default').checked = !!gw.isDefault;
      if ($('a2a-gw-protocol')) $('a2a-gw-protocol').value = gw.protocol || 'http';
      if ($('a2a-gw-multiple-listeners')) $('a2a-gw-multiple-listeners').checked = !!gw.multipleListeners;
      updateA2aGatewayYaml();
      dlg.classList.remove('hidden');
    };

    window.handleA2aSaveGateway = function(e) {
      if (e) e.preventDefault();
      const name = ($('a2a-gw-name')?.value || '').trim();
      const port = parseInt($('a2a-gw-port')?.value || '26080', 10);
      const isDefault = !!$('a2a-gw-default')?.checked;
      const protocol = $('a2a-gw-protocol')?.value || 'http';
      const multipleListeners = !!$('a2a-gw-multiple-listeners')?.checked;
      if (!name || isNaN(port)) return;

      state.customGateways = state.customGateways || [];
      if (isDefault) {
        state.customGateways.forEach(g => { g.isDefault = false; });
      }
      if (window.a2aEditingGateway) {
        const target = state.customGateways.find(g => g.name === window.a2aEditingGateway);
        if (target) {
          target.port = port;
          target.isDefault = isDefault;
          target.protocol = protocol;
          target.multipleListeners = multipleListeners;
        }
      } else {
        const existing = state.customGateways.find(g => g.name === name);
        if (existing) {
          existing.port = port;
          existing.isDefault = isDefault;
          existing.protocol = protocol;
          existing.multipleListeners = multipleListeners;
        } else {
          state.customGateways.push({ name, port, isDefault, protocol, multipleListeners, policies: 0 });
        }
      }
      closeA2aAddGatewayModal();
      renderA2a();
    };

    window.handleA2aDeleteGateway = function(name) {
      if (!name) return;
      if (state.customGateways) {
        state.customGateways = state.customGateways.filter(g => g.name !== name);
      }
      renderA2a();
    };

    window.updateA2aGatewayYaml = function() {
      const name = ($('a2a-gw-name')?.value || 'default').trim();
      const port = parseInt($('a2a-gw-port')?.value || '26080', 10);
      const isDefault = $('a2a-gw-default')?.checked;
      const protocol = $('a2a-gw-protocol')?.value || 'http';
      const multi = $('a2a-gw-multiple-listeners')?.checked;

      let yaml = 'gateways:\n  - name: ' + name + '\n    port: ' + port + '\n    protocol: ' + protocol;
      if (isDefault) yaml += '\n    default: true';
      if (multi) yaml += '\n    multiple_listeners: true';

      const preview = $('a2a-gw-yaml-preview');
      if (preview) preview.textContent = yaml;
    };

    window.toggleA2aGwYaml = function(el) {
      const preview = $('a2a-gw-yaml-preview');
      const arrow = $('a2a-gw-yaml-arrow');
      if (preview) {
        const isHidden = preview.classList.toggle('hidden');
        if (arrow) arrow.textContent = isHidden ? '▶' : '▼';
      }
    };

    // Route Modal
    window.openA2aAddRouteModal = function() {
      window.a2aEditingRoute = null;
      const dlg = $('a2a-add-route-dialog');
      if (!dlg) return;
      if ($('a2a-route-modal-title')) $('a2a-route-modal-title').textContent = uiText('Add route', '添加路由');
      if ($('a2a-route-name')) {
        $('a2a-route-name').value = '';
        $('a2a-route-name').disabled = false;
      }
      if ($('a2a-route-kind')) $('a2a-route-kind').value = 'a2a';
      const gwSelect = $('a2a-route-gateway');
      if (gwSelect) {
        const gws = state.customGateways || [{ name: 'default' }];
        gwSelect.innerHTML = gws.map(g => '<option value="' + esc(g.name) + '">' + esc(g.name) + '</option>').join('');
      }
      if ($('a2a-route-hostnames')) $('a2a-route-hostnames').value = '';
      if ($('a2a-route-path-type')) $('a2a-route-path-type').value = 'prefix';
      if ($('a2a-route-path-value')) $('a2a-route-path-value').value = '/a2a';
      if ($('a2a-route-method')) $('a2a-route-method').value = '';

      const hContainer = $('a2a-route-headers-container');
      if (hContainer) hContainer.innerHTML = '<div class="a2a-box-empty">No header matches configured. Click "+ Add header match" to add one.</div>';
      const qContainer = $('a2a-route-queries-container');
      if (qContainer) qContainer.innerHTML = '<div class="a2a-box-empty">No query matches configured. Click "+ Add query match" to add one.</div>';

      const bContainer = $('a2a-route-backends-container');
      if (bContainer) {
        bContainer.innerHTML = '';
        addA2aRouteBackendRow('planner', 100);
      }

      updateA2aRouteYaml();
      dlg.classList.remove('hidden');
    };

    window.closeA2aAddRouteModal = function() {
      const dlg = $('a2a-add-route-dialog');
      if (dlg) dlg.classList.add('hidden');
    };

    window.openA2aEditRouteModal = function(name) {
      window.a2aEditingRoute = name;
      const dlg = $('a2a-add-route-dialog');
      if (!dlg) return;
      const routes = state.customRoutes || [];
      const r = routes.find(rt => rt.name === name);
      if (!r) return;

      if ($('a2a-route-modal-title')) $('a2a-route-modal-title').textContent = uiText('Edit route', '编辑路由');
      if ($('a2a-route-name')) {
        $('a2a-route-name').value = r.name;
        $('a2a-route-name').disabled = true;
      }
      if ($('a2a-route-kind')) $('a2a-route-kind').value = (r.protocol || r.kind || 'a2a').toLowerCase();
      const gwSelect = $('a2a-route-gateway');
      if (gwSelect) {
        const gws = state.customGateways || [{ name: 'default' }];
        gwSelect.innerHTML = gws.map(g => '<option value="' + esc(g.name) + '" ' + (g.name === (r.gateway || 'default') ? 'selected' : '') + '>' + esc(g.name) + '</option>').join('');
      }
      if ($('a2a-route-hostnames')) $('a2a-route-hostnames').value = (r.hostnames || []).join(', ');
      const p = (r.matches && r.matches[0] && r.matches[0].path) || {};
      if ($('a2a-route-path-type')) $('a2a-route-path-type').value = p.type || 'prefix';
      if ($('a2a-route-path-value')) $('a2a-route-path-value').value = p.value || '';
      if ($('a2a-route-method')) $('a2a-route-method').value = (r.matches && r.matches[0] && r.matches[0].method) || '';

      const bContainer = $('a2a-route-backends-container');
      if (bContainer) {
        bContainer.innerHTML = '';
        const bList = r.weighted_backends || r.backends || [];
        if (bList.length > 0) {
          bList.forEach(b => addA2aRouteBackendRow(b.name, b.weight || 100));
        } else {
          addA2aRouteBackendRow('backend-1', 100);
        }
      }

      updateA2aRouteYaml();
      dlg.classList.remove('hidden');
    };

    window.addA2aRouteHeaderRow = function(name, val) {
      const container = $('a2a-route-headers-container');
      if (!container) return;
      const emptyBox = container.querySelector('.a2a-box-empty');
      if (emptyBox) emptyBox.remove();
      const row = document.createElement('div');
      row.className = 'a2a-match-row';
      row.style.cssText = 'display:flex;gap:8px;align-items:center;margin-top:6px;';
      row.innerHTML = '<input type="text" class="a2a-input a2a-hdr-name" placeholder="Header name" value="' + esc(name || '') + '" oninput="updateA2aRouteYaml()" style="flex:1;">'
        + '<input type="text" class="a2a-input a2a-hdr-val" placeholder="Value" value="' + esc(val || '') + '" oninput="updateA2aRouteYaml()" style="flex:1;">'
        + '<button type="button" class="btn btn-ghost" style="color:#ef4444;padding:4px 8px;" onclick="this.parentElement.remove();if(!$(\'a2a-route-headers-container\').children.length)$(\'a2a-route-headers-container\').innerHTML=\'<div class=\\\'a2a-box-empty\\\'>No header matches configured. Click &quot;+ Add header match&quot; to add one.</div>\';updateA2aRouteYaml();">✕</button>';
      container.appendChild(row);
      updateA2aRouteYaml();
    };

    window.addA2aRouteQueryRow = function(key, val) {
      const container = $('a2a-route-queries-container');
      if (!container) return;
      const emptyBox = container.querySelector('.a2a-box-empty');
      if (emptyBox) emptyBox.remove();
      const row = document.createElement('div');
      row.className = 'a2a-match-row';
      row.style.cssText = 'display:flex;gap:8px;align-items:center;margin-top:6px;';
      row.innerHTML = '<input type="text" class="a2a-input a2a-qry-key" placeholder="Query parameter" value="' + esc(key || '') + '" oninput="updateA2aRouteYaml()" style="flex:1;">'
        + '<input type="text" class="a2a-input a2a-qry-val" placeholder="Value" value="' + esc(val || '') + '" oninput="updateA2aRouteYaml()" style="flex:1;">'
        + '<button type="button" class="btn btn-ghost" style="color:#ef4444;padding:4px 8px;" onclick="this.parentElement.remove();if(!$(\'a2a-route-queries-container\').children.length)$(\'a2a-route-queries-container\').innerHTML=\'<div class=\\\'a2a-box-empty\\\'>No query matches configured. Click &quot;+ Add query match&quot; to add one.</div>\';updateA2aRouteYaml();">✕</button>';
      container.appendChild(row);
      updateA2aRouteYaml();
    };

    window.addA2aRouteBackendRow = function(name, weight) {
      const container = $('a2a-route-backends-container');
      if (!container) return;
      const row = document.createElement('div');
      row.className = 'a2a-backend-row';
      row.style.cssText = 'display:flex;gap:8px;align-items:center;margin-top:6px;';
      row.innerHTML = '<input type="text" class="a2a-input a2a-backend-name" placeholder="Backend name" value="' + esc(name || '') + '" oninput="updateA2aRouteYaml()" style="flex:2;">'
        + '<div style="display:flex;align-items:center;gap:4px;flex:1;">'
        + '  <input type="number" class="a2a-input a2a-backend-weight" placeholder="Weight" value="' + (weight !== undefined ? weight : 100) + '" min="0" max="100" oninput="updateA2aRouteYaml()" style="width:70px;">'
        + '  <span style="font-size:12px;color:var(--muted)">%</span>'
        + '</div>'
        + '<button type="button" class="btn btn-ghost" style="color:#ef4444;padding:4px 8px;" onclick="if(document.querySelectorAll(\'#a2a-route-backends-container .a2a-backend-row\').length>1){this.parentElement.remove();updateA2aRouteYaml();}">✕</button>';
      container.appendChild(row);
      updateA2aRouteYaml();
    };

    window.updateA2aRouteYaml = function() {
      const name = ($('a2a-route-name')?.value || 'my-route').trim();
      const kind = $('a2a-route-kind')?.value || 'a2a';
      const gateway = $('a2a-route-gateway')?.value || 'default';
      const hostnames = ($('a2a-route-hostnames')?.value || '').split(',').map(s => s.trim()).filter(Boolean);
      const pathType = $('a2a-route-path-type')?.value || 'prefix';
      const pathVal = $('a2a-route-path-value')?.value || '/a2a';
      const method = $('a2a-route-method')?.value || '';

      let yaml = 'routes:\n  - name: ' + name + '\n    kind: ' + kind + '\n    gateway: ' + gateway;
      if (hostnames.length > 0) {
        yaml += '\n    hostnames:';
        hostnames.forEach(h => { yaml += '\n      - "' + h + '"'; });
      }
      yaml += '\n    matches:\n      - path:\n          type: ' + pathType + '\n          value: ' + pathVal;
      if (method) {
        yaml += '\n        method: ' + method;
      }
      const bRows = document.querySelectorAll('#a2a-route-backends-container .a2a-backend-row');
      if (bRows.length > 0) {
        yaml += '\n    backends:';
        bRows.forEach(r => {
          const bName = (r.querySelector('.a2a-backend-name')?.value || 'backend').trim();
          const bWeight = parseInt(r.querySelector('.a2a-backend-weight')?.value || '100', 10);
          yaml += '\n      - name: ' + bName + '\n        weight: ' + bWeight;
        });
      }

      const preview = $('a2a-route-yaml-preview');
      if (preview) preview.textContent = yaml;
    };

    window.toggleA2aRouteYaml = function(el) {
      const preview = $('a2a-route-yaml-preview');
      const arrow = $('a2a-route-yaml-arrow');
      if (preview) {
        const isHidden = preview.classList.toggle('hidden');
        if (arrow) arrow.textContent = isHidden ? '▶' : '▼';
      }
    };

    window.handleA2aSaveRoute = function(e) {
      if (e) e.preventDefault();
      const name = ($('a2a-route-name')?.value || '').trim();
      const kind = $('a2a-route-kind')?.value || 'a2a';
      const gateway = $('a2a-route-gateway')?.value || 'default';
      const hostnames = ($('a2a-route-hostnames')?.value || '').split(',').map(s => s.trim()).filter(Boolean);
      const pathType = $('a2a-route-path-type')?.value || 'prefix';
      const pathVal = $('a2a-route-path-value')?.value || '/a2a';
      const method = $('a2a-route-method')?.value || '';
      if (!name) return;

      const backends = [];
      document.querySelectorAll('#a2a-route-backends-container .a2a-backend-row').forEach(r => {
        const bName = (r.querySelector('.a2a-backend-name')?.value || '').trim();
        const bWeight = parseInt(r.querySelector('.a2a-backend-weight')?.value || '100', 10);
        if (bName) backends.push({ name: bName, weight: bWeight });
      });

      const routeObj = {
        name,
        protocol: kind,
        gateway,
        hostnames,
        matches: [{ path: { type: pathType, value: pathVal }, ...(method ? { method } : {}) }],
        weighted_backends: backends
      };

      state.customRoutes = state.customRoutes || [];
      if (window.a2aEditingRoute) {
        const idx = state.customRoutes.findIndex(r => r.name === window.a2aEditingRoute);
        if (idx >= 0) state.customRoutes[idx] = routeObj;
      } else {
        const idx = state.customRoutes.findIndex(r => r.name === name);
        if (idx >= 0) state.customRoutes[idx] = routeObj;
        else state.customRoutes.push(routeObj);
      }
      closeA2aAddRouteModal();
      renderA2a();
    };

    window.handleA2aDeleteRoute = function(name) {
      if (!name) return;
      if (state.customRoutes) {
        state.customRoutes = state.customRoutes.filter(r => r.name !== name);
      }
      renderA2a();
    };

    window.handleA2aAgentAction = function() {
      unavailableAction();
    };

    window.cancelCurrentTaskChain = function() {
      unavailableAction();
    };

    window.handleA2aSearch = function(val) {
      window.a2aSearchQuery = (val || '').trim().toLowerCase();
      renderA2a();
    };

    window.handleA2aRoleFilter = function(role) {
      window.a2aRoleFilter = role;
      renderA2a();
    };

    window.handleA2aStatusFilter = function(status) {
      window.a2aStatusFilter = status;
      renderA2a();
    };

    window.handleA2aTeamFilter = function(team) {
      window.a2aTeamFilter = team;
      renderA2a();
    };

    window.filterA2aTaskTab = function(tab, btn) {
      window.a2aTaskTab = tab;
      document.querySelectorAll('#a2a-tasks-panel .tool-filter-btn').forEach((b) => b.classList.toggle('active', b === btn));
      renderA2a();
    };

    window.handleMcpToolToggle = function() {
      unavailableAction();
    };

    window.filterMcpCatalog = function(risk, btn) {
      window.mcpCatalogRiskFilter = risk;
      document.querySelectorAll('.tool-filter-btn').forEach((b) => b.classList.toggle('active', b === btn));
      if (typeof renderMcp === 'function') {
        renderMcp();
      }
    };

    window.selectedLlmProvider = 'all';
    window.filterLlmProvider = function(provider, btn) {
      window.selectedLlmProvider = provider;
      document.querySelectorAll('.llm-provider-tab').forEach((b) => {
        b.classList.toggle('active', b.dataset.provider === provider);
      });
      if (typeof renderLlm === 'function') {
        renderLlm();
      }
    };

    window.handleLlmAccountAction = function() {
      unavailableAction();
    };

    window.handlePoolAction = function() {
      unavailableAction();
    };

    function clusterDetail(cluster) {
      return (cluster.endpoints || []).map((e) => e.address + ':' + e.port + (e.healthy === false ? ' (unhealthy)' : '')).join(', ');
    }

    function clusterTls(cluster) {
      if (!cluster || !cluster.tls) return 'plaintext';
      return cluster.tls.mode === 'simple' ? 'tls' : 'mtls';
    }

    function pathText(match) {
      const path = match && match.path;
      if (!path) return '/';
      const value = path.value || '/';
      return path.type === 'exact' ? value : value;
    }

    function clusterMap(config) {
      const map = {};
      (config.clusters || []).forEach((cluster) => {
        map[cluster.name] = cluster;
      });
      return map;
    }

    function buildServicesClientFallback(config) {
      const clusters = clusterMap(config);
      const unified = [];
      let totalHealthyEps = 0;
      let totalAllEps = 0;
      const domainsSet = new Set();

      // 1. Ingress xDS routes
      (config.listeners || []).forEach((listener) => {
        const listenerPort = listener.bind ? parseInt(listener.bind.split(':').pop() || '80', 10) : 80;
        const listenerStr = listener.name + ':' + listenerPort;

        (listener.virtual_hosts || []).forEach((host) => {
          (host.domains || []).forEach(d => domainsSet.add(d));
          const primaryDomain = (host.domains && host.domains[0]) || '*';

          (host.routes || []).forEach((route) => {
            const id = 'xds:' + listener.name + ':' + primaryDomain + ':' + (route.name || '');
            let pathStr = '/';
            let matchType = 'prefix';
            if (route.matches && route.matches[0] && route.matches[0].path) {
              const p = route.matches[0].path;
              matchType = p.type || 'prefix';
              pathStr = p.value || p;
            }

            const headers = [];
            (route.matches || []).forEach(m => {
              (m.headers || []).forEach(h => headers.push({ name: h.name, value: h.value }));
            });

            const clusterItems = [];
            let routeHealthyEps = 0;
            let routeTotalEps = 0;
            let routeTlsModes = [];
            const totalWeight = (route.weighted_clusters || []).reduce((acc, c) => acc + (c.weight || 0), 0);

            (route.weighted_clusters || []).forEach((wc) => {
              const percent = totalWeight > 0 ? (wc.weight / totalWeight) * 100 : 100;
              const cl = clusters[wc.name] || {};
              const http2 = !!cl.http2;
              const tlsMode = cl.tls ? 'tls' : 'plaintext';
              if (!routeTlsModes.includes(tlsMode)) routeTlsModes.push(tlsMode);

              const eps = (cl.endpoints || []).map((ep) => {
                const isHealthy = ep.healthy !== false;
                if (isHealthy) {
                  routeHealthyEps++;
                  totalHealthyEps++;
                }
                routeTotalEps++;
                totalAllEps++;
                return {
                  address: (ep.address || '127.0.0.1') + ':' + (ep.port || 8080),
                  weight: ep.weight || 1,
                  health_status: isHealthy ? 'healthy' : 'unhealthy'
                };
              });

              clusterItems.push({
                name: wc.name,
                weight: wc.weight || 100,
                percent: Math.round(percent),
                http2: http2,
                tls_mode: tlsMode,
                circuit_breaker: cl.circuit_breaker ? 'Max Conns: ' + (cl.circuit_breaker.max_connections || 'default') : null,
                outlier_detection: cl.outlier_detection ? 'Interval: ' + (cl.outlier_detection.interval || '10s') : null,
                endpoints: eps
              });
            });

            const status = (routeTotalEps > 0 && routeHealthyEps < routeTotalEps) ? 'degraded' : 'healthy';

            unified.push({
              id: id,
              name: route.name || 'unnamed',
              source: 'xds',
              listener: listenerStr,
              listener_port: listenerPort,
              domain: primaryDomain,
              path: pathStr,
              match_type: matchType,
              methods: ['*'],
              headers: headers,
              protocol: clusterItems.some(c => c.http2) ? 'HTTP/2' : 'HTTP/1.1',
              tls_mode: routeTlsModes.includes('tls') ? 'tls' : 'plaintext',
              clusters: clusterItems,
              metrics: {
                requests: 0,
                failures: 0,
                in_flight: 0,
                p95_ms: 0,
                error_rate_pct: 0.0
              },
              policies: (listener.security && listener.security.authorization) ? listener.security.authorization.map(a => 'authz:' + (a.action || 'allow')) : [],
              replace_prefix_match: null,
              health_ratio: routeTotalEps > 0 ? (routeHealthyEps + '/' + routeTotalEps) : '1/1',
              status: status
            });
          });
        });
      });

      // 2. Agent HTTP routes
      (config.routes || []).forEach((route) => {
        if (route.protocol === 'http') {
          domainsSet.add('agent-mesh.local');
          const id = 'agent:http:' + route.name;
          let pathStr = '/';
          let matchType = 'prefix';
          if (route.matches && route.matches[0] && route.matches[0].path) {
            const p = route.matches[0].path;
            matchType = p.type || 'prefix';
            pathStr = p.value || p;
          }

          const clusterItems = [];
          (route.weighted_backends || []).forEach((wb) => {
            clusterItems.push({
              name: wb.name,
              weight: wb.weight || 100,
              percent: 100,
              http2: false,
              tls_mode: 'plaintext',
              circuit_breaker: null,
              outlier_detection: null,
              endpoints: [{
                address: '127.0.0.1:8080',
                weight: wb.weight || 1,
                health_status: 'healthy'
              }]
            });
            totalHealthyEps++;
            totalAllEps++;
          });

          unified.push({
            id: id,
            name: route.name,
            source: 'agent_http',
            listener: 'agent-ingress:8080',
            listener_port: 8080,
            domain: 'agent-mesh.local',
            path: pathStr,
            match_type: matchType,
            methods: ['*'],
            headers: [],
            protocol: 'HTTP/1.1',
            tls_mode: 'plaintext',
            clusters: clusterItems,
            metrics: { requests: 0, failures: 0, in_flight: 0, p95_ms: 0, error_rate_pct: 0.0 },
            policies: route.policies || [],
            replace_prefix_match: route.replace_prefix_match || null,
            health_ratio: '1/1',
            status: 'healthy'
          });
        }
      });

      return {
        kpis: {
          total_services: domainsSet.size,
          total_routes: unified.length,
          healthy_endpoints: totalHealthyEps,
          total_endpoints: totalAllEps,
          error_rate_pct: 0.0,
          config_version: config.version || 'v1'
        },
        services: unified
      };
    }

    function renderServices() {
      const sTable = $('services-table');
      if (!sTable) return;

      const config = state.config || {};
      const svcData = state.servicesData || buildServicesClientFallback(config);
      const kpis = svcData.kpis || {};
      const allServices = svcData.services || [];

      // Populate filter dropdowns if not done
      const listenerSel = $('svc-listener-filter');
      const domainSel = $('svc-domain-filter');
      if (listenerSel && listenerSel.options && listenerSel.options.length <= 1) {
        const listeners = [...new Set(allServices.map(s => s.listener))];
        listeners.forEach(l => {
          const opt = document.createElement('option');
          opt.value = l;
          opt.textContent = 'Listener: ' + l;
          listenerSel.appendChild(opt);
        });
      }
      if (domainSel && domainSel.options && domainSel.options.length <= 1) {
        const domains = [...new Set(allServices.map(s => s.domain))];
        domains.forEach(d => {
          const opt = document.createElement('option');
          opt.value = d;
          opt.textContent = 'Domain: ' + d;
          domainSel.appendChild(opt);
        });
      }

      // Filter state defaults
      if (!state.svcSearchQuery) state.svcSearchQuery = '';
      if (!state.svcListenerFilter) state.svcListenerFilter = 'all';
      if (!state.svcDomainFilter) state.svcDomainFilter = 'all';
      if (!state.svcSourceFilter) state.svcSourceFilter = 'all';
      if (!state.svcHealthFilter) state.svcHealthFilter = 'all';
      if (typeof state.svcSelectedId !== 'string' && allServices.length > 0) {
        state.svcSelectedId = allServices[0].id;
      }

      // Filter logic
      const q = (state.svcSearchQuery || '').trim().toLowerCase();
      const filtered = allServices.filter(s => {
        if (state.svcListenerFilter !== 'all' && s.listener !== state.svcListenerFilter) return false;
        if (state.svcDomainFilter !== 'all' && s.domain !== state.svcDomainFilter) return false;
        if (state.svcSourceFilter !== 'all' && s.source !== state.svcSourceFilter) return false;
        if (state.svcHealthFilter !== 'all' && s.status !== state.svcHealthFilter) return false;
        if (q) {
          const hay = [s.name, s.domain, s.path, s.listener, s.source, s.status, (s.clusters || []).map(c => c.name).join(' ')].join(' ').toLowerCase();
          if (!hay.includes(q)) return false;
        }
        return true;
      });

      if ($('svc-count-badge')) $('svc-count-badge').textContent = filtered.length + ' Routes';

      // Group by Domain
      const domainGroups = {};
      filtered.forEach(s => {
        if (!domainGroups[s.domain]) domainGroups[s.domain] = [];
        domainGroups[s.domain].push(s);
      });

      const head = '<thead><tr>' +
        '<th>Route &amp; Hierarchy</th>' +
        '<th>Match Pattern</th>' +
        '<th>Target Clusters &amp; Weight</th>' +
        '<th>Endpoints</th>' +
        '<th>Traffic / P95</th>' +
        '<th>Status</th>' +
        '<th style="text-align:right">Action</th>' +
        '</tr></thead>';

      if (filtered.length === 0) {
        sTable.innerHTML = head + '<tbody><tr><td colspan="7" style="text-align:center;padding:24px;color:var(--muted)">No matching APIs found</td></tr></tbody>';
        return;
      }

      let tbodyHtml = '<tbody>';
      Object.keys(domainGroups).forEach(domain => {
        const routes = domainGroups[domain];
        tbodyHtml += '<tr class="svc-domain-row">' +
          '<td colspan="7" class="svc-domain-cell">' +
          '<div style="display:flex;align-items:center;gap:8px">' +
          '<svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><circle cx="8" cy="8" r="6"/><path d="M2 8h12M8 2a9 9 0 0 1 0 12M8 2a9 9 0 0 0 0 12"/></svg>' +
          '<span>' + esc(domain) + '</span>' +
          '<span style="font-size:10.5px;color:var(--muted);font-weight:normal">(' + routes.length + ' routes)</span>' +
          '</div></td></tr>';

        routes.forEach((svc, rIdx) => {
          const isLast = rIdx === routes.length - 1;
          const treeSym = isLast ? '└──' : '├──';
          const isSel = state.svcSelectedId === svc.id;

          // Target cluster chips
          const clustersHtml = (svc.clusters || []).map(c =>
            '<span class="svc-cluster-chip">' + esc(c.name) + ' <span class="svc-cluster-weight">' + c.percent + '%</span></span>'
          ).join('');

          // Health pill
          const hCls = svc.status === 'healthy' ? 'healthy' : (svc.status === 'degraded' ? 'degraded' : 'unhealthy');

          // Match tag
          const matchTag = '<span class="svc-match-tag">' + esc(svc.match_type.toUpperCase()) + ' ' + esc(svc.path) + '</span>';

          // Traffic pill
          const m = svc.metrics || {};
          const trafficStr = (m.requests > 0) ? (m.requests + ' req · ' + (m.p95_ms || 12) + 'ms') : (svc.source === 'xds' ? 'xDS Live' : 'Ready');

          tbodyHtml += '<tr class="svc-route-row ' + (isSel ? 'active-row' : '') + '" data-svc-id="' + esc(svc.id) + '">' +
            '<td><span class="svc-tree-connector">' + treeSym + '</span> <strong>' + esc(svc.name) + '</strong> ' +
            '<span class="svc-source-tag ' + (svc.source === 'xds' ? 'xds' : 'agent') + '">' + esc(svc.source === 'xds' ? 'xDS' : 'Agent') + '</span></td>' +
            '<td>' + matchTag + '</td>' +
            '<td>' + clustersHtml + '</td>' +
            '<td><span class="svc-health-pill ' + hCls + '">' + esc(svc.health_ratio) + '</span></td>' +
            '<td class="code muted" style="font-size:11px">' + esc(trafficStr) + '</td>' +
            '<td><span class="status-badge ' + hCls + '">' + esc(svc.status) + '</span></td>' +
            '<td style="text-align:right"><button type="button" class="btn btn-secondary" style="font-size:10.5px;padding:2px 8px">Inspect</button></td>' +
            '</tr>';
        });
      });
      tbodyHtml += '</tbody>';
      sTable.innerHTML = head + tbodyHtml;

      // Event bindings
      sTable.querySelectorAll('.svc-route-row').forEach(row => {
        row.addEventListener('click', () => {
          const id = row.dataset.svcId;
          state.svcSelectedId = id;
          state.svcDrawerOpen = true;
          renderServices();
          openServiceDrawer(id, allServices);
        });
      });

      // Filter events
      const searchIn = $('svc-search-input');
      if (searchIn && !searchIn.dataset.ready) {
        searchIn.dataset.ready = '1';
        searchIn.addEventListener('input', (e) => {
          state.svcSearchQuery = e.target.value;
          renderServices();
        });
      }
      if (listenerSel && !listenerSel.dataset.ready) {
        listenerSel.dataset.ready = '1';
        listenerSel.addEventListener('change', (e) => {
          state.svcListenerFilter = e.target.value;
          renderServices();
        });
      }
      if (domainSel && !domainSel.dataset.ready) {
        domainSel.dataset.ready = '1';
        domainSel.addEventListener('change', (e) => {
          state.svcDomainFilter = e.target.value;
          renderServices();
        });
      }
      const sourceSel = $('svc-source-filter');
      if (sourceSel && !sourceSel.dataset.ready) {
        sourceSel.dataset.ready = '1';
        sourceSel.addEventListener('change', (e) => {
          state.svcSourceFilter = e.target.value;
          renderServices();
        });
      }
      const healthSel = $('svc-health-filter');
      if (healthSel && !healthSel.dataset.ready) {
        healthSel.dataset.ready = '1';
        healthSel.addEventListener('change', (e) => {
          state.svcHealthFilter = e.target.value;
          renderServices();
        });
      }
      const resetBtn = $('svc-reset-btn');
      if (resetBtn && !resetBtn.dataset.ready) {
        resetBtn.dataset.ready = '1';
        resetBtn.addEventListener('click', () => {
          state.svcSearchQuery = '';
          state.svcListenerFilter = 'all';
          state.svcDomainFilter = 'all';
          state.svcSourceFilter = 'all';
          state.svcHealthFilter = 'all';
          if ($('svc-search-input')) $('svc-search-input').value = '';
          if (listenerSel) listenerSel.value = 'all';
          if (domainSel) domainSel.value = 'all';
          if (sourceSel) sourceSel.value = 'all';
          if (healthSel) healthSel.value = 'all';
          renderServices();
        });
      }

      // Drawer close & backdrop
      const closeBtn = $('svc-drawer-close');
      const backdrop = $('svc-drawer-backdrop');
      const drawer = $('svc-drawer');
      if (closeBtn && !closeBtn.dataset.ready) {
        closeBtn.dataset.ready = '1';
        closeBtn.addEventListener('click', () => {
          state.svcDrawerOpen = false;
          if (drawer) drawer.classList.remove('open');
          if (backdrop) backdrop.classList.remove('open');
        });
      }
      if (backdrop && !backdrop.dataset.ready) {
        backdrop.dataset.ready = '1';
        backdrop.addEventListener('click', () => {
          state.svcDrawerOpen = false;
          if (drawer) drawer.classList.remove('open');
          if (backdrop) backdrop.classList.remove('open');
        });
      }

      // Drawer tab switching
      document.querySelectorAll('.svc-drawer-tab-btn').forEach(btn => {
        if (!btn.dataset.ready) {
          btn.dataset.ready = '1';
          btn.addEventListener('click', () => {
            document.querySelectorAll('.svc-drawer-tab-btn').forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            const tabId = btn.dataset.tab;
            document.querySelectorAll('.svc-drawer-view').forEach(v => v.classList.remove('active'));
            const targetView = $('svc-tab-' + tabId);
            if (targetView) targetView.classList.add('active');
          });
        }
      });

      if (state.svcDrawerOpen && state.svcSelectedId) {
        openServiceDrawer(state.svcSelectedId, allServices);
      }
    }

    function openServiceDrawer(svcId, allServices) {
      const svc = allServices.find(s => s.id === svcId);
      if (!svc) return;

      const drawer = $('svc-drawer');
      const backdrop = $('svc-drawer-backdrop');
      if (drawer) drawer.classList.add('open');
      if (backdrop) backdrop.classList.add('open');

      if ($('svc-drawer-name')) $('svc-drawer-name').textContent = svc.name;
      if ($('svc-drawer-domain')) $('svc-drawer-domain').textContent = 'Domain: ' + svc.domain + ' · ' + svc.listener;
      if ($('svc-drawer-status')) {
        const badge = $('svc-drawer-status');
        badge.className = 'svc-health-pill ' + (svc.status === 'healthy' ? 'healthy' : (svc.status === 'degraded' ? 'degraded' : 'unhealthy'));
        badge.textContent = svc.status.charAt(0).toUpperCase() + svc.status.slice(1);
      }
      if ($('svc-drawer-source')) {
        const sTag = $('svc-drawer-source');
        sTag.className = 'svc-source-tag ' + (svc.source === 'xds' ? 'xds' : 'agent');
        sTag.textContent = svc.source === 'xds' ? 'xDS Ingress' : 'Agent HTTP';
      }

      // Tab 1: Overview
      if ($('svc-prop-id')) $('svc-prop-id').textContent = svc.id;
      if ($('svc-prop-listener')) $('svc-prop-listener').textContent = svc.listener;
      if ($('svc-prop-domain')) $('svc-prop-domain').textContent = svc.domain;
      if ($('svc-prop-protocol')) $('svc-prop-protocol').textContent = svc.protocol + ' (' + svc.tls_mode.toUpperCase() + ')';
      if ($('svc-prop-rewrite')) $('svc-prop-rewrite').textContent = svc.replace_prefix_match || 'None';

      // Tab 2: Routes
      if ($('svc-route-match-type')) $('svc-route-match-type').textContent = svc.match_type;
      if ($('svc-route-match-path')) $('svc-route-match-path').textContent = svc.path;
      if ($('svc-route-methods')) $('svc-route-methods').textContent = (svc.methods || []).join(', ');
      const headersTbody = $('svc-headers-tbody');
      if (headersTbody) {
        if (!svc.headers || svc.headers.length === 0) {
          headersTbody.innerHTML = '<tr><td colspan="2" class="muted">No custom header conditions required</td></tr>';
        } else {
          headersTbody.innerHTML = svc.headers.map(h =>
            '<tr><td class="code bold">' + esc(h.name) + '</td><td class="code">' + esc(h.value) + '</td></tr>'
          ).join('');
        }
      }

      // Tab 3: Backends
      const clustersTbody = $('svc-clusters-tbody');
      if (clustersTbody) {
        clustersTbody.innerHTML = (svc.clusters || []).map(c =>
          '<tr>' +
          '<td class="code bold">' + esc(c.name) + '</td>' +
          '<td class="code">' + c.weight + '</td>' +
          '<td class="code">' + c.percent + '%</td>' +
          '<td><span class="cap-tag">' + (c.http2 ? 'HTTP/2' : 'HTTP/1.1') + '</span></td>' +
          '<td><span class="level ' + esc(c.tls_mode) + '">' + esc(c.tls_mode) + '</span></td>' +
          '</tr>'
        ).join('');
      }
      const resContainer = $('svc-clusters-resilience');
      if (resContainer) {
        resContainer.innerHTML = (svc.clusters || []).map(c => {
          return '<div style="background:var(--soft);border:1px solid var(--line);border-radius:6px;padding:8px 10px">' +
            '<div style="font-size:11.5px;font-weight:750;margin-bottom:4px">' + esc(c.name) + ' Resilience</div>' +
            '<div style="font-size:11px;color:var(--muted)">Circuit Breaker: ' + esc(c.circuit_breaker || 'Default threshold (1024 conns)') + '</div>' +
            '<div style="font-size:11px;color:var(--muted)">Outlier Detection: ' + esc(c.outlier_detection || 'Enabled (5 consecutive 5xx, 10s)') + '</div>' +
            '</div>';
        }).join('');
      }

      // Tab 4: Health & Endpoints
      const endTbody = $('svc-endpoints-tbody');
      let hCount = 0;
      let tCount = 0;
      if (endTbody) {
        const allEps = [];
        (svc.clusters || []).forEach(c => {
          (c.endpoints || []).forEach(ep => {
            tCount++;
            if (ep.health_status === 'healthy') hCount++;
            allEps.push(ep);
          });
        });
        if (allEps.length === 0) {
          endTbody.innerHTML = '<tr><td colspan="3" class="muted">No static/discovered endpoints</td></tr>';
        } else {
          endTbody.innerHTML = allEps.map(ep =>
            '<tr>' +
            '<td class="code bold">' + esc(ep.address) + '</td>' +
            '<td class="code">' + ep.weight + '</td>' +
            '<td><span class="status-badge ' + (ep.health_status === 'healthy' ? 'healthy' : 'unhealthy') + '">' + esc(ep.health_status) + '</span></td>' +
            '</tr>'
          ).join('');
        }
      }
      if ($('svc-health-summary-text')) $('svc-health-summary-text').textContent = hCount + ' / ' + tCount + ' Healthy';
      if ($('svc-health-summary-badge')) {
        const b = $('svc-health-summary-badge');
        const pct = tCount > 0 ? Math.round((hCount / tCount) * 100) : 100;
        b.className = 'svc-health-pill ' + (pct === 100 ? 'healthy' : (pct >= 50 ? 'degraded' : 'unhealthy'));
        b.textContent = pct + '% Healthy';
      }

      // Tab 5: Traffic
      const m = svc.metrics || {};
      if ($('svc-traffic-reqs')) $('svc-traffic-reqs').textContent = m.requests || 0;
      if ($('svc-traffic-concurrency')) $('svc-traffic-concurrency').textContent = m.in_flight || 0;
      if ($('svc-traffic-p95')) $('svc-traffic-p95').textContent = (m.p95_ms || 12) + ' ms';
      if ($('svc-traffic-err-rate')) $('svc-traffic-err-rate').textContent = (m.error_rate_pct || 0).toFixed(2) + '%';

      // Tab 6: Policies
      const polList = $('svc-policies-list');
      if (polList) {
        if (!svc.policies || svc.policies.length === 0) {
          polList.innerHTML = '<div style="color:var(--muted);font-size:11.5px;padding:8px">No security/routing policies attached</div>';
        } else {
          polList.innerHTML = svc.policies.map(p =>
            '<div style="display:flex;align-items:center;justify-content:space-between;background:var(--soft);padding:6px 10px;border-radius:4px;border:1px solid var(--line)">' +
            '<span class="code bold">' + esc(p) + '</span>' +
            '<span class="status-badge healthy">Active</span>' +
            '</div>'
          ).join('');
        }
      }

      // Actions: Probe, Config, Trace, Copy
      const probeBtn = $('svc-act-probe');
      const probeBox = $('svc-probe-box');
      if (probeBtn) {
        probeBtn.onclick = () => {
          probeBox.style.display = probeBox.style.display === 'none' ? 'block' : 'none';
          if ($('svc-probe-path')) $('svc-probe-path').value = svc.path;
          if ($('svc-probe-output')) {
            $('svc-probe-output').textContent = uiText('Probe is not connected. Target: ', '探测接口尚未接入。目标：') + svc.domain + svc.path;
          }
        };
      }
      const probeRunBtn = $('svc-probe-run-btn');
      if (probeRunBtn) {
        probeRunBtn.onclick = () => {
          unavailableAction();
        };
      }

      const cfgBtn = $('svc-act-view-config');
      const cfgBox = $('svc-config-box');
      if (cfgBtn) {
        cfgBtn.onclick = () => {
          cfgBox.style.display = cfgBox.style.display === 'none' ? 'block' : 'none';
          if ($('svc-config-json')) {
            $('svc-config-json').textContent = JSON.stringify({
              "route_name": svc.name,
              "source": svc.source,
              "domain": svc.domain,
              "match": { "type": svc.match_type, "path": svc.path, "methods": svc.methods, "headers": svc.headers },
              "weighted_clusters": svc.clusters,
              "policies": svc.policies
            }, null, 2);
          }
        };
      }
      const cfgCopyBtn = $('svc-config-copy-btn');
      if (cfgCopyBtn) {
        cfgCopyBtn.onclick = () => {
          if ($('svc-config-json')) {
            copyText($('svc-config-json').textContent);
          }
        };
      }

      const traceBtn = $('svc-act-open-trace');
      if (traceBtn) {
        traceBtn.onclick = () => {
          if (window.jumpToTrace) {
            window.jumpToTrace(svc.name);
          }
        };
      }

      const copyCurlBtn = $('svc-act-copy-curl');
      if (copyCurlBtn) {
        copyCurlBtn.onclick = () => {
          const quote = value => "'" + String(value).replace(/'/g, "'\"'\"'") + "'";
          if (!svc.listener_port) { unavailableAction(); return; }
          const target = (svc.tls_mode === 'plaintext' ? 'http' : 'https') + '://127.0.0.1:' + svc.listener_port + svc.path;
          copyText('curl -i -H ' + quote('Host: ' + svc.domain) + ' --url ' + quote(target));
        };
      }
    }

    function matches(row) {
      const q = state.query.trim().toLowerCase();
      if (!q) return true;
      return row.some((cell) => String(cell).toLowerCase().includes(q));
    }

    function inventory(config) {
      const rows = [];
      (config.clusters || []).forEach((cluster) => {
        rows.push([t('apiService'), cluster.name, clusterDetail(cluster)]);
      });
      backendsOf(config, 'mcp').forEach((backend) => {
        rows.push([t('mcpPlugin'), backend.name, backend.endpoint || '']);
      });
      backendsOf(config, 'a2a').forEach((backend) => {
        rows.push([t('agent'), backend.name, backend.agent || backend.endpoint || '']);
      });
      (config.providers || []).forEach((provider) => {
        rows.push([t('llmProvider'), provider.name, provider.kind || provider.base_url || '']);
      });
      return rows;
    }

    function fillTable(id, columns, rows) {
      const el = $(id);
      if (!el) return;
      const visible = rows.filter(matches);
      const head = '<tr>' + columns.map((c) => '<th>' + esc(c) + '</th>').join('') + '</tr>';
      const body = visible.length
        ? visible.map((row) => '<tr>' + row.map((cell) => '<td>' + esc(cell) + '</td>').join('') + '</tr>').join('')
        : '<tr><td colspan="' + columns.length + '" class="muted">' + esc(t('noConfig')) + '</td></tr>';
      el.innerHTML = head + body;
    }

    function gatewayServices(config) {
      const svcData = (state && state.servicesData) ? state.servicesData : buildServicesClientFallback(config || (state && state.config) || {});
      return (svcData && svcData.services) ? svcData.services : [];
    }

    function renderConfiguration() {
      renderConfigurationPage();
    }

    function render() {
      const renderers = { services: renderServices, llm: renderLlm, mcp: renderMcp, a2a: renderA2a, observability: renderObservability, security: renderSecurity, 'cost-control': renderCost, configuration: renderConfiguration };
      if (renderers[state.tab]) {
        renderers[state.tab]();
        updatePageChrome();
        applyPageFilter();
        return;
      }
      const config = state.config;
      const mcp = backendsOf(config, 'mcp');
      const agents = backendsOf(config, 'a2a');
      const llmBackends = backendsOf(config, 'llm');
      const providers = config.providers || [];
      const gateway = gatewayServices(config);
      // 1. Overview Dashboard Updates
      const invList = inventory(config);
      if ($('metric-api')) $('metric-api').textContent = gateway.length;
      if ($('ov-stat-clusters')) $('ov-stat-clusters').textContent = String((config.clusters || []).length);
      if ($('ov-stat-routes')) $('ov-stat-routes').textContent = String((config.routes || []).length + (config.listeners || []).reduce((n, l) => n + (l.virtual_hosts || []).reduce((m, h) => m + (h.routes || []).length, 0), 0));

      if ($('metric-mcp')) $('metric-mcp').textContent = mcp.length;
      if ($('ov-stat-mcp-tools')) $('ov-stat-mcp-tools').textContent = String(mcp.reduce((acc, b) => acc + (b.tools || []).length, 0));

      if ($('metric-agents')) $('metric-agents').textContent = agents.length;
      if ($('ov-stat-a2a-channels')) $('ov-stat-a2a-channels').textContent = String(agents.length);

      if ($('metric-llm')) $('metric-llm').textContent = llmBackends.length;
      if ($('ov-stat-providers')) $('ov-stat-providers').textContent = String(providers.length);

      // Protocol distribution dynamic counts
      ['ov-proto-http', 'ov-proto-sse', 'ov-proto-grpc', 'ov-proto-ws'].forEach(id => { if ($(id)) $(id).textContent = '—'; });

      // Policy overview counts
      const policies = config.policies || [];
      if ($('ov-policy-sec')) $('ov-policy-sec').textContent = String(policies.filter((p) => p.auth).length);
      if ($('ov-policy-traffic')) $('ov-policy-traffic').textContent = String(policies.filter((p) => p.rate_limit || p.token_limit).length);
      if ($('ov-policy-budget')) $('ov-policy-budget').textContent = String(policies.filter((p) => p.token_limit).length);
      if ($('ov-policy-total')) $('ov-policy-total').textContent = String(policies.length);

      // Card jump links
      document.querySelectorAll('.ov-kpi-link[data-tab-jump]').forEach((link) => {
        if (!link.dataset.ready) {
          link.dataset.ready = '1';
          link.addEventListener('click', (e) => {
            e.preventDefault();
            setTab(link.dataset.tabJump);
          });
        }
      });

      // Quick action buttons
      document.querySelectorAll('.ov-action-btn[data-action]').forEach((btn) => {
        if (!btn.dataset.ready) {
          btn.dataset.ready = '1';
          btn.addEventListener('click', () => {
            setTab(btn.dataset.action);
          });
        }
      });
      updatePageChrome();
      applyPageFilter();
    }

    function renderLlm() { renderLlmWorkspace(); }

    function renderMcp() {
      const config = state.config || {};
      const mcpBackends = backendsOf(config, 'mcp');

      // 1. Initial Defaults
      if (!state.mcpSelectedServer) {
        state.mcpSelectedServer = (mcpBackends[0] && mcpBackends[0].name) || '';
      }
      if (typeof state.mcpSelectedInvIdx !== 'number') {
        state.mcpSelectedInvIdx = 0;
      }
      if (!window.mcpCapActiveTab) {
        window.mcpCapActiveTab = 'tools';
      }

      // 2. MCP Target Servers Dataset Assembly
      const serverMap = new Map();
      const deleted = state.deletedMcpServers || new Set();
      mcpBackends.forEach((b) => {
        if (deleted.has(b.name)) return;
        if (!serverMap.has(b.name)) {
          let transportLabel = 'Streamable HTTP';
          if (b.transport) {
            transportLabel = b.transport;
          } else if (b.endpoint?.includes('/sse') || b.protocol === 'sse') {
            transportLabel = 'Legacy SSE';
          } else if (b.endpoint?.startsWith('cmd:') || b.command) {
            transportLabel = 'Command Line';
          }
          serverMap.set(b.name, {
            name: b.name,
            endpoint: b.endpoint || '—',
            transport: b.endpoint?.split(':')[0]?.toUpperCase() || 'HTTP',
            transportLabel,
            sessionMode: '—',
            health: 'ready',
            toolsCount: (b.tools || []).length,
            resCount: (b.resources || []).length,
            promptCount: (b.prompts || []).length,
            lastHandshake: '—'
          });
        }
      });

      (state.localMcpServers || []).forEach((s) => {
        if (deleted.has(s.name)) return;
        if (!serverMap.has(s.name)) {
          serverMap.set(s.name, {
            name: s.name,
            endpoint: s.endpoint || '—',
            transport: s.transport?.split(' ')[0]?.toUpperCase() || 'HTTP',
            transportLabel: s.transport || 'Streamable HTTP',
            sessionMode: '—',
            health: 'ready',
            toolsCount: (s.tools || []).length,
            resCount: 0,
            promptCount: 0,
            lastHandshake: '—'
          });
        }
      });

      const allServers = Array.from(serverMap.values());

      if ($('mcp-target-count-badge')) {
        $('mcp-target-count-badge').textContent = allServers.length + ' ' + (allServers.length === 1 ? uiText('target', '个服务') : uiText('targets', '个服务'));
      }

      // Active selected server
      let curServer = allServers.find((s) => s.name === state.mcpSelectedServer);
      if (!curServer) {
        curServer = allServers[0] || { name: '—', endpoint: '—', transport: '—', transportLabel: '—', sessionMode: '—', toolsCount: 0, resCount: '—', promptCount: '—', lastHandshake: '—' };
        state.mcpSelectedServer = curServer.name;
      }
      if ($('mcp-target-inspector')) $('mcp-target-inspector').classList.toggle('hidden', !allServers.length);
      if ($('mcp-target-inspector-health')) $('mcp-target-inspector-health').innerHTML = '<span class="status-pill healthy"><span class="mcp-status-dot"></span>' + esc(curServer.health || 'ready') + '</span>';

      // Render Target Servers Table (id="mcp-servers-table" preserved for tests)
      const tbodyTargets = $('tbody-mcp-target-servers');
      if (tbodyTargets) {
        if (!allServers.length) {
          tbodyTargets.innerHTML = '<tr><td colspan="5" class="mcp-empty-row">' + uiText('No MCP servers configured.', '未配置 MCP 服务。') + '</td></tr>';
        } else {
          tbodyTargets.innerHTML = allServers.map((s) => {
            const isSelected = s.name === curServer.name;
            return '<tr class="' + (isSelected ? 'active-row' : '') + '" tabindex="0" data-mcp-server="' + esc(s.name) + '">'
              + '<td class="mcp-cell-name"><strong>' + esc(s.name) + '</strong></td>'
              + '<td><span class="mcp-type-badge">' + esc(s.transportLabel) + '</span></td>'
              + '<td><span class="mcp-endpoint-code">' + esc(s.endpoint) + '</span></td>'
              + '<td><span class="mcp-state-badge ready">' + esc(s.health || 'ready') + '</span></td>'
              + '<td style="text-align:right;">'
              + '<div class="mcp-cell-actions">'
              + '<button type="button" class="mcp-icon-btn" title="' + uiText('Edit server', '编辑服务') + '" onclick="event.stopPropagation();openMcpEditModal(\'' + esc(s.name) + '\')">'
              + '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"></path><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path></svg>'
              + '</button>'
              + '<button type="button" class="mcp-icon-btn mcp-delete-btn" title="' + uiText('Delete server', '删除服务') + '" onclick="event.stopPropagation();handleMcpDeleteServer(\'' + esc(s.name) + '\')">'
              + '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>'
              + '</button>'
              + '</div>'
              + '</td>'
              + '</tr>';
          }).join('');
        }
        tbodyTargets.querySelectorAll('[data-mcp-server]').forEach(row => {
          row.onclick = () => {
            state.mcpSelectedInvIdx = 0;
            selectMcpServer(row.dataset.mcpServer);
            applyPageFilter();
          };
        });
      }
      if ($('mcp-target-inspector-name')) $('mcp-target-inspector-name').textContent = curServer.name;
      if ($('mcp-target-inspector-endpoint')) $('mcp-target-inspector-endpoint').textContent = curServer.endpoint;
      if ($('mcp-target-inspector-transport')) $('mcp-target-inspector-transport').textContent = curServer.transportLabel || curServer.transport;
      if ($('mcp-target-inspector-affinity')) $('mcp-target-inspector-affinity').textContent = curServer.sessionMode;
      if ($('mcp-target-inspector-caps-summary')) {
        $('mcp-target-inspector-caps-summary').textContent = curServer.toolsCount + ' Tools, ' + curServer.resCount + ' Resources, ' + curServer.promptCount + ' Prompts';
      }
      if ($('mcp-target-inspector-last-handshake')) $('mcp-target-inspector-last-handshake').textContent = curServer.lastHandshake;

      // 3. Discovered Capabilities Dataset Assembly
      const configuredServer = mcpBackends.find(b => b.name === curServer.name) || (state.localMcpServers || []).find(b => b.name === curServer.name);
      const curCapSet = {
        tools: (configuredServer?.tools || []).map(name => ({ name, desc: uiText('Declared in configuration', '配置声明的工具'), schema: '—', risk: 'Unclassified', policy: '—' })),
        resources: (configuredServer?.resources || []).map(uri => ({ uri, desc: uiText('Declared resource', '配置声明的资源'), mime: '—', policy: '—' })),
        prompts: (configuredServer?.prompts || []).map(name => ({ name, desc: uiText('Declared prompt', '配置声明的提示词'), args: '—', policy: '—' }))
      };
      const activeTab = window.mcpCapActiveTab || 'tools';

      if ($('mcp-cap-panel-title')) {
        $('mcp-cap-panel-title').textContent = curServer.name + ' · ' + uiText('Configured capabilities', '配置声明的能力');
      }

      // Update Capabilities Tab Buttons Text
      if ($('btn-mcp-cap-tools')) $('btn-mcp-cap-tools').textContent = 'Tools (' + (curCapSet.tools || []).length + ')';
      if ($('btn-mcp-cap-resources')) $('btn-mcp-cap-resources').textContent = 'Resources (' + (curCapSet.resources || []).length + ')';
      if ($('btn-mcp-cap-prompts')) $('btn-mcp-cap-prompts').textContent = 'Prompts (' + (curCapSet.prompts || []).length + ')';

      const theadCap = $('thead-mcp-capabilities');
      const tbodyCap = $('tbody-mcp-capabilities');

      if (theadCap && tbodyCap) {
        if (activeTab === 'tools') {
          theadCap.innerHTML = '<tr><th>Tool Name (别名与标识)</th><th>Description (能力描述)</th><th>Input Schema (参数定义)</th><th>Risk Level</th><th>PEP Policy (鉴权策略)</th></tr>';
          const items = curCapSet.tools || [];
          if ($('mcp-cap-count-badge')) $('mcp-cap-count-badge').textContent = items.length + ' 个工具';
          tbodyCap.innerHTML = items.map((t) => {
            const riskCls = t.risk.toLowerCase();
            return '<tr>'
              + '<td class="code bold">' + esc(t.name) + '</td>'
              + '<td>' + esc(t.desc) + '</td>'
              + '<td><code style="font-size:10.5px;color:var(--muted);">' + esc(t.schema) + '</code></td>'
              + '<td><span class="risk-badge ' + riskCls + '">' + esc(t.risk) + '</span></td>'
              + '<td><span class="policy-pill' + (t.policy === '—' ? '' : ' allow') + '">' + esc(t.policy) + '</span></td>'
              + '</tr>';
          }).join('');
        } else if (activeTab === 'resources') {
          theadCap.innerHTML = '<tr><th>Resource URI (资源标识)</th><th>Description (描述)</th><th>MIME Type</th><th>PEP Policy</th></tr>';
          const items = curCapSet.resources || [];
          if ($('mcp-cap-count-badge')) $('mcp-cap-count-badge').textContent = items.length + ' 个资源';
          tbodyCap.innerHTML = items.map((r) => {
            return '<tr>'
              + '<td class="code bold">' + esc(r.uri) + '</td>'
              + '<td>' + esc(r.desc) + '</td>'
              + '<td class="code muted">' + esc(r.mime) + '</td>'
              + '<td><span class="policy-pill allow">' + esc(r.policy) + '</span></td>'
              + '</tr>';
          }).join('');
        } else if (activeTab === 'prompts') {
          theadCap.innerHTML = '<tr><th>Prompt Name (提示词标识)</th><th>Description (说明)</th><th>Declared Arguments (入参要求)</th><th>PEP Policy</th></tr>';
          const items = curCapSet.prompts || [];
          if ($('mcp-cap-count-badge')) $('mcp-cap-count-badge').textContent = items.length + ' 个提示词模板';
          tbodyCap.innerHTML = items.map((p) => {
            return '<tr>'
              + '<td class="code bold">' + esc(p.name) + '</td>'
              + '<td>' + esc(p.desc) + '</td>'
              + '<td class="code">' + esc(p.args) + '</td>'
              + '<td><span class="policy-pill allow">' + esc(p.policy) + '</span></td>'
              + '</tr>';
          }).join('');
        }
      }

      // 4. Invocations Dataset & Detail Inspector Assembly
      if (!tbodyCap.children.length) tbodyCap.innerHTML = emptyRow(5, uiText('No capability data for this selection.', '当前选择暂无能力数据。'));

      const allInvocations = [];
      $('mcp-invocations-badge').textContent = allInvocations.length + uiText(' records', ' 条记录');
      if (state.mcpSelectedInvIdx >= allInvocations.length) {
        state.mcpSelectedInvIdx = 0;
      }
      const activeInv = allInvocations[state.mcpSelectedInvIdx] || allInvocations[0];

      // Render Invocations Table (id="mcp-invocations-table" preserved for tests)
      const tbodyInv = $('tbody-mcp-invocations');
      if (tbodyInv) {
        tbodyInv.innerHTML = allInvocations.map((inv, idx) => {
          const isSelected = idx === state.mcpSelectedInvIdx;
          const isSuccess = inv.status.includes('200');
          let statusBadge = '<span class="status-badge healthy">' + esc(inv.status) + '</span>';
          if (!isSuccess) statusBadge = '<span class="status-badge throttled">' + esc(inv.status) + '</span>';

          return '<tr class="' + (isSelected ? 'active-row' : '') + '" style="cursor:pointer;" onclick="selectMcpInvocation(' + idx + ')">'
            + '<td class="code muted">' + esc(inv.time) + '</td>'
            + '<td class="code bold">' + esc(inv.method) + '</td>'
            + '<td class="code">' + esc(inv.server) + '</td>'
            + '<td class="code"><span class="cap-tag">' + esc(inv.op) + '</span></td>'
            + '<td class="code">' + esc(inv.caller) + '</td>'
            + '<td>' + statusBadge + '</td>'
            + '<td class="code">' + esc(inv.duration) + '</td>'
            + '</tr>';
        }).join('');
      }

      // Update Invocation Audit Detail Panel
      $('mcp-invocation-detail').classList.toggle('hidden', !activeInv);
      if (!activeInv) {
        tbodyInv.innerHTML = emptyRow(7, uiText('No invocation records available for this server.', '当前服务暂无可用的调用记录。'));
        return;
      }
      if ($('mcp-inv-detail-id')) $('mcp-inv-detail-id').textContent = activeInv.id;
      if ($('mcp-inv-detail-status')) {
        const isSuccess = activeInv.status.includes('200');
        $('mcp-inv-detail-status').innerHTML = '<span class="status-badge ' + (isSuccess ? 'healthy' : 'throttled') + '">' + esc(activeInv.status) + '</span>';
      }
      if ($('mcp-inv-detail-session')) $('mcp-inv-detail-session').textContent = activeInv.session;
      if ($('mcp-inv-detail-caller')) $('mcp-inv-detail-caller').textContent = activeInv.caller;
      if ($('mcp-inv-detail-target')) $('mcp-inv-detail-target').textContent = activeInv.server;
      if ($('mcp-inv-detail-method')) $('mcp-inv-detail-method').textContent = activeInv.method + ' :: ' + activeInv.op;
      if ($('mcp-inv-detail-policy')) {
        const isAllow = activeInv.policy.toLowerCase().includes('allow');
        $('mcp-inv-detail-policy').innerHTML = '<span class="policy-pill ' + (isAllow ? 'allow' : 'deny') + '">' + esc(activeInv.policy) + '</span>';
      }
      if ($('mcp-inv-detail-trace')) $('mcp-inv-detail-trace').textContent = activeInv.trace;
      if ($('mcp-inv-detail-args')) $('mcp-inv-detail-args').textContent = activeInv.args;
    }

    function renderA2a() {
      const config = state.config || {};

      // 1. Initialize customGateways if not present
      if (!state.customGateways) {
        const fromConfig = (config.listeners || []).map(l => {
          const port = parseInt((l.bind || '').split(':').pop() || '26080', 10);
          return {
            name: l.name || 'default',
            port: port || 26080,
            isDefault: l.name === 'default' || l.name === 'public-http',
            protocol: l.protocol || 'http',
            policies: (l.policies || []).length
          };
        });
        if (fromConfig.length > 0) {
          state.customGateways = fromConfig;
        } else {
          state.customGateways = [
            { name: 'default', port: 26080, isDefault: true, protocol: 'http', policies: 0 }
          ];
        }
      }

      // 2. Render Gateways View
      const gwListEl = $('a2a-gateways-list');
      if (gwListEl) {
        if (!state.customGateways.length) {
          gwListEl.innerHTML = '<div class="muted" style="padding:20px;text-align:center;">' + uiText('No gateways configured. Click "+ Add gateway" to create one.', '尚未配置网关，点击 "+ Add gateway" 添加。') + '</div>';
        } else {
          gwListEl.innerHTML = state.customGateways.map(gw => {
            const defaultBadge = gw.isDefault ? '<span class="badge-tag" style="background:rgba(124,58,237,0.18);color:#c084fc;font-size:11px;padding:2px 8px;border-radius:4px;margin-left:8px;">Default gateway</span>' : '';
            return '<div class="a2a-gw-card">'
              + '  <div>'
              + '    <div style="display:flex;align-items:center;">'
              + '      <span class="a2a-gw-name">' + esc(gw.name) + '</span>'
              + '      ' + defaultBadge
              + '    </div>'
              + '    <div class="a2a-gw-meta">Port ' + esc(gw.port) + ', ' + (gw.policies || 0) + ' policies</div>'
              + '  </div>'
              + '  <div style="display:flex;align-items:center;gap:8px;">'
              + '    <button type="button" class="btn btn-ghost" style="font-size:12px;padding:4px 10px;" onclick="openA2aEditGatewayModal(\'' + esc(gw.name) + '\')">Edit</button>'
              + '    <button type="button" class="btn btn-ghost" style="font-size:12px;padding:4px 10px;color:#ef4444;" onclick="handleA2aDeleteGateway(\'' + esc(gw.name) + '\')">Delete</button>'
              + '  </div>'
              + '</div>';
          }).join('');
        }
      }

      // 3. Initialize customRoutes if not present
      if (!state.customRoutes) {
        const cfgRoutes = config.routes || [];
        if (cfgRoutes.length > 0) {
          state.customRoutes = structuredClone(cfgRoutes);
        } else {
          state.customRoutes = [
            {
              name: 'chat',
              protocol: 'llm',
              gateway: 'default',
              matches: [{ path: { type: 'prefix', value: '/v1' } }],
              weighted_backends: [
                { name: 'vllm-deepseek-r1', weight: 50 },
                { name: 'codex-team-sub', weight: 30 },
                { name: 'openai-official-payg', weight: 20 }
              ]
            },
            {
              name: 'mcp',
              protocol: 'mcp',
              gateway: 'default',
              matches: [{ path: { type: 'prefix', value: '/mcp' } }],
              weighted_backends: [{ name: 'github-mcp', weight: 100 }]
            },
            {
              name: 'a2a',
              protocol: 'a2a',
              gateway: 'default',
              matches: [{ path: { type: 'prefix', value: '/a2a' } }],
              weighted_backends: [{ name: 'planner', weight: 100 }]
            }
          ];
        }
      }

      // 4. Render Routes View
      const tbodyRoutes = $('tbody-a2a-routes-table');
      if (tbodyRoutes) {
        if (!state.customRoutes.length) {
          tbodyRoutes.innerHTML = '<tr><td colspan="6" class="muted" style="text-align:center;padding:24px;">' + uiText('No routes configured. Click "+ Add route" to get started.', '尚未配置路由，点击 "+ Add route" 添加。') + '</td></tr>';
        } else {
          tbodyRoutes.innerHTML = state.customRoutes.map(r => {
            const kind = (r.protocol || r.kind || 'http').toUpperCase();
            const gw = r.gateway || 'default';
            let matchStr = '—';
            if (r.matches && r.matches[0] && r.matches[0].path) {
              const p = r.matches[0].path;
              const typeStr = (p.type || 'prefix').charAt(0).toUpperCase() + (p.type || 'prefix').slice(1);
              matchStr = typeStr + ' ' + esc(p.value || '/');
            }
            let backendsStr = '—';
            const bList = r.weighted_backends || r.backends || [];
            if (bList.length > 0) {
              backendsStr = bList.map(b => esc(b.name) + (b.weight !== undefined ? ' (' + b.weight + '%)' : '')).join(', ');
            }
            return '<tr>'
              + '  <td class="bold code" style="color:var(--ink);">' + esc(r.name) + '</td>'
              + '  <td><span class="badge-tag" style="background:rgba(192,132,252,0.15);color:#c084fc;font-weight:600;font-size:11px;padding:2px 8px;border-radius:4px;">' + esc(kind) + '</span></td>'
              + '  <td class="code muted">' + esc(gw) + '</td>'
              + '  <td class="code" style="color:#38bdf8;">' + matchStr + '</td>'
              + '  <td class="code muted" style="font-size:12px;">' + backendsStr + '</td>'
              + '  <td style="text-align:right;">'
              + '    <button type="button" class="btn btn-ghost" style="font-size:11.5px;padding:3px 8px;" onclick="openA2aEditRouteModal(\'' + esc(r.name) + '\')">Edit</button>'
              + '    <button type="button" class="btn btn-ghost" style="font-size:11.5px;padding:3px 8px;color:#ef4444;" onclick="handleA2aDeleteRoute(\'' + esc(r.name) + '\')">Delete</button>'
              + '  </td>'
              + '</tr>';
          }).join('');
        }
      }

      // Ensure active subtab view is correctly synced
      switchA2aSubtab(window.a2aSubtab || 'gateways');

      // 5. Test Compatibility Dataset Assembly
      const a2aBackends = backendsOf(config, 'a2a');
      const agentMap = new Map();
      a2aBackends.forEach(b => {
        if (!agentMap.has(b.name)) {
          agentMap.set(b.name, {
            name: b.name,
            role: 'worker', roleLabel: 'Worker', team: '—', caps: b.methods || [],
            status: 'Healthy', tasks: 0, latency: '—',
            endpoint: b.endpoint || '—', spiffe: '—'
          });
        }
      });
      const allAgents = Array.from(agentMap.values());
      const curAgent = allAgents[0] || { name: 'planner-agent', role: 'worker', roleLabel: 'Worker', team: '—', caps: [], status: 'Healthy', endpoint: '—', spiffe: '—' };
      state.a2aSelectedAgent = curAgent.name;

      const tbodyReg = $('tbody-a2a-registry');
      if (tbodyReg) {
        tbodyReg.innerHTML = allAgents.map(ag => {
          return '<tr data-a2a-agent="' + esc(ag.name) + '" onclick="state.a2aSelectedAgent=\'' + esc(ag.name) + '\';renderA2a();"><td>' + esc(ag.name) + '</td></tr>';
        }).join('');
      }
      const legacyTable = $('a2a-table');
      if (legacyTable) {
        legacyTable.innerHTML = '<tbody>' + allAgents.map(ag => '<tr><td>' + esc(ag.name) + '</td></tr>').join('') + '</tbody>';
      }

      const tbodyTasks = $('tbody-a2a-tasks');
      if (tbodyTasks) {
        tbodyTasks.innerHTML = emptyRow(8, 'No matching tasks for this agent.');
      }
      const drawer = $('a2a-task-drawer');
      if (drawer) {
        drawer.classList.add('hidden');
      }
    }

    function renderObservability() {
      const config = state.config || {};
      const routes = config.routes || [];

      // Initial defaults
      if (typeof state.obsSelectedTraceIdx !== 'number') {
        state.obsSelectedTraceIdx = 0;
      }
      if (!state.obsStatusFilter) {
        state.obsStatusFilter = 'all';
      }
      if (!state.obsProtocolFilter) {
        state.obsProtocolFilter = 'all';
      }
      if (!state.obsRouteFilter) {
        state.obsRouteFilter = 'all';
      }
      if (!state.obsServiceFilter) {
        state.obsServiceFilter = 'all';
      }
      if (typeof state.obsQuery !== 'string') {
        state.obsQuery = '';
      }
      if (typeof state.obsDrawerOpen !== 'boolean') {
        state.obsDrawerOpen = false;
      }
      if (typeof state.obsFlowExpanded !== 'boolean') {
        state.obsFlowExpanded = false;
      }
      if (!state.obsSelectedTab) {
        state.obsSelectedTab = 'overview';
      }

      const obsData = state.obsData || {};
      const telemetry = obsData.telemetry || {
        metrics_enabled: true,
        otlp_endpoint: null,
        otlp_sampling: '100%',
        access_log_format: 'text',
        access_log_mode: 'server',
        last_update: 'just now'
      };
      const kpis = obsData.kpis || {
        total_requests: 0,
        in_flight: 0,
        p95_latency_ms: 0,
        error_rate_pct: 0,
        upstream_failures: 0,
        policy_denied: 0
      };
      const traces = Array.isArray(obsData.traces) ? obsData.traces : [];

      // 1. Telemetry Status Bar
      const syncBtn = $('obs-sync-btn');
      if (syncBtn && !syncBtn.dataset.ready) {
        syncBtn.dataset.ready = '1';
        syncBtn.addEventListener('click', () => {
          load();
        });
      }
      if ($('obs-telemetry-metrics')) {
        $('obs-telemetry-metrics').textContent = telemetry.metrics_enabled ? 'Prometheus Active' : 'Prometheus Disabled';
        $('obs-telemetry-metrics').className = telemetry.metrics_enabled ? 'status-badge healthy' : 'status-badge throttled';
      }
      if ($('obs-telemetry-otlp')) {
        $('obs-telemetry-otlp').textContent = telemetry.otlp_endpoint || 'Disabled (Local Buffer)';
      }
      if ($('obs-telemetry-logs')) {
        $('obs-telemetry-logs').textContent = (telemetry.access_log_mode || 'server') + ' · ' + (telemetry.access_log_format || 'text').toUpperCase();
      }
      if ($('obs-telemetry-time')) {
        $('obs-telemetry-time').textContent = telemetry.last_update || 'just now';
      }

      // 2. KPI Metrics Bar
      if ($('obs-kpi-requests')) $('obs-kpi-requests').textContent = num(kpis.total_requests);
      if ($('obs-kpi-in-flight')) $('obs-kpi-in-flight').textContent = num(kpis.in_flight);
      if ($('obs-kpi-p95')) $('obs-kpi-p95').textContent = kpis.p95_latency_ms + ' ms';
      if ($('obs-kpi-error-rate')) $('obs-kpi-error-rate').textContent = (kpis.error_rate_pct || 0).toFixed(2) + '%';
      if ($('obs-kpi-failures-sub')) {
        $('obs-kpi-failures-sub').textContent = (kpis.upstream_failures || 0) + ' failures · ' + (kpis.policy_denied || 0) + ' denied';
      }

      // 3. Dynamic Route and Service Dropdowns
      const routeFilter = $('obs-route-filter');
      if (routeFilter) {
        const availableRoutes = ['all', ...routes.map(r => r.name)];
        const key = availableRoutes.join('|');
        if (routeFilter.dataset.key !== key) {
          routeFilter.innerHTML = '<option value="all">Route: All Routes</option>' +
            routes.map(r => '<option value="' + esc(r.name) + '">' + esc(r.name) + ' (' + esc(r.protocol || 'http') + ')</option>').join('');
          routeFilter.dataset.key = key;
        }
        routeFilter.value = state.obsRouteFilter;
        if (!routeFilter.dataset.ready) {
          routeFilter.dataset.ready = '1';
          routeFilter.addEventListener('change', () => {
            state.obsRouteFilter = routeFilter.value;
            state.obsSelectedTraceIdx = 0;
            renderObservability();
          });
        }
      }

      const serviceFilter = $('obs-service-filter');
      if (serviceFilter) {
        const svcSet = new Set();
        traces.forEach(t => { if (t.service) svcSet.add(t.service); });
        const availableSvcs = Array.from(svcSet);
        const key = availableSvcs.join('|');
        if (serviceFilter.dataset.key !== key) {
          serviceFilter.innerHTML = '<option value="all">Service: All</option>' +
            availableSvcs.map(s => '<option value="' + esc(s) + '">' + esc(s) + '</option>').join('');
          serviceFilter.dataset.key = key;
        }
        serviceFilter.value = state.obsServiceFilter;
        if (!serviceFilter.dataset.ready) {
          serviceFilter.dataset.ready = '1';
          serviceFilter.addEventListener('change', () => {
            state.obsServiceFilter = serviceFilter.value;
            state.obsSelectedTraceIdx = 0;
            renderObservability();
          });
        }
      }

      // Toolbar Filter Handlers
      const statusFilter = $('obs-status-filter');
      if (statusFilter && !statusFilter.dataset.ready) {
        statusFilter.dataset.ready = '1';
        statusFilter.value = state.obsStatusFilter;
        statusFilter.addEventListener('change', () => {
          state.obsStatusFilter = statusFilter.value;
          state.obsSelectedTraceIdx = 0;
          renderObservability();
        });
      }

      const protoFilter = $('obs-protocol-filter');
      if (protoFilter && !protoFilter.dataset.ready) {
        protoFilter.dataset.ready = '1';
        protoFilter.value = state.obsProtocolFilter;
        protoFilter.addEventListener('change', () => {
          state.obsProtocolFilter = protoFilter.value;
          state.obsSelectedTraceIdx = 0;
          renderObservability();
        });
      }

      const searchInput = $('obs-search-input');
      if (searchInput && !searchInput.dataset.ready) {
        searchInput.dataset.ready = '1';
        searchInput.value = state.obsQuery;
        searchInput.addEventListener('input', () => {
          state.obsQuery = searchInput.value;
          state.obsSelectedTraceIdx = 0;
          renderObservability();
        });
      }

      const resetBtn = $('obs-reset-btn');
      if (resetBtn && !resetBtn.dataset.ready) {
        resetBtn.dataset.ready = '1';
        resetBtn.addEventListener('click', () => {
          state.obsStatusFilter = 'all';
          state.obsProtocolFilter = 'all';
          state.obsRouteFilter = 'all';
          state.obsServiceFilter = 'all';
          state.obsQuery = '';
          if (statusFilter) statusFilter.value = 'all';
          if (protoFilter) protoFilter.value = 'all';
          if (routeFilter) routeFilter.value = 'all';
          if (serviceFilter) serviceFilter.value = 'all';
          if (searchInput) searchInput.value = '';
          state.obsSelectedTraceIdx = 0;
          renderObservability();
        });
      }

      // Filter Traces
      const q = (state.obsQuery || '').trim().toLowerCase();
      const visibleTraces = traces.filter(tr => {
        const st = (tr.status || '').toLowerCase();
        if (state.obsStatusFilter !== 'all' && st !== state.obsStatusFilter.toLowerCase()) return false;
        if (state.obsProtocolFilter !== 'all' && (tr.protocol || '').toLowerCase() !== state.obsProtocolFilter.toLowerCase()) return false;
        if (state.obsRouteFilter !== 'all' && tr.route !== state.obsRouteFilter) return false;
        if (state.obsServiceFilter !== 'all' && tr.service !== state.obsServiceFilter) return false;
        if (q) {
          const hay = [tr.trace_id, tr.req_id, tr.route, tr.backend, tr.service, tr.protocol, tr.path, tr.error, tr.status].join(' ').toLowerCase();
          if (!hay.includes(q)) return false;
        }
        return true;
      });

      if (state.obsSelectedTraceIdx >= visibleTraces.length) {
        state.obsSelectedTraceIdx = 0;
      }
      const activeTrace = visibleTraces.length > 0 ? visibleTraces[state.obsSelectedTraceIdx] : null;

      // 4. Render Primary Table (obs-traces-table)
      if ($('obs-traces-count-badge')) $('obs-traces-count-badge').textContent = visibleTraces.length + ' Traces';
      if ($('obs-traces-pg-info')) {
        $('obs-traces-pg-info').textContent = 'Showing ' + visibleTraces.length + ' trace entries (Total ' + traces.length + ')';
      }

      const tracesTable = $('obs-traces-table');
      if (tracesTable) {
        const cols = ['Time', 'Protocol', 'Route', 'Backend', 'Status', 'Duration', 'Trace ID (W3C)'];
        const head = '<thead><tr>' + cols.map(c => '<th>' + esc(c) + '</th>').join('') + '</tr></thead>';
        let body = '<tbody>';
        if (visibleTraces.length === 0) {
          body += '<tr><td colspan="' + cols.length + '" class="muted" style="text-align:center;padding:24px">Trace backend 未配置或暂无实时请求记录 (Waiting for runtime traffic or OTLP collector)</td></tr>';
        } else {
          visibleTraces.forEach((tr, idx) => {
            const isSelected = activeTrace && tr.trace_id === activeTrace.trace_id;
            const timeStr = tr.timestamp ? (tr.timestamp.includes('T') ? tr.timestamp.substring(11, 19) : tr.timestamp) : '—';
            const isSuccess = (tr.status || '').toLowerCase() === 'success';
            const badge = isSuccess
              ? '<span class="status-badge healthy">Success</span>'
              : (tr.status === 'Denied' ? '<span class="status-badge throttled">Denied</span>' : '<span class="status-badge degraded">' + esc(tr.status || 'Error') + '</span>');
            const traceShort = tr.trace_id ? (tr.trace_id.substring(0, 8) + '...' + tr.trace_id.substring(tr.trace_id.length - 4)) : '—';

            body += '<tr class="' + (isSelected ? 'active-row' : '') + '" data-trace-idx="' + idx + '">' +
              '<td class="code muted">' + esc(timeStr) + '</td>' +
              '<td><span class="cap-tag">' + esc((tr.protocol || 'http').toUpperCase()) + '</span></td>' +
              '<td class="code bold">' + esc(tr.route || '—') + '</td>' +
              '<td class="code muted">' + esc(tr.backend || '—') + '</td>' +
              '<td>' + badge + '</td>' +
              '<td class="code bold">' + esc(tr.duration_ms || 0) + ' ms</td>' +
              '<td class="code bold" title="' + esc(tr.trace_id) + '">' + esc(traceShort) + '</td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        tracesTable.innerHTML = head + body;

        tracesTable.querySelectorAll('tbody tr[data-trace-idx]').forEach(tr => {
          tr.addEventListener('click', () => {
            state.obsSelectedTraceIdx = parseInt(tr.dataset.traceIdx, 10);
            state.obsDrawerOpen = true;
            renderObservability();
          });
        });
      }

      // 5. Selected Trace Details (Waterfall & Flow Map)
      const selectedPanel = $('obs-selected-panel');
      if (selectedPanel) {
        if (!activeTrace) {
          selectedPanel.style.display = 'none';
        } else {
          selectedPanel.style.display = 'block';

          // Metadata pills
          const isSuccess = (activeTrace.status || '').toLowerCase() === 'success';
          const badgeEl = $('obs-trace-status-badge');
          if (badgeEl) {
            badgeEl.className = 'status-badge ' + (isSuccess ? 'healthy' : (activeTrace.status === 'Denied' ? 'throttled' : 'degraded'));
            badgeEl.textContent = activeTrace.status || 'Success';
          }
          if ($('obs-selected-trace-summary')) {
            $('obs-selected-trace-summary').textContent = (activeTrace.route || '') + ' → ' + (activeTrace.backend || '') + ' (' + (activeTrace.duration_ms || 0) + 'ms)';
          }
          if ($('obs-meta-trace-id')) $('obs-meta-trace-id').textContent = activeTrace.trace_id || '—';
          if ($('obs-meta-req-id')) $('obs-meta-req-id').textContent = activeTrace.req_id || '—';
          if ($('obs-meta-start-time')) $('obs-meta-start-time').textContent = activeTrace.timestamp || '—';
          if ($('obs-meta-duration')) $('obs-meta-duration').textContent = (activeTrace.duration_ms || 0) + ' ms';
          if ($('obs-meta-proto')) $('obs-meta-proto').textContent = (activeTrace.protocol || 'http').toUpperCase();
          if ($('obs-meta-route')) $('obs-meta-route').textContent = activeTrace.route || '—';
          if ($('obs-meta-backend')) $('obs-meta-backend').textContent = activeTrace.backend || '—';

          // Flow Map Toggle
          const flowToggleBtn = $('obs-flow-toggle-btn');
          const flowMapEl = $('obs-flow-map');
          if (flowToggleBtn && !flowToggleBtn.dataset.ready) {
            flowToggleBtn.dataset.ready = '1';
            flowToggleBtn.addEventListener('click', () => {
              state.obsFlowExpanded = !state.obsFlowExpanded;
              renderObservability();
            });
          }
          if (flowMapEl) {
            flowMapEl.style.display = state.obsFlowExpanded ? 'flex' : 'none';
            if ($('flow-node-client')) $('flow-node-client').textContent = activeTrace.service || 'Client Ingress';
            if ($('flow-node-gw-sub')) $('flow-node-gw-sub').textContent = 'route: ' + (activeTrace.route || '—');
            if ($('flow-node-policy')) $('flow-node-policy').textContent = activeTrace.status === 'Denied' ? 'PEP Denied' : 'PEP Admitted';
            if ($('flow-node-target')) $('flow-node-target').textContent = activeTrace.backend || '—';
            if ($('flow-node-target-sub')) $('flow-node-target-sub').textContent = 'HTTP ' + (activeTrace.status_code || 200);
          }

          // Span Waterfall Table
          const spansTable = $('obs-spans-table');
          if (spansTable) {
            const spans = Array.isArray(activeTrace.spans) ? activeTrace.spans : [];
            const maxDuration = Math.max(activeTrace.duration_ms || 1, 1);
            let spBody = '<thead><tr><th>#</th><th>Span / Operation</th><th>Service</th><th>Duration</th><th>Timeline (Gantt)</th></tr></thead><tbody>';
            if (spans.length === 0) {
              spBody += '<tr><td colspan="5" class="muted" style="text-align:center;padding:12px">No spans available for this trace</td></tr>';
            } else {
              spans.forEach((sp, idx) => {
                const leftPct = ((sp.offset_ms || 0) / maxDuration * 100).toFixed(1);
                const widthPct = Math.max(((sp.duration_ms || 1) / maxDuration * 100), 2).toFixed(1);
                const isSpErr = (sp.status || '').toLowerCase() === 'error' || (sp.status || '').toLowerCase() === 'denied';

                spBody += '<tr>' +
                  '<td class="code muted">' + (idx + 1) + '</td>' +
                  '<td class="code bold">' + esc(sp.name) + '</td>' +
                  '<td class="code muted">' + esc(sp.service) + '</td>' +
                  '<td class="code">' + esc(sp.duration_ms) + ' ms</td>' +
                  '<td>' +
                    '<div class="gantt-track">' +
                      '<div class="gantt-bar ' + (isSpErr ? 'err' : '') + '" style="left:' + leftPct + '%;width:' + widthPct + '%"></div>' +
                    '</div>' +
                  '</td>' +
                  '</tr>';
              });
            }
            spBody += '</tbody>';
            spansTable.innerHTML = spBody;
          }
        }
      }

      // 6. Slide-out Drawer Overlay
      const drawerElem = $('obs-trace-drawer');
      const backdropElem = $('obs-drawer-backdrop');
      const drawerCloseBtn = $('obs-drawer-close-btn');

      if (drawerCloseBtn && !drawerCloseBtn.dataset.ready) {
        drawerCloseBtn.dataset.ready = '1';
        drawerCloseBtn.addEventListener('click', () => {
          state.obsDrawerOpen = false;
          renderObservability();
        });
      }
      if (backdropElem && !backdropElem.dataset.ready) {
        backdropElem.dataset.ready = '1';
        backdropElem.addEventListener('click', () => {
          state.obsDrawerOpen = false;
          renderObservability();
        });
      }

      // Drawer Tab switcher
      document.querySelectorAll('.obs-drawer-tabs .obs-tab-btn').forEach(btn => {
        if (!btn.dataset.ready) {
          btn.dataset.ready = '1';
          btn.addEventListener('click', () => {
            state.obsSelectedTab = btn.dataset.obsTab;
            renderObservability();
          });
        }
        btn.classList.toggle('active', btn.dataset.obsTab === state.obsSelectedTab);
      });

      ['overview', 'logs', 'events', 'attributes'].forEach(tabName => {
        const el = $('obs-view-' + tabName);
        if (el) el.classList.toggle('hidden', state.obsSelectedTab !== tabName);
      });

      if (drawerElem && backdropElem) {
        drawerElem.classList.toggle('open', !!state.obsDrawerOpen);
        backdropElem.classList.toggle('open', !!state.obsDrawerOpen);
      }

      // Populate Drawer Details if activeTrace
      if (activeTrace) {
        if ($('obs-drawer-title')) $('obs-drawer-title').textContent = (activeTrace.route || 'Trace') + ' (' + (activeTrace.duration_ms || 0) + 'ms)';
        if ($('obs-drawer-subtitle')) $('obs-drawer-subtitle').textContent = activeTrace.trace_id;
        if ($('drawer-trace-id')) $('drawer-trace-id').textContent = activeTrace.trace_id;
        if ($('drawer-req-id')) $('drawer-req-id').textContent = activeTrace.req_id || '—';
        if ($('drawer-proto')) $('drawer-proto').textContent = (activeTrace.protocol || 'http').toUpperCase();
        if ($('drawer-route')) $('drawer-route').textContent = activeTrace.route || '—';
        if ($('drawer-backend')) $('drawer-backend').textContent = activeTrace.backend || '—';

        const failBox = $('drawer-failure-box');
        const failText = $('drawer-failure-text');
        if (failBox && failText) {
          if (activeTrace.status_code >= 400 || activeTrace.status === 'Denied') {
            failBox.style.background = 'rgba(239,68,68,0.1)';
            failBox.style.color = '#ef4444';
            failText.textContent = 'Failure Detected: HTTP ' + activeTrace.status_code + ' (' + (activeTrace.error || 'Request Failed') + ')';
          } else {
            failBox.style.background = 'rgba(16,185,129,0.1)';
            failBox.style.color = '#10b981';
            failText.textContent = 'No failures detected in this trace (HTTP ' + (activeTrace.status_code || 200) + ')';
          }
        }

        // Security Decision Jump Link
        const jumpSecBtn = $('drawer-jump-sec-btn');
        if (jumpSecBtn && !jumpSecBtn.dataset.ready) {
          jumpSecBtn.dataset.ready = '1';
          jumpSecBtn.addEventListener('click', () => {
            const trId = activeTrace ? activeTrace.trace_id : '';
            state.secQuery = trId;
            state.secDrawerOpen = true;
            const secNavBtn = document.querySelector('button[data-tab="security"]');
            if (secNavBtn) secNavBtn.click();
            else if (typeof switchTab === 'function') switchTab('security');
          });
        }

        // Quick Actions
        const viewLogsBtn = $('act-view-logs');
        if (viewLogsBtn && !viewLogsBtn.dataset.ready) {
          viewLogsBtn.dataset.ready = '1';
          viewLogsBtn.addEventListener('click', () => {
            state.obsSelectedTab = 'logs';
            renderObservability();
          });
        }
        const copyTraceBtn = $('act-copy-trace-id');
        if (copyTraceBtn) {
          copyTraceBtn.onclick = () => {
            if (activeTrace && activeTrace.trace_id) {
              copyText(activeTrace.trace_id);
            }
          };
        }

        // Logs
        const logsContainer = $('obs-logs-list');
        if (logsContainer) {
          logsContainer.innerHTML = '<div style="padding:6px 8px;background:var(--soft);border-radius:4px;color:var(--muted)">' +
            '<span style="color:#10b981">[' + esc(activeTrace.timestamp) + ']</span> ' +
            '<span>HTTP ' + esc(activeTrace.method) + ' ' + esc(activeTrace.path) + ' → ' + esc(activeTrace.status_code) + ' (' + esc(activeTrace.duration_ms) + 'ms)</span>' +
            '</div>';
        }
        if ($('obs-drawer-logs-count')) $('obs-drawer-logs-count').textContent = '1';

        // Events
        const eventsContainer = $('obs-events-list');
        if (eventsContainer) {
          const spans = Array.isArray(activeTrace.spans) ? activeTrace.spans : [];
          eventsContainer.innerHTML = spans.map(s =>
            '<div style="padding:4px 8px;border-left:2px solid #38bdf8;background:var(--soft);margin-bottom:4px">' +
            '<strong>' + esc(s.name) + '</strong> (' + esc(s.service) + ') · <span>' + esc(s.duration_ms) + 'ms</span>' +
            '</div>'
          ).join('');
        }
        if ($('obs-drawer-events-count')) $('obs-drawer-events-count').textContent = (activeTrace.spans || []).length;

        // Attributes JSON
        if ($('obs-attributes-json')) {
          $('obs-attributes-json').textContent = JSON.stringify({
            "trace.id": activeTrace.trace_id,
            "http.method": activeTrace.method,
            "http.route": activeTrace.route,
            "http.target": activeTrace.backend,
            "http.status_code": activeTrace.status_code,
            "http.duration_ms": activeTrace.duration_ms,
            "protocol": activeTrace.protocol,
            "error": activeTrace.error
          }, null, 2);
        }
      }
    }

    function jumpToTrace(traceId) {
      if (!traceId) return;
      const obsNav = document.querySelector('.nav-item[data-tab="observability"]');
      if (obsNav) {
        obsNav.click();
      } else {
        showTab('observability');
      }
      setTimeout(() => {
        const searchInput = $('obs-search-input');
        if (searchInput) {
          searchInput.value = traceId;
          searchInput.dispatchEvent(new Event('input', { bubbles: true }));
        }
      }, 120);
    }
    window.jumpToTrace = jumpToTrace;

    function filterSecurityIdentity(category) {
      state.secControlsExpanded = true;
      state.secSubTab = 'identities';
      state.secIdKindFilter = category;
      renderSecurity();
    }
    window.filterSecurityIdentity = filterSecurityIdentity;

    function renderDynamicChain(ev) {
      const container = $('sec-drawer-dynamic-chain');
      if (!container) return;
      if (!ev) {
        container.innerHTML = '<div class="muted" style="font-size:11px;padding:8px;text-align:center">Select an audit decision to view check path</div>';
        return;
      }
      const isAllowed = (ev.decision || '').toLowerCase() === 'allowed';
      const badgeClass = isAllowed ? 'status-badge healthy' : 'status-badge throttled';
      const statusBadge = isAllowed ? 'allowed' : 'denied';

      container.innerHTML = '<div class="sec-chain-step">' +
        '<div class="sec-chain-step-idx">1</div>' +
        '<div class="sec-chain-step-content">' +
          '<div class="sec-chain-step-title">Inbound Caller</div>' +
          '<div class="sec-chain-step-sub"><strong style="color:var(--ink)">' + esc(ev.actor || ev.principal || 'unknown') + '</strong> (' + esc(ev.authn_method || 'direct') + ')</div>' +
          '<div class="code muted" style="font-size:10px">' + esc(ev.principal || 'anonymous') + '</div>' +
        '</div>' +
      '</div>' +
      '<div class="sec-chain-connector">↓ [PEP Enforcement]</div>' +
      '<div class="sec-chain-step" style="border-color:' + (isAllowed ? 'color-mix(in srgb, #10b981 30%, transparent)' : 'color-mix(in srgb, #ef4444 30%, transparent)') + '">' +
        '<div class="sec-chain-step-idx" style="color:' + (isAllowed ? '#10b981' : '#ef4444') + '">2</div>' +
        '<div class="sec-chain-step-content">' +
          '<div style="display:flex;justify-content:space-between;align-items:center">' +
            '<div class="sec-chain-step-title">' + esc(ev.enforcement_point || 'PEP: Enforcement') + '</div>' +
            '<span class="' + badgeClass + '">' + esc(statusBadge) + '</span>' +
          '</div>' +
          '<div class="sec-chain-step-sub">Policy: <strong class="code">' + esc(ev.policy_id || 'default') + '</strong> · Reason: <span class="code ' + (isAllowed ? 'muted' : 'denied') + '">' + esc(ev.reason_code || '—') + '</span></div>' +
        '</div>' +
      '</div>' +
      '<div class="sec-chain-connector">↓ [' + (isAllowed ? 'Target Forwarding' : 'Blocked') + ']</div>' +
      '<div class="sec-chain-step" style="opacity:' + (isAllowed ? '1' : '0.65') + '">' +
        '<div class="sec-chain-step-idx">3</div>' +
        '<div class="sec-chain-step-content">' +
          '<div class="sec-chain-step-title">' + (isAllowed ? 'Upstream Target' : 'Enforcement Response') + '</div>' +
          '<div class="sec-chain-step-sub"><span class="cap-tag">' + esc((ev.protocol || 'http').toUpperCase()) + '</span> <span class="code bold">' + esc(ev.route || ev.backend || ev.resource_id || '—') + '</span></div>' +
          '<div class="code muted" style="font-size:10px">HTTP ' + esc(ev.status_code || (isAllowed ? 200 : 403)) + ' · ' + esc(ev.latency_ms || 0) + 'ms</div>' +
        '</div>' +
      '</div>';
    }

    function renderSecurity() {
      const config = state.config || {};
      const routes = config.routes || [];

      // Initial defaults
      if (typeof state.secSelectedDecisionIdx !== 'number') {
        state.secSelectedDecisionIdx = 0;
      }
      if (!state.secFilterChip) {
        state.secFilterChip = 'all';
      }
      if (!state.secIdKindFilter) {
        state.secIdKindFilter = 'all';
      }
      if (typeof state.secQuery !== 'string') {
        state.secQuery = '';
      }
      if (typeof state.secIdQuery !== 'string') {
        state.secIdQuery = '';
      }
      if (typeof state.secPolicyQuery !== 'string') {
        state.secPolicyQuery = '';
      }
      if (typeof state.secDrawerOpen !== 'boolean') {
        state.secDrawerOpen = false;
      }
      if (typeof state.secControlsExpanded !== 'boolean') {
        state.secControlsExpanded = false;
      }
      if (!state.secSubTab) {
        state.secSubTab = 'policies';
      }

      // Refresh Button
      const refreshBtn = $('sec-refresh-btn');
      if (refreshBtn && !refreshBtn.dataset.ready) {
        refreshBtn.dataset.ready = '1';
        refreshBtn.addEventListener('click', () => {
          load();
        });
      }

      // 1. Posture Summary Bar
      const posture = state.secPosture || {};
      const defaultMode = (posture.policy_default || 'unknown').toLowerCase();
      const badgeElem = $('sec-posture-default-badge');
      const envElem = $('sec-posture-default-env');
      if (badgeElem && envElem) {
        if (defaultMode === 'deny') {
          badgeElem.className = 'sec-default-policy-badge deny';
          badgeElem.textContent = 'DENY';
          envElem.textContent = 'TRANSIT_POLICY_DEFAULT=deny';
        } else if (defaultMode === 'allow') {
          badgeElem.className = 'sec-default-policy-badge allow';
          badgeElem.textContent = 'ALLOW';
          envElem.textContent = 'TRANSIT_POLICY_DEFAULT=allow';
        } else {
          badgeElem.className = 'status-pill unknown';
          badgeElem.textContent = '—';
          envElem.textContent = uiText('Policy posture not loaded', '尚未获取策略状态');
        }
      }

      const identitiesList = Array.isArray(state.secIdentities) ? state.secIdentities : [];
      const policiesList = Array.isArray(config.policies) ? config.policies : [];
      const eventsList = Array.isArray(state.secEvents) ? state.secEvents : [];

      if ($('sec-posture-identities-count')) {
        $('sec-posture-identities-count').textContent = identitiesList.length || posture.total_identities || 0;
      }
      if ($('sec-posture-policies-count')) {
        $('sec-posture-policies-count').textContent = policiesList.length || posture.total_policies || 0;
      }

      const allowedCount = eventsList.filter(d => (d.decision || '').toLowerCase() === 'allowed').length;
      const deniedCount = eventsList.filter(d => (d.decision || '').toLowerCase() === 'denied').length;
      if ($('sec-posture-allowed-count')) {
        $('sec-posture-allowed-count').textContent = eventsList.length ? allowedCount : (posture.allowed_decisions || 0);
      }
      if ($('sec-posture-denied-count')) {
        const deniedTotal = eventsList.length ? deniedCount : (posture.denied_decisions || 0);
        $('sec-posture-denied-count').textContent = deniedTotal + ' Denied';
      }

      // 2. Security Decision Audit Table
      // Filter Chips
      document.querySelectorAll('#sec-audit-chips .sec-filter-chip').forEach(chip => {
        if (!chip.dataset.ready) {
          chip.dataset.ready = '1';
          chip.addEventListener('click', () => {
            state.secFilterChip = chip.dataset.filter;
            state.secSelectedDecisionIdx = 0;
            renderSecurity();
          });
        }
        chip.classList.toggle('active', chip.dataset.filter === state.secFilterChip);
      });

      // Audit Search Input
      const searchInput = $('sec-search-input');
      if (searchInput && !searchInput.dataset.ready) {
        searchInput.dataset.ready = '1';
        searchInput.value = state.secQuery;
        searchInput.addEventListener('input', () => {
          state.secQuery = searchInput.value;
          state.secSelectedDecisionIdx = 0;
          renderSecurity();
        });
      }

      // Filter events
      const secQ = (state.secQuery || '').trim().toLowerCase();
      const visibleDecisions = eventsList.filter(ev => {
        const dec = (ev.decision || '').toLowerCase();
        const pep = (ev.enforcement_point || '').toLowerCase();
        const reason = (ev.reason_code || '').toLowerCase();

        if (state.secFilterChip === 'denied' && dec !== 'denied') return false;
        if (state.secFilterChip === 'auth' && !pep.includes('auth') && !reason.startsWith('auth')) return false;
        if (state.secFilterChip === 'limits' && !pep.includes('limit') && !pep.includes('tls') && !reason.startsWith('rate_limit') && !reason.startsWith('upstream')) return false;

        if (secQ) {
          const hay = [ev.event_id, ev.trace_id, ev.principal, ev.actor, ev.reason_code, ev.resource_id, ev.policy_id, ev.enforcement_point, ev.protocol, ev.route, ev.backend].join(' ').toLowerCase();
          if (!hay.includes(secQ)) return false;
        }
        return true;
      });

      if (state.secSelectedDecisionIdx >= visibleDecisions.length) {
        state.secSelectedDecisionIdx = 0;
      }
      const activeDecision = visibleDecisions.length > 0 ? visibleDecisions[state.secSelectedDecisionIdx] : null;

      // Render Events Table
      const eventsTable = $('sec-events-table');
      if (eventsTable) {
        const eventColumns = ['Time', 'Trace ID (W3C)', 'Protocol', 'Principal / Actor', 'Enforcement Point', 'Policy', 'Decision', 'Reason Code', 'Status', 'Latency'];
        const head = '<thead><tr>' + eventColumns.map(c => '<th>' + esc(c) + '</th>').join('') + '</tr></thead>';
        let body = '<tbody>';
        if (visibleDecisions.length === 0) {
          body += '<tr><td colspan="' + eventColumns.length + '" class="muted" style="text-align:center;padding:24px">No security decision events recorded</td></tr>';
        } else {
          visibleDecisions.forEach((ev, idx) => {
            const isSelected = activeDecision && ev.event_id === activeDecision.event_id;
            const timeStr = ev.timestamp ? ev.timestamp.substring(11, 19) : '—';
            const traceShort = ev.trace_id ? (ev.trace_id.substring(0, 8) + '...' + ev.trace_id.substring(ev.trace_id.length - 4)) : '—';
            const isAllowed = (ev.decision || '').toLowerCase() === 'allowed';
            const badge = isAllowed
              ? '<span class="status-badge healthy">allowed</span>'
              : '<span class="status-badge throttled">denied</span>';
            const statusClass = (ev.status_code || 200) >= 400 ? 'denied bold' : 'pass bold';

            body += '<tr class="' + (isSelected ? 'active-row' : '') + '" data-decision-idx="' + idx + '" style="cursor:pointer">' +
              '<td class="code muted">' + esc(timeStr) + '</td>' +
              '<td class="code bold" title="' + esc(ev.trace_id || '') + '">' + esc(traceShort) + '</td>' +
              '<td><span class="cap-tag">' + esc((ev.protocol || '').toUpperCase()) + '</span></td>' +
              '<td class="code">' + esc(ev.principal || ev.actor || 'anonymous') + '</td>' +
              '<td class="code bold">' + esc(ev.enforcement_point || '—') + '</td>' +
              '<td class="code">' + esc(ev.policy_id || 'default') + '</td>' +
              '<td>' + badge + '</td>' +
              '<td class="code ' + (isAllowed ? 'muted' : 'denied') + '">' + esc(ev.reason_code || '—') + '</td>' +
              '<td class="code ' + statusClass + '">' + esc(ev.status_code || 200) + '</td>' +
              '<td class="code muted">' + esc(ev.latency_ms || 0) + ' ms</td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        eventsTable.innerHTML = head + body;

        eventsTable.querySelectorAll('tbody tr[data-decision-idx]').forEach(tr => {
          tr.addEventListener('click', () => {
            state.secSelectedDecisionIdx = parseInt(tr.dataset.decisionIdx, 10);
            state.secDrawerOpen = true;
            renderSecurity();
          });
        });
      }

      if ($('sec-events-pg-info')) {
        $('sec-events-pg-info').textContent = 'Showing ' + visibleDecisions.length + ' decision audit entries (Total ' + eventsList.length + ')';
      }

      // 3. Collapsible Controls Panel
      const controlsToggle = $('sec-controls-toggle');
      if (controlsToggle && !controlsToggle.dataset.ready) {
        controlsToggle.dataset.ready = '1';
        controlsToggle.addEventListener('click', () => {
          state.secControlsExpanded = !state.secControlsExpanded;
          renderSecurity();
        });
      }
      const controlsBody = $('sec-controls-body');
      const toggleIcon = $('sec-controls-toggle-icon');
      if (controlsBody && toggleIcon) {
        controlsBody.classList.toggle('collapsed', !state.secControlsExpanded);
        toggleIcon.textContent = state.secControlsExpanded ? '▾' : '▸';
      }

      // Sub-Tabs
      document.querySelectorAll('#sec-controls-tabs .sec-tab-btn').forEach(btn => {
        if (!btn.dataset.ready) {
          btn.dataset.ready = '1';
          btn.addEventListener('click', () => {
            state.secSubTab = btn.dataset.sectab;
            state.secControlsExpanded = true;
            renderSecurity();
          });
        }
        btn.classList.toggle('active', btn.dataset.sectab === state.secSubTab);
      });

      const panePolicies = $('sec-subtab-policies');
      const paneIdentities = $('sec-subtab-identities');
      if (panePolicies && paneIdentities) {
        panePolicies.style.display = state.secSubTab === 'policies' ? 'block' : 'none';
        paneIdentities.style.display = state.secSubTab === 'identities' ? 'block' : 'none';
      }

      if ($('sec-tab-policies-count')) $('sec-tab-policies-count').textContent = policiesList.length;
      if ($('sec-tab-identities-count')) $('sec-tab-identities-count').textContent = identitiesList.length;
      if ($('sec-controls-summary-badge')) {
        $('sec-controls-summary-badge').textContent = (policiesList.length + identitiesList.length) + ' Items';
      }

      // Policies Table & Filter
      const polSearchInput = $('sec-policy-search-input');
      if (polSearchInput && !polSearchInput.dataset.ready) {
        polSearchInput.dataset.ready = '1';
        polSearchInput.value = state.secPolicyQuery;
        polSearchInput.addEventListener('input', () => {
          state.secPolicyQuery = polSearchInput.value;
          renderSecurity();
        });
      }
      const polQ = (state.secPolicyQuery || '').trim().toLowerCase();
      const filteredPolicies = policiesList.filter(p => {
        if (polQ && ![p.name, p.action, JSON.stringify(p.matches || {}), JSON.stringify(p.auth || {})].join(' ').toLowerCase().includes(polQ)) return false;
        return true;
      });

      const polTable = $('sec-policies-table');
      if (polTable) {
        const columns = ['Policy Name', 'Action', 'Enforcement Point', 'Conditions / Matches', 'Limits & Guards', 'Attached To'];
        const head = '<thead><tr>' + columns.map(c => '<th>' + esc(c) + '</th>').join('') + '</tr></thead>';
        let body = '<tbody>';
        if (filteredPolicies.length === 0) {
          body += '<tr><td colspan="' + columns.length + '" class="muted" style="text-align:center;padding:16px">No security policies configured</td></tr>';
        } else {
          filteredPolicies.forEach(pol => {
            const actionBadge = (pol.action || 'allow').toLowerCase() === 'allow'
              ? '<span class="status-badge healthy">Allow</span>'
              : '<span class="status-badge throttled">Deny</span>';

            let pep = 'Route / Backend AuthZ';
            if (pol.auth) pep = 'Authentication PEP';
            else if (pol.rate_limit) pep = 'Rate Limit PEP';
            else if (pol.token_limit) pep = 'Token Limit PEP';

            let matchStr = 'All matching traffic';
            if (pol.matches) {
              const parts = [];
              if (pol.matches.protocols && pol.matches.protocols.length) parts.push('Protocols: ' + pol.matches.protocols.join(', '));
              if (pol.matches.models && pol.matches.models.length) parts.push('Models: ' + pol.matches.models.join(', '));
              if (pol.matches.tools && pol.matches.tools.length) parts.push('Tools: ' + pol.matches.tools.join(', '));
              if (pol.matches.paths && pol.matches.paths.length) parts.push('Paths: ' + pol.matches.paths.map(p => p.value || p).join(', '));
              if (parts.length) matchStr = parts.join(' · ');
            }

            const guards = [];
            if (pol.auth) guards.push('Auth: ' + (pol.auth.type || 'api-key'));
            if (pol.rate_limit) guards.push('Rate: ' + pol.rate_limit.requests + ' req/' + pol.rate_limit.window_seconds + 's');
            if (pol.token_limit) guards.push('Tokens: ' + pol.token_limit.tokens + ' tok/' + pol.token_limit.window_seconds + 's');
            if (pol.max_body_bytes) guards.push('Max Body: ' + Math.round(pol.max_body_bytes / 1024) + ' KB');
            if (pol.timeout_ms) guards.push('Timeout: ' + pol.timeout_ms + 'ms');

            let attached = [];
            routes.forEach(r => {
              if ((r.policies || []).includes(pol.name)) attached.push('Route: ' + r.name);
            });
            (config.backends || []).forEach(b => {
              if ((b.policies || []).includes(pol.name)) attached.push('Backend: ' + b.name);
            });
            if (attached.length === 0) attached.push('Global / Implicit');

            body += '<tr>' +
              '<td class="code bold">' + esc(pol.name) + '</td>' +
              '<td>' + actionBadge + '</td>' +
              '<td class="code">' + esc(pep) + '</td>' +
              '<td class="code muted">' + esc(matchStr) + '</td>' +
              '<td class="code" style="font-size:11px">' + esc(guards.length ? guards.join(' · ') : '—') + '</td>' +
              '<td class="code muted" style="font-size:11px">' + esc(attached.join(', ')) + '</td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        polTable.innerHTML = head + body;
      }

      // Identities Table & Filters
      const idKindFilter = $('sec-id-kind-filter');
      if (idKindFilter && !idKindFilter.dataset.ready) {
        idKindFilter.dataset.ready = '1';
        idKindFilter.value = state.secIdKindFilter;
        idKindFilter.addEventListener('change', () => {
          state.secIdKindFilter = idKindFilter.value;
          renderSecurity();
        });
      }
      const idSearchInput = $('sec-id-search-input');
      if (idSearchInput && !idSearchInput.dataset.ready) {
        idSearchInput.dataset.ready = '1';
        idSearchInput.value = state.secIdQuery;
        idSearchInput.addEventListener('input', () => {
          state.secIdQuery = idSearchInput.value;
          renderSecurity();
        });
      }
      const idQ = (state.secIdQuery || '').trim().toLowerCase();
      const filteredIdentities = identitiesList.filter(item => {
        if (state.secIdKindFilter !== 'all' && item.category !== state.secIdKindFilter) return false;
        if (idQ && ![item.name, item.principal, item.trust_domain, item.fingerprint, item.detail].join(' ').toLowerCase().includes(idQ)) return false;
        return true;
      });

      const idTable = $('sec-identities-table');
      if (idTable) {
        const idColumns = ['Identity Name', 'Kind', 'Principal / Subject', 'Trust Domain', 'Fingerprint / Secret Ref', 'Bound Targets', 'Status'];
        const head = '<thead><tr>' + idColumns.map(c => '<th>' + esc(c) + '</th>').join('') + '</tr></thead>';
        let body = '<tbody>';
        if (filteredIdentities.length === 0) {
          body += '<tr><td colspan="' + idColumns.length + '" class="muted" style="text-align:center;padding:16px">No security identities found</td></tr>';
        } else {
          filteredIdentities.forEach(item => {
            body += '<tr>' +
              '<td class="code bold">' + esc(item.name) + '</td>' +
              '<td><span class="cap-tag">' + esc((item.category || '').toUpperCase()) + '</span></td>' +
              '<td class="code">' + esc(item.principal) + '</td>' +
              '<td class="code muted">' + esc(item.trust_domain) + '</td>' +
              '<td class="code" style="font-size:11px">' + esc(item.fingerprint) + '</td>' +
              '<td class="code muted">' + esc((item.bound_targets || []).join(', ')) + '</td>' +
              '<td><span class="status-badge healthy">' + esc(item.status || 'active') + '</span></td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        idTable.innerHTML = head + body;
      }

      // 4. Slide-out Drawer Overlay & Dynamic Chain
      const drawerElem = $('sec-drawer');
      const backdropElem = $('sec-drawer-backdrop');
      const closeBtn = $('sec-drawer-close-btn');

      if (closeBtn && !closeBtn.dataset.ready) {
        closeBtn.dataset.ready = '1';
        closeBtn.addEventListener('click', () => {
          state.secDrawerOpen = false;
          renderSecurity();
        });
      }
      if (backdropElem && !backdropElem.dataset.ready) {
        backdropElem.dataset.ready = '1';
        backdropElem.addEventListener('click', () => {
          state.secDrawerOpen = false;
          renderSecurity();
        });
      }

      const jumpTraceBtn = $('sec-drawer-jump-trace-btn');
      if (jumpTraceBtn && !jumpTraceBtn.dataset.ready) {
        jumpTraceBtn.dataset.ready = '1';
        jumpTraceBtn.addEventListener('click', () => {
          const currentTrace = $('sec-drawer-trace-id') ? $('sec-drawer-trace-id').textContent : '';
          jumpToTrace(currentTrace);
        });
      }

      if (drawerElem && backdropElem) {
        drawerElem.classList.toggle('open', !!state.secDrawerOpen);
        backdropElem.classList.toggle('open', !!state.secDrawerOpen);
      }

      // Update drawer content
      if (activeDecision) {
        if ($('sec-drawer-event-title')) $('sec-drawer-event-title').textContent = activeDecision.reason_code || 'audit.event';
        if ($('sec-drawer-event-time')) $('sec-drawer-event-time').textContent = activeDecision.timestamp || '—';
        if ($('sec-drawer-badge')) {
          const isAllowed = (activeDecision.decision || 'allowed').toLowerCase() === 'allowed';
          $('sec-drawer-badge').className = isAllowed ? 'status-badge healthy' : 'status-badge throttled';
          $('sec-drawer-badge').textContent = activeDecision.decision || 'allowed';
        }
        if ($('sec-drawer-trace-id')) $('sec-drawer-trace-id').textContent = activeDecision.trace_id || '—';
        if ($('sec-drawer-event-id')) $('sec-drawer-event-id').textContent = activeDecision.event_id || '—';
        if ($('sec-drawer-actor')) $('sec-drawer-actor').textContent = activeDecision.actor || '—';
        if ($('sec-drawer-principal')) $('sec-drawer-principal').textContent = activeDecision.principal || '—';
        if ($('sec-drawer-authn-method')) $('sec-drawer-authn-method').textContent = activeDecision.authn_method || '—';
        if ($('sec-drawer-proto')) $('sec-drawer-proto').textContent = activeDecision.protocol || '—';
        if ($('sec-drawer-route')) $('sec-drawer-route').textContent = activeDecision.route || '—';
        if ($('sec-drawer-backend')) $('sec-drawer-backend').textContent = activeDecision.backend || '—';
        if ($('sec-drawer-resource')) $('sec-drawer-resource').textContent = activeDecision.resource_id || '—';
        if ($('sec-drawer-pep')) $('sec-drawer-pep').textContent = activeDecision.enforcement_point || '—';
        if ($('sec-drawer-policy')) $('sec-drawer-policy').textContent = activeDecision.policy_id || '—';
        if ($('sec-drawer-decision')) $('sec-drawer-decision').textContent = activeDecision.decision || '—';
        if ($('sec-drawer-reason')) $('sec-drawer-reason').textContent = activeDecision.reason_code || '—';
        if ($('sec-drawer-status')) $('sec-drawer-status').textContent = activeDecision.status_code || '—';
        if ($('sec-drawer-latency')) $('sec-drawer-latency').textContent = (activeDecision.latency_ms || 0) + ' ms';
        if ($('sec-drawer-hash')) $('sec-drawer-hash').textContent = activeDecision.evidence_hash || '—';
        if ($('sec-drawer-attributes')) {
          $('sec-drawer-attributes').textContent = JSON.stringify(activeDecision.redacted_attributes || {}, null, 2);
        }
      }
      renderDynamicChain(activeDecision);
    }

    function num(value) {
      return Number(value || 0).toLocaleString();
    }

    function renderCost() {
      const cost = state.cost || {};
      const config = state.config || {};
      const routes = config.routes || [];
      const spend = cost.spend_ledger || { models: [], accounts: [] };
      const tokens = cost.token_ledger || { tiers: {} };
      const eff = cost.efficiency_ledger || { mcp_tools: [], a2a_methods: [] };
      const opt = cost.optimization_ledger || { opportunities: [] };
      const events = cost.events || [];

      // Initial defaults
      if (!state.costLedgerTab) {
        state.costLedgerTab = 'spend';
      }

      // 1. Controls Setup (Route, Model filter, Refresh)
      const routeSelect = $('cost-route-select');
      if (routeSelect) {
        const routeNames = routes.length ? ['all-routes', ...routes.map((r) => r.name)] : ['all-routes'];
        const currentRoute = state.costRoute && routeNames.includes(state.costRoute) ? state.costRoute : routeNames[0];
        state.costRoute = currentRoute;
        const key = routeNames.join('|');
        if (routeSelect.dataset.key !== key) {
          routeSelect.innerHTML = routeNames.map((name) => '<option value="' + esc(name) + '">' + esc(name) + '</option>').join('');
          routeSelect.dataset.key = key;
        }
        routeSelect.value = currentRoute;
        if (!routeSelect.dataset.ready) {
          routeSelect.dataset.ready = '1';
          routeSelect.addEventListener('change', () => {
            state.costRoute = routeSelect.value;
            renderCost();
          });
        }
      }

      const modelFilter = $('cost-model-filter');
      if (modelFilter) {
        const rawModels = (spend.models || []).map((m) => m.model).concat(cost.models || []);
        const uniqueModels = ['all-models', ...Array.from(new Set(rawModels)).filter(Boolean)];
        const currentModel = state.costModelFilter && uniqueModels.includes(state.costModelFilter) ? state.costModelFilter : uniqueModels[0];
        state.costModelFilter = currentModel;
        const mKey = uniqueModels.join('|');
        if (modelFilter.dataset.key !== mKey) {
          modelFilter.innerHTML = uniqueModels.map((m) => '<option value="' + esc(m) + '">' + esc(m) + '</option>').join('');
          modelFilter.dataset.key = mKey;
        }
        modelFilter.value = currentModel;
        if (!modelFilter.dataset.ready) {
          modelFilter.dataset.ready = '1';
          modelFilter.addEventListener('change', () => {
            state.costModelFilter = modelFilter.value;
            renderCost();
          });
        }
      }

      const refreshBtn = $('cost-refresh-btn');
      if (refreshBtn && !refreshBtn.dataset.ready) {
        refreshBtn.dataset.ready = '1';
        refreshBtn.addEventListener('click', () => {
          load();
        });
      }

      // Ledger View Sub-tabs
      const ledgerTabs = $('cost-ledger-tabs');
      if (ledgerTabs && !ledgerTabs.dataset.ready) {
        ledgerTabs.dataset.ready = '1';
        ledgerTabs.querySelectorAll('.cost-tab-btn').forEach((btn) => {
          btn.addEventListener('click', () => {
            state.costLedgerTab = btn.dataset.tabCost;
            ledgerTabs.querySelectorAll('.cost-tab-btn').forEach((b) => b.classList.toggle('active', b === btn));
            ['spend', 'token', 'efficiency', 'optimization', 'requests'].forEach((tabKey) => {
              const view = $('cost-view-' + tabKey);
              if (view) view.classList.toggle('hidden', tabKey !== state.costLedgerTab);
            });
            renderCost();
          });
        });
      }

      // Ensure active tab view is visible
      ['spend', 'token', 'efficiency', 'optimization', 'requests'].forEach((tabKey) => {
        const view = $('cost-view-' + tabKey);
        if (view) view.classList.toggle('hidden', tabKey !== (state.costLedgerTab || 'spend'));
      });
      if (ledgerTabs) {
        ledgerTabs.querySelectorAll('.cost-tab-btn').forEach((b) => b.classList.toggle('active', b.dataset.tabCost === (state.costLedgerTab || 'spend')));
      }

      // 2. Top 5 KPI Metrics (Real runtime data, physical separation)
      if ($('cost-kpi-api-usd')) $('cost-kpi-api-usd').textContent = spend.total_api_usd || '$0.00';
      if ($('cost-kpi-api-priced')) {
        $('cost-kpi-api-priced').textContent = (spend.priced_requests || 0) + ' priced / ' + (spend.unpriced_requests || 0) + ' unpriced';
      }
      if ($('cost-kpi-credits')) $('cost-kpi-credits').textContent = spend.total_chatgpt_credits || '0';
      if ($('cost-kpi-tokens')) $('cost-kpi-tokens').textContent = num(tokens.total_tokens || 0);
      if ($('cost-kpi-in-out-sub')) {
        const inTok = (tokens.total_input_uncached || 0) + (tokens.total_cache_read || 0);
        const outTok = (tokens.total_output_non_reasoning || 0) + (tokens.total_reasoning || 0);
        $('cost-kpi-in-out-sub').textContent = compactNum(inTok) + ' in / ' + compactNum(outTok) + ' out';
      }
      if ($('cost-kpi-cache-rate')) {
        const rate = typeof tokens.cache_hit_rate_pct === 'number' ? tokens.cache_hit_rate_pct.toFixed(1) + '%' : '0.0%';
        $('cost-kpi-cache-rate').textContent = rate;
      }
      if ($('cost-kpi-cache-sub')) {
        $('cost-kpi-cache-sub').textContent = compactNum(tokens.total_cache_read || 0) + ' read tokens';
      }
      if ($('cost-kpi-saved-tokens')) {
        $('cost-kpi-saved-tokens').textContent = num(tokens.context_saved_tokens || opt.gross_potential_tokens || 0);
      }

      // 3. Tab 1: Spend Ledger
      const spendModelTable = $('cost-spend-model-table');
      if (spendModelTable) {
        const modelRows = spend.models || [];
        if ($('cost-spend-models-count')) $('cost-spend-models-count').textContent = modelRows.length + ' models';
        let head = '<thead><tr>' +
          '<th>Model</th><th>Provider</th><th>Requests</th>' +
          '<th>Uncached In</th><th>Cached In</th><th>Output</th>' +
          '<th>API USD</th><th>Credits</th><th>Pricing Status</th>' +
          '</tr></thead>';
        let body = '<tbody>';
        if (modelRows.length === 0) {
          body += '<tr><td colspan="9" style="text-align:center;color:var(--muted);padding:14px">No spend records observed</td></tr>';
        } else {
          modelRows.forEach((m) => {
            const isExact = m.pricing_status === 'exact';
            const badgeCls = isExact ? 'healthy' : 'throttled';
            body += '<tr>' +
              '<td class="code bold">' + esc(m.model) + '</td>' +
              '<td><span class="cap-tag">' + esc(m.provider) + '</span></td>' +
              '<td class="code">' + num(m.requests) + '</td>' +
              '<td class="code muted">' + num(m.uncached_input_tokens) + '</td>' +
              '<td class="code muted">' + num(m.cached_input_tokens) + '</td>' +
              '<td class="code muted">' + num(m.output_tokens) + '</td>' +
              '<td class="code bold">' + esc(m.api_usd || '$0.00') + '</td>' +
              '<td class="code bold">' + esc(m.chatgpt_credits || '0') + '</td>' +
              '<td><span class="status-badge ' + badgeCls + '">' + esc(m.pricing_status) + '</span></td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        spendModelTable.innerHTML = head + body;
      }

      const spendAccountTable = $('cost-spend-account-table');
      if (spendAccountTable) {
        const accountRows = spend.accounts || [];
        if ($('cost-spend-accounts-count')) $('cost-spend-accounts-count').textContent = accountRows.length + ' accounts';
        let head = '<thead><tr>' +
          '<th>Account / Key</th><th>Provider</th><th>Requests</th><th>API USD</th><th>Credits</th>' +
          '</tr></thead>';
        let body = '<tbody>';
        if (accountRows.length === 0) {
          body += '<tr><td colspan="5" style="text-align:center;color:var(--muted);padding:14px">No account spend records observed</td></tr>';
        } else {
          accountRows.forEach((a) => {
            body += '<tr>' +
              '<td class="code bold">' + esc(a.account) + '</td>' +
              '<td><span class="cap-tag">' + esc(a.provider) + '</span></td>' +
              '<td class="code">' + num(a.requests) + '</td>' +
              '<td class="code bold">' + esc(a.api_usd || '$0.00') + '</td>' +
              '<td class="code bold">' + esc(a.chatgpt_credits || '0') + '</td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        spendAccountTable.innerHTML = head + body;
      }

      // 4. Tab 2: Token & Cache Ledger
      if ($('cost-bucket-uncached')) $('cost-bucket-uncached').textContent = num(tokens.total_input_uncached || 0);
      if ($('cost-bucket-cache-read')) $('cost-bucket-cache-read').textContent = num(tokens.total_cache_read || 0);
      if ($('cost-bucket-cache-write')) $('cost-bucket-cache-write').textContent = num(tokens.total_cache_write || 0);
      if ($('cost-bucket-output-std')) $('cost-bucket-output-std').textContent = num(tokens.total_output_non_reasoning || 0);
      if ($('cost-bucket-reasoning')) $('cost-bucket-reasoning').textContent = num(tokens.total_reasoning || 0);
      if ($('cost-bucket-unclassified')) $('cost-bucket-unclassified').textContent = num(tokens.total_unclassified || 0);

      const tiers = tokens.tiers || {};
      if ($('cost-tier-provider-read')) $('cost-tier-provider-read').textContent = num(tiers.provider_cache_read_tokens || 0) + ' tokens';
      if ($('cost-tier-provider-write')) $('cost-tier-provider-write').textContent = num(tiers.provider_cache_write_tokens || 0) + ' tokens';
      if ($('cost-tier-gateway-prefix')) $('cost-tier-gateway-prefix').textContent = num(tiers.gateway_prefix_cache_tokens || 0) + ' tokens';
      if ($('cost-tier-agent-reduced')) $('cost-tier-agent-reduced').textContent = num(tiers.agent_context_tokens_reduced || 0) + ' tokens';

      if ($('cost-quality-complete')) $('cost-quality-complete').textContent = num(tokens.complete_events || 0);
      if ($('cost-quality-inconsistent')) $('cost-quality-inconsistent').textContent = num(tokens.inconsistent_events || 0);
      if ($('cost-token-quality-badge')) {
        const hasInconsistent = (tokens.inconsistent_events || 0) > 0;
        $('cost-token-quality-badge').className = 'status-badge ' + (hasInconsistent ? 'throttled' : 'healthy');
        $('cost-token-quality-badge').textContent = hasInconsistent ? 'Inconsistent Detected' : 'Complete';
      }
      if ($('cost-quality-hit-ratio')) {
        $('cost-quality-hit-ratio').textContent = typeof tokens.cache_hit_rate_pct === 'number' ? tokens.cache_hit_rate_pct.toFixed(1) + '%' : '0.0%';
      }
      if ($('cost-quality-context-saved')) {
        $('cost-quality-context-saved').textContent = num(tokens.context_saved_tokens || 0) + ' tokens';
      }

      // 5. Tab 3: Execution Efficiency
      if ($('cost-eff-retries-badge')) {
        $('cost-eff-retries-badge').textContent = (eff.total_retries || 0) + ' Retries (' + num(eff.retry_overhead_tokens || 0) + ' Overhead Tokens)';
      }
      const mcpTable = $('cost-mcp-table');
      if (mcpTable) {
        const mcpRows = eff.mcp_tools || [];
        if ($('cost-mcp-summary')) $('cost-mcp-summary').textContent = mcpRows.length + ' tools (' + num(eff.mcp_calls || 0) + ' calls)';
        let head = '<thead><tr><th>Tool</th><th>Backend</th><th>Calls</th><th>Failures</th><th>I/O Bytes</th><th>Avg Latency</th></tr></thead>';
        let body = '<tbody>';
        if (mcpRows.length === 0) {
          body += '<tr><td colspan="6" style="text-align:center;color:var(--muted);padding:14px">No MCP tool executions recorded</td></tr>';
        } else {
          mcpRows.forEach((t) => {
            body += '<tr>' +
              '<td class="code bold">' + esc(t.tool) + '</td>' +
              '<td class="code">' + esc(t.backend) + '</td>' +
              '<td class="code">' + num(t.calls) + '</td>' +
              '<td class="code ' + (t.failures > 0 ? 'denied bold' : 'muted') + '">' + num(t.failures) + '</td>' +
              '<td class="code muted">' + compactNum(t.io_bytes) + ' B</td>' +
              '<td class="code">' + num(t.avg_latency_ms) + ' ms</td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        mcpTable.innerHTML = head + body;
      }

      const a2aTable = $('cost-a2a-table');
      if (a2aTable) {
        const a2aRows = eff.a2a_methods || [];
        if ($('cost-a2a-summary')) $('cost-a2a-summary').textContent = a2aRows.length + ' methods (' + num(eff.a2a_calls || 0) + ' calls)';
        let head = '<thead><tr><th>Method</th><th>Backend</th><th>Calls</th><th>Failures</th><th>I/O Bytes</th><th>Avg Latency</th><th>Direct Calls</th><th>Rollup Calls</th></tr></thead>';
        let body = '<tbody>';
        if (a2aRows.length === 0) {
          body += '<tr><td colspan="8" style="text-align:center;color:var(--muted);padding:14px">No A2A method calls recorded</td></tr>';
        } else {
          a2aRows.forEach((m) => {
            body += '<tr>' +
              '<td class="code bold">' + esc(m.method) + '</td>' +
              '<td class="code">' + esc(m.backend) + '</td>' +
              '<td class="code">' + num(m.calls) + '</td>' +
              '<td class="code ' + (m.failures > 0 ? 'denied bold' : 'muted') + '">' + num(m.failures) + '</td>' +
              '<td class="code muted">' + compactNum(m.io_bytes) + ' B</td>' +
              '<td class="code">' + num(m.avg_latency_ms) + ' ms</td>' +
              '<td class="code">' + num(m.direct_calls) + '</td>' +
              '<td class="code muted">' + num(m.rollup_calls) + '</td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        a2aTable.innerHTML = head + body;
      }

      // 6. Tab 4: Optimization & Savings
      if ($('cost-opt-cache-saved')) $('cost-opt-cache-saved').textContent = num(opt.tokens_saved_cache || 0);
      if ($('cost-opt-rtk-saved')) $('cost-opt-rtk-saved').textContent = num(opt.tokens_saved_rtk || 0);
      if ($('cost-opt-condense-saved')) $('cost-opt-condense-saved').textContent = num(opt.tokens_saved_condense || 0);
      if ($('cost-opt-gross-tokens')) $('cost-opt-gross-tokens').textContent = num(opt.gross_potential_tokens || 0);

      const optTable = $('cost-optimization-table');
      if (optTable) {
        const opps = opt.opportunities || [];
        let head = '<thead><tr><th>Category</th><th>Description</th><th>Evidence</th><th>Potential Tokens Saved</th><th>Potential USD Saved</th><th>Status</th></tr></thead>';
        let body = '<tbody>';
        if (opps.length === 0) {
          body += '<tr><td colspan="6" style="text-align:center;color:var(--muted);padding:14px">No optimization opportunities recorded</td></tr>';
        } else {
          opps.forEach((o) => {
            const statusCls = o.realized ? 'healthy' : 'degraded';
            body += '<tr>' +
              '<td><span class="cap-tag">' + esc(o.category) + '</span></td>' +
              '<td class="bold">' + esc(o.description) + '</td>' +
              '<td class="code muted" style="font-size:10.5px">' + esc(o.evidence) + '</td>' +
              '<td class="code bold">' + num(o.potential_tokens_saved) + '</td>' +
              '<td class="code muted">' + esc(o.potential_usd_saved || '—') + '</td>' +
              '<td><span class="status-badge ' + statusCls + '">' + (o.realized ? 'Realized' : 'Potential') + '</span></td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        optTable.innerHTML = head + body;
      }

      // 7. Tab 5: Request Cost Audit & Detail Drawer
      let filteredEvents = events;
      if (state.costRoute && state.costRoute !== 'all-routes') {
        filteredEvents = filteredEvents.filter((ev) => ev.route === state.costRoute);
      }
      if (state.costModelFilter && state.costModelFilter !== 'all-models') {
        filteredEvents = filteredEvents.filter((ev) => ev.model === state.costModelFilter);
      }

      if ($('cost-events-count')) $('cost-events-count').textContent = filteredEvents.length + ' events';

      let activeEvent = null;
      if (state.costSelectedEventId) {
        activeEvent = filteredEvents.find((ev) => ev.event_id === state.costSelectedEventId);
      }
      if (!activeEvent && filteredEvents.length > 0) {
        activeEvent = filteredEvents[0];
        state.costSelectedEventId = activeEvent.event_id;
      }

      const eventsTable = $('cost-events-table');
      if (eventsTable) {
        let head = '<thead><tr>' +
          '<th>Time</th><th>Trace ID</th><th>Protocol</th><th>Operation / Route</th><th>Model</th>' +
          '<th>Tokens (In / Out / Cache)</th><th>Latency</th><th>Spend (USD / Credits)</th><th>Attribution</th><th>Quality</th>' +
          '</tr></thead>';
        let body = '<tbody>';
        if (filteredEvents.length === 0) {
          body += '<tr><td colspan="10" style="text-align:center;color:var(--muted);padding:14px">No cost audit events recorded</td></tr>';
        } else {
          filteredEvents.forEach((ev) => {
            const isSel = activeEvent && ev.event_id === activeEvent.event_id;
            const timeStr = ev.timestamp_ms ? new Date(ev.timestamp_ms).toLocaleTimeString() : '—';
            const tb = ev.token_breakdown || {};
            const inTok = (tb.input_uncached || 0) + (tb.cache_read || 0);
            const outTok = (tb.output_non_reasoning || 0) + (tb.reasoning || 0);
            const tokStr = inTok + ' / ' + outTok + ' / ' + (tb.cache_read || 0);
            const spendStr = (ev.api_usd || 'Unpriced') + ' / ' + (ev.chatgpt_credits || '—');
            const qualCls = ev.data_quality === 'complete' ? 'healthy' : 'throttled';
            const shortTrace = ev.trace_id ? (ev.trace_id.length > 16 ? ev.trace_id.slice(0, 16) + '...' : ev.trace_id) : '—';

            body += '<tr class="' + (isSel ? 'active-row' : '') + '" data-event-id="' + esc(ev.event_id) + '">' +
              '<td class="code muted">' + esc(timeStr) + '</td>' +
              '<td class="code bold"><a href="javascript:void(0)" onclick="event.stopPropagation(); if (window.jumpToTrace) window.jumpToTrace(\'' + esc(ev.trace_id) + '\')" class="muted-link">' + esc(shortTrace) + '</a></td>' +
              '<td><span class="cap-tag">' + esc(ev.protocol) + '</span></td>' +
              '<td class="code">' + esc(ev.operation || ev.route) + '</td>' +
              '<td class="code bold">' + esc(ev.model) + '</td>' +
              '<td class="code muted" style="font-size:10.5px">' + esc(tokStr) + '</td>' +
              '<td class="code">' + (ev.latency_ms || 0) + ' ms</td>' +
              '<td class="code bold">' + esc(spendStr) + '</td>' +
              '<td><span class="cap-tag">' + esc(ev.attribution_mode || 'direct') + '</span></td>' +
              '<td><span class="status-badge ' + qualCls + '">' + esc(ev.data_quality) + '</span></td>' +
              '</tr>';
          });
        }
        body += '</tbody>';
        eventsTable.innerHTML = head + body;

        eventsTable.querySelectorAll('tbody tr[data-event-id]').forEach((tr) => {
          tr.addEventListener('click', () => {
            state.costSelectedEventId = tr.dataset.eventId;
            renderCost();
          });
        });
      }

      // Update Request Detail Drawer
      if (activeEvent) {
        if ($('cost-det-event-id')) $('cost-det-event-id').textContent = activeEvent.event_id;
        if ($('cost-det-trace')) $('cost-det-trace').textContent = activeEvent.trace_id;
        if ($('cost-det-span')) $('cost-det-span').textContent = activeEvent.span_id;
        if ($('cost-det-route')) $('cost-det-route').textContent = activeEvent.route;
        if ($('cost-det-backend')) $('cost-det-backend').textContent = activeEvent.backend;
        if ($('cost-det-model')) $('cost-det-model').textContent = activeEvent.model;
        if ($('cost-det-provider')) $('cost-det-provider').textContent = activeEvent.provider;
        if ($('cost-det-protocol')) $('cost-det-protocol').textContent = activeEvent.protocol;
        if ($('cost-det-operation')) $('cost-det-operation').textContent = activeEvent.operation;
        if ($('cost-det-status')) {
          $('cost-det-status').textContent = activeEvent.status_code || 200;
          $('cost-det-status').className = 'obs-prop-v code bold ' + (activeEvent.status_code >= 400 ? 'denied' : 'pass');
        }
        if ($('cost-det-latency')) $('cost-det-latency').textContent = (activeEvent.latency_ms || 0) + ' ms';

        if ($('cost-det-usd')) $('cost-det-usd').textContent = activeEvent.api_usd || 'Unpriced';
        if ($('cost-det-credits')) $('cost-det-credits').textContent = activeEvent.chatgpt_credits || '—';
        if ($('cost-det-pricing-status')) {
          const isExact = activeEvent.pricing_status === 'exact';
          $('cost-det-pricing-status').innerHTML = '<span class="status-badge ' + (isExact ? 'healthy' : 'throttled') + '">' + esc(activeEvent.pricing_status) + '</span>';
        }
        if ($('cost-det-attribution')) {
          $('cost-det-attribution').innerHTML = '<span class="cap-tag">' + esc(activeEvent.attribution_mode || 'direct') + '</span>';
        }
        if ($('cost-det-quality')) {
          const isComp = activeEvent.data_quality === 'complete';
          $('cost-det-quality').innerHTML = '<span class="status-badge ' + (isComp ? 'healthy' : 'throttled') + '">' + esc(activeEvent.data_quality) + '</span>';
        }

        const tb = activeEvent.token_breakdown || {};
        if ($('cost-det-tok-uncached')) $('cost-det-tok-uncached').textContent = num(tb.input_uncached || 0);
        if ($('cost-det-tok-cache-read')) $('cost-det-tok-cache-read').textContent = num(tb.cache_read || 0);
        if ($('cost-det-tok-cache-write')) $('cost-det-tok-cache-write').textContent = num(tb.cache_write || 0);
        if ($('cost-det-tok-out-std')) $('cost-det-tok-out-std').textContent = num(tb.output_non_reasoning || 0);
        if ($('cost-det-tok-reasoning')) $('cost-det-tok-reasoning').textContent = num(tb.reasoning || 0);
        if ($('cost-det-tok-unclass')) $('cost-det-tok-unclass').textContent = num(tb.unclassified || 0);

        if ($('cost-det-ttft')) $('cost-det-ttft').textContent = typeof activeEvent.ttft_ms === 'number' ? activeEvent.ttft_ms + ' ms' : '—';
        if ($('cost-det-io-bytes')) $('cost-det-io-bytes').textContent = compactNum(activeEvent.io_bytes || 0) + ' B';
        if ($('cost-det-retries')) $('cost-det-retries').textContent = activeEvent.retries || 0;

        const viewTraceBtn = $('cost-det-view-trace-btn');
        if (viewTraceBtn) {
          viewTraceBtn.onclick = () => {
            if (activeEvent.trace_id && window.jumpToTrace) {
              window.jumpToTrace(activeEvent.trace_id);
            }
          };
        }
      } else {
        if ($('cost-det-event-id')) $('cost-det-event-id').textContent = '-';
        if ($('cost-det-trace')) $('cost-det-trace').textContent = '-';
        if ($('cost-det-span')) $('cost-det-span').textContent = '-';
        if ($('cost-det-route')) $('cost-det-route').textContent = '-';
        if ($('cost-det-backend')) $('cost-det-backend').textContent = '-';
        if ($('cost-det-model')) $('cost-det-model').textContent = '-';
        if ($('cost-det-provider')) $('cost-det-provider').textContent = '-';
        if ($('cost-det-protocol')) $('cost-det-protocol').textContent = '-';
        if ($('cost-det-operation')) $('cost-det-operation').textContent = '-';
        if ($('cost-det-status')) $('cost-det-status').textContent = '-';
        if ($('cost-det-latency')) $('cost-det-latency').textContent = '-';
        if ($('cost-det-usd')) $('cost-det-usd').textContent = '$0.00';
        if ($('cost-det-credits')) $('cost-det-credits').textContent = '0';
        if ($('cost-det-pricing-status')) $('cost-det-pricing-status').innerHTML = '—';
        if ($('cost-det-attribution')) $('cost-det-attribution').innerHTML = '—';
        if ($('cost-det-quality')) $('cost-det-quality').innerHTML = '—';
        if ($('cost-det-tok-uncached')) $('cost-det-tok-uncached').textContent = '0';
        if ($('cost-det-tok-cache-read')) $('cost-det-tok-cache-read').textContent = '0';
        if ($('cost-det-tok-cache-write')) $('cost-det-tok-cache-write').textContent = '0';
        if ($('cost-det-tok-out-std')) $('cost-det-tok-out-std').textContent = '0';
        if ($('cost-det-tok-reasoning')) $('cost-det-tok-reasoning').textContent = '0';
        if ($('cost-det-tok-unclass')) $('cost-det-tok-unclass').textContent = '0';
        if ($('cost-det-ttft')) $('cost-det-ttft').textContent = '—';
        if ($('cost-det-io-bytes')) $('cost-det-io-bytes').textContent = '0 B';
        if ($('cost-det-retries')) $('cost-det-retries').textContent = '0';
      }

      // 8. Legacy Cost Logic Preserved
      if ($('metric-api-usd')) $('metric-api-usd').textContent = cost.api_usd || '$0';
      if ($('metric-credits')) $('metric-credits').textContent = cost.chatgpt_credits || '0';
      if ($('metric-local-input')) $('metric-local-input').textContent = num(cost.local_input);
      if ($('metric-local-cache')) $('metric-local-cache').textContent = num(cost.local_cache_read);
      if ($('metric-local-write')) $('metric-local-write').textContent = num(cost.local_cache_write);
      if ($('metric-local-output')) $('metric-local-output').textContent = num(cost.local_output);
      if ($('metric-local-total')) $('metric-local-total').textContent = num(cost.local_total);
      const billing = cost.billing || state.billing || 'subscription';
      const isApi = billing === 'api';
      if ($('metric-api-usd')) {
        $('metric-api-usd').textContent = isApi ? (cost.api_usd || '$0') : '—';
      }
      if ($('metric-credits')) {
        $('metric-credits').textContent = isApi
          ? '—'
          : (cost.local_credits_complete ? (cost.local_credits || '0') : t('unpublished'));
      }
      const familySelect = $('cost-family');
      if (familySelect) {
        familySelect.value = cost.family || state.family || 'chatgpt';
        if (!familySelect.dataset.ready) {
          familySelect.dataset.ready = '1';
          familySelect.addEventListener('change', () => {
            state.family = familySelect.value;
            state.model = 'all';
            localStorage.setItem('transit-family', state.family);
            localStorage.setItem('transit-model', 'all');
            load();
          });
        }
      }
      const modelSelect = $('cost-model');
      if (modelSelect) {
        const names = ['all'].concat(cost.models || []);
        const key = (cost.family || '') + '|' + names.join('|');
        if (modelSelect.dataset.key !== key) {
          modelSelect.innerHTML = names.map((name) => {
            const label = name === 'all' ? t('allModels') : name;
            return '<option value="' + esc(name) + '">' + esc(label) + '</option>';
          }).join('');
          modelSelect.dataset.key = key;
        }
        if (!modelSelect.dataset.ready) {
          modelSelect.dataset.ready = '1';
          modelSelect.addEventListener('change', () => {
            state.model = modelSelect.value;
            localStorage.setItem('transit-model', state.model);
            load();
          });
        }
        const want = cost.model || 'all';
        modelSelect.value = names.indexOf(want) >= 0 ? want : 'all';
      }
      const billingSelect = $('cost-billing');
      if (billingSelect) {
        if (billingSelect.options && billingSelect.options.length > 0) {
          billingSelect.options[0].text = t('subscription');
        }
        billingSelect.value = billing;
        const countrySelect = $('fx-country');
        if (countrySelect) countrySelect.disabled = !isApi;
        if (!billingSelect.dataset.ready) {
          billingSelect.dataset.ready = '1';
          billingSelect.addEventListener('change', () => {
            state.billing = billingSelect.value;
            localStorage.setItem('transit-billing', state.billing);
            load();
          });
        }
      }
      const select = $('fx-country');
      if (select && !select.dataset.ready) {
        const countries = [];
        const seen = {};
        (cost.fx_rates || []).forEach((row) => {
          if (!row.country || seen[row.country]) return;
          seen[row.country] = true;
          countries.push(row);
        });
        countries.sort((a, b) => a.country.localeCompare(b.country));
        select.innerHTML = countries.map((row) => {
          const selected = row.country === (cost.fx_country || 'United States') ? ' selected' : '';
          return '<option value="' + esc(row.country) + '"' + selected + '>' + esc(row.country) + ' (' + esc(row.currency_code) + ')</option>';
        }).join('');
        select.dataset.ready = '1';
        select.addEventListener('change', () => {
          state.country = select.value;
          localStorage.setItem('transit-country', state.country);
          load();
        });
      } else if (select && cost.fx_country && select.value !== cost.fx_country) {
        select.value = cost.fx_country;
      }
      const meta = $('fx-meta');
      if (meta) {
        const fx = cost.local_fx;
        meta.textContent = fx
          ? ('1 USD = ' + fx.units_per_usd + ' ' + fx.currency_code + '  ' + (cost.fx_as_of || ''))
          : '';
      }
      fillTable('cost-local-table', [
        t('source'), t('model'), t('requests'), t('prompt'), t('cached'), t('cacheWrite'), t('completion'), t('tokens')
      ], (cost.local || []).map((row) => [
        row.source || '',
        row.model || '',
        num(row.requests),
        num(row.prompt_tokens),
        num(row.cached_prompt_tokens),
        num(row.cache_write_tokens),
        num(row.completion_tokens),
        num(row.total_tokens)
      ]));
      fillTable('cost-usage-table', [
        t('model'), t('requests'), t('prompt'), t('cached'), t('completion'), t('apiUsd'), t('chatgptCredits'), t('complete')
      ], (cost.usage || []).map((row) => [
        row.model || '',
        row.requests || 0,
        row.prompt_tokens || 0,
        row.cached_prompt_tokens || 0,
        row.completion_tokens || 0,
        row.api_usd || row.error || '',
        row.chatgpt_credits || '',
        row.chatgpt_credits_complete === true ? 'yes' : (row.error ? '' : 'no')
      ]));
      renderFlame(cost.ticks || []);
      fillTable('cost-rate-table', [
        t('vendor'), t('model'), t('tier'), t('context'), t('inputUsd'), t('cachedUsd'), t('outputUsd'), t('inputCredits'), t('cachedCredits'), t('outputCredits')
      ], (cost.rate_card || []).map((row) => [
        row.vendor || '',
        row.model || '',
        row.tier || '',
        row.context || '',
        row.input_usd_per_1m || '',
        row.cached_usd_per_1m || '',
        row.output_usd_per_1m || '',
        row.input_credits_per_1m || '',
        row.cached_credits_per_1m || '',
        row.output_credits_per_1m || ''
      ]));
    }

    function compactNum(value) {
      const n = Number(value) || 0;
      if (n >= 1e9) return (n / 1e9).toFixed(1) + 'B';
      if (n >= 1e6) return (n / 1e6).toFixed(1) + 'M';
      if (n >= 1e3) return (n / 1e3).toFixed(1) + 'K';
      return String(Math.round(n));
    }

    function renderFlame(ticks) {
      const svg = $('cost-flame-svg');
      const last = $('flame-last');
      if (!svg) return;
      const W = 960, H = 280, padL = 58, padR = 16, top = 18, plotB = 252;
      const plotW = W - padL - padR;
      if (!ticks.length) {
        if (last) last.textContent = '';
        svg.innerHTML = '<text x="480" y="140" text-anchor="middle" fill="#667085" font-size="13">' + esc(t('noLocalUsage')) + '</text>';
        return;
      }
      const daily = ticks.map((p) => Number(p.tokens) || 0);
      let sum = 0;
      const mountain = daily.map((v) => { sum += v; return sum; });
      const n = ticks.length;
      const maxM = Math.max(mountain[n - 1], 1);
      const xAt = (i) => padL + (n === 1 ? plotW / 2 : i * plotW / (n - 1));
      const yM = (v) => top + (1 - v / maxM) * (plotB - top);
      let area = 'M ' + xAt(0) + ' ' + plotB;
      mountain.forEach((v, i) => { area += ' L ' + xAt(i) + ' ' + yM(v); });
      area += ' L ' + xAt(n - 1) + ' ' + plotB + ' Z';
      const line = mountain.map((v, i) => (i ? 'L' : 'M') + ' ' + xAt(i) + ' ' + yM(v)).join(' ');
      if (last) last.textContent = ticks[0].day + ' → ' + ticks[n - 1].day + '  ' + compactNum(mountain[n - 1]);
      const grid = [0, 0.5, 1].map((p) => {
        const v = maxM * (1 - p);
        const y = top + p * (plotB - top);
        return '<line x1="' + padL + '" y1="' + y + '" x2="' + (W - padR) + '" y2="' + y + '" stroke="#2a3038" stroke-width="1"/>'
          + '<text x="' + (padL - 6) + '" y="' + (y + 3) + '" text-anchor="end" fill="#8b93a0" font-size="10">' + compactNum(v) + '</text>';
      }).join('');
      svg.innerHTML = '<defs><linearGradient id="flameFill" x1="0" y1="1" x2="0" y2="0">'
        + '<stop offset="0%" stop-color="#3a0a00" stop-opacity="0.2"/>'
        + '<stop offset="100%" stop-color="#ff8a00" stop-opacity="0.7"/>'
        + '</linearGradient></defs>'
        + grid
        + '<path id="flame-area" d="' + area + '" fill="url(#flameFill)"/>'
        + '<path id="flame-line" d="' + line + '" fill="none" stroke="#ffd36a" stroke-width="1.6"/>';
    }

    function applyChrome() {
      document.documentElement.dataset.theme = state.theme;
      document.documentElement.lang = state.lang;
      localStorage.setItem('transit-theme', state.theme);
      localStorage.setItem('transit-lang', state.lang);
      document.querySelectorAll('[data-i18n]').forEach((el) => {
        el.textContent = t(el.dataset.i18n);
      });
      if ($('search-input')) $('search-input').placeholder = t('search');
      $('theme-toggle').dataset.on = state.theme;
      $('theme-toggle').setAttribute('aria-label', state.theme === 'dark' ? t('themeLight') : t('themeDark'));
      $('lang-toggle').dataset.on = state.lang;
      $('lang-toggle').setAttribute('aria-label', state.lang === 'zh' ? 'EN' : '中');
      const active = document.querySelector('.nav button.active');
      if (active) $('title').textContent = t(active.dataset.tab);
      updatePageChrome();
    }

    function setTab(tab) {
      const button = document.querySelector('.nav button[data-tab="' + tab + '"]') || document.querySelector('.nav button[data-tab="overview"]');
      if (button) {
        closeDrawers();
        state.queries[state.tab] = state.query;
        state.tab = button.dataset.tab;
        state.query = state.queries[state.tab] || '';
        if ($('search-input')) $('search-input').value = state.query;
        document.querySelectorAll('.nav button').forEach((b) => {
          b.classList.toggle('active', b === button);
          if (b === button) b.setAttribute('aria-current', 'page');
          else b.removeAttribute('aria-current');
        });
        if ($('title')) $('title').textContent = t(button.dataset.tab);
        document.querySelectorAll('main section').forEach((section) => section.classList.add('hidden'));
        const targetSec = $('tab-' + button.dataset.tab);
        if (targetSec) targetSec.classList.remove('hidden');
        history.replaceState(null, '', '#' + button.dataset.tab);
        render();
        if (state.started) load();
      }
    }

    async function load() {
      if (state.loading) { state.reloadPending = true; return; }
      state.loading = true;
      const requestedTab = state.tab;
      updatePageChrome();
      try {
        const country = encodeURIComponent(state.country || localStorage.getItem('transit-country') || 'United States');
        const family = encodeURIComponent(state.family || localStorage.getItem('transit-family') || 'chatgpt');
        const model = encodeURIComponent(state.model || localStorage.getItem('transit-model') || 'all');
        const billing = encodeURIComponent(state.billing || localStorage.getItem('transit-billing') || 'subscription');
        const endpoints = [{ key: 'config', url: '/debug/config' }];
        if (state.tab === 'llm') endpoints.push({ key: 'llmData', url: '/debug/llm' });
        if (state.tab === 'cost-control') endpoints.push({ key: 'cost', url: '/debug/cost?country=' + country + '&family=' + family + '&model=' + model + '&billing=' + billing });
        if (state.tab === 'observability') endpoints.push({ key: 'obsData', url: '/debug/observability' });
        if (state.tab === 'services') endpoints.push({ key: 'servicesData', url: '/debug/services' });
        if (state.tab === 'security') endpoints.push(
          { key: 'secPosture', url: '/debug/security/posture' },
          { key: 'secEvents', url: '/debug/security/events' },
          { key: 'secIdentities', url: '/debug/security/identities' });
        const results = await Promise.allSettled(endpoints.map(async item => {
          const response = await managementFetch(item.url, { cache: 'no-store', signal: AbortSignal.timeout(8000) });
          if (!response.ok) throw new Error(item.url.split('?')[0] + ' returned ' + response.status);
          return response.json();
        }));
        let changed = false;
        results.forEach((result, index) => {
          const { key, url } = endpoints[index];
          if (result.status === 'fulfilled') {
            changed = changed || JSON.stringify(state[key]) !== JSON.stringify(result.value);
            state[key] = result.value;
            delete state.endpointErrors[url.split('?')[0]];
          } else {
            state.endpointErrors[url.split('?')[0]] = result.reason.message;
          }
        });
        if (results.every(result => result.status === 'fulfilled')) state.syncedAt[requestedTab] = new Date().toLocaleTimeString();
        if (changed || requestedTab === 'llm') render();
      } catch (err) {
        $('error').textContent = err.message;
        $('error').classList.remove('hidden');
      } finally {
        state.loading = false;
        updatePageChrome();
        if (state.reloadPending) { state.reloadPending = false; load(); }
      }
    }

    function closeDrawers() {
      document.querySelectorAll('.svc-drawer.open, .svc-drawer-backdrop.open, .obs-trace-drawer.open, .obs-drawer-backdrop.open, .sec-drawer.open, .sec-drawer-backdrop.open').forEach(el => el.classList.remove('open'));
      state.svcDrawerOpen = false;
      state.obsDrawerOpen = false;
      state.secDrawerOpen = false;
      syncDrawers();
    }

    function syncDrawers() {
      const drawers = document.querySelectorAll('.svc-drawer, .obs-trace-drawer, .sec-drawer');
      const active = Array.from(drawers).find(el => el.classList.contains('open') && !el.closest('section.hidden'));
      drawers.forEach(el => {
        el.setAttribute('role', 'dialog');
        el.setAttribute('aria-modal', 'true');
        el.setAttribute('aria-label', uiText('Record details', '记录详情'));
        el.setAttribute('aria-hidden', String(el !== active));
        el.inert = el !== active;
        el.tabIndex = -1;
      });
      if ((active?.id || null) === (state.openDrawerId || null)) return;
      document.querySelector('.nav').inert = !!active;
      document.querySelectorAll('main > *').forEach(el => { el.inert = !!active && !el.contains(active); });
      if (active) {
        active.parentElement.querySelectorAll(':scope > *').forEach(el => {
          if (el !== active && !el.className.includes('backdrop')) el.inert = true;
        });
        state.previousOverflow = document.body.style.overflow;
        document.body.style.overflow = 'hidden';
        (active.querySelector('button') || active).focus({ preventScroll: true });
      } else {
        document.querySelectorAll('main section > [inert]').forEach(el => { el.inert = false; });
        drawers.forEach(el => { el.inert = true; });
        document.body.style.overflow = state.previousOverflow || '';
        const opener = state.drawerReturnSelector && document.querySelector(state.drawerReturnSelector);
        if (opener) { opener.tabIndex = 0; opener.focus({ preventScroll: true }); }
      }
      state.openDrawerId = active?.id || null;
    }

    document.querySelectorAll('.nav button').forEach((button) => {
      button.addEventListener('click', () => setTab(button.dataset.tab));
    });
    if ($('search-input')) {
      $('search-input').addEventListener('input', (event) => {
        state.query = event.target.value;
        applyPageFilter();
      });
    }
    $('theme-toggle').addEventListener('click', () => {
      state.theme = state.theme === 'dark' ? 'light' : 'dark';
      applyChrome();
    });
    $('lang-toggle').addEventListener('click', () => {
      state.lang = state.lang === 'zh' ? 'en' : 'zh';
      applyChrome();
      render();
    });
    if ($('reload-data')) $('reload-data').addEventListener('click', load);
    if ($('poll-toggle')) $('poll-toggle').addEventListener('click', () => {
      state.paused = !state.paused;
      updatePageChrome();
      if (!state.paused) load();
    });
    if ($('examples-toggle')) $('examples-toggle').addEventListener('click', () => {
      state.showExamples = !state.showExamples;
      window.selectedLlmProvider = 'all';
      state.mcpSelectedServer = null;
      state.a2aSelectedAgent = null;
      state.a2aSelectedTaskIdx = 0;
      state.mcpSelectedInvIdx = 0;
      render();
    });
    document.addEventListener('keydown', event => {
      const drawer = state.openDrawerId && $(state.openDrawerId);
      if (drawer && event.key === 'Tab') {
        const controls = Array.from(drawer.querySelectorAll('button, a[href], input, select, [tabindex="0"]')).filter(el => !el.disabled && el.getClientRects().length);
        const first = controls[0] || drawer, last = controls[controls.length - 1] || drawer;
        if (event.shiftKey && (document.activeElement === first || document.activeElement === drawer)) { event.preventDefault(); last.focus(); }
        else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        const si = $('search-input');
        if (si && !drawer) {
          event.preventDefault();
          si.focus();
          si.select();
        }
      } else if (event.key === 'Escape') {
        closeDrawers();
        if ($('ui-toast')) $('ui-toast').classList.add('hidden');
      } else if (['Enter', ' '].includes(event.key) && event.target.matches('tr[tabindex]')) {
        event.preventDefault();
        event.target.click();
      }
    });
    document.addEventListener('visibilitychange', () => {
      if (!document.hidden && !state.paused) load();
    });
    document.addEventListener('click', event => {
      const row = event.target.closest('tr[data-svc-id], tr[data-trace-idx], tr[data-decision-idx]');
      if (!row) return;
      const attr = ['data-svc-id', 'data-trace-idx', 'data-decision-idx'].find(name => row.hasAttribute(name));
      state.drawerReturnSelector = '#' + row.closest('table').id + ' [' + attr + '="' + CSS.escape(row.getAttribute(attr)) + '"]';
    }, true);
    const drawerObserver = new MutationObserver(syncDrawers);
    document.querySelectorAll('.svc-drawer, .obs-trace-drawer, .sec-drawer').forEach(el => drawerObserver.observe(el, { attributes: true, attributeFilter: ['class'] }));
    const contentObserver = new MutationObserver(records => {
      if (records.some(record => record.target.closest?.('#tab-' + state.tab))) applyPageFilter();
    });
    document.querySelectorAll('main section').forEach(el => contentObserver.observe(el, { childList: true, subtree: true }));
    syncDrawers();
    applyChrome();
    setTab(location.hash ? location.hash.slice(1) : 'overview');
    state.started = true;
    load();
    setInterval(() => { if (!document.hidden && !state.paused) load(); }, 4000);
