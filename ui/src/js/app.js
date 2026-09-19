    const I18N = {
      en: {
        overview: 'Overview',
        core: 'Core',
        base: 'Core',
        ai: 'AI',
        setup: 'Setup',
        configuration: 'Configuration',
        mcp: 'MCP',
        gatewayGroup: 'GATEWAY',
        a2a: 'A2A',
        resources: 'Resources',
        services: 'APIs',
        routes: 'Routes',
        subscription: 'Subscription',
        provider: 'Provider',
        model: 'Model',
        server: 'Server',
        search: 'Search',
        themeLight: 'Light',
        themeDark: 'Dark',
        apiServices: 'API services',
        mcpPlugins: 'MCP plugins',
        agents: 'Agents',
        llmProviders: 'LLM providers',
        overviewSubtitle: 'Monitor your gateway\'s configuration, gateway assets, and runtime status',
        viewServices: 'View APIs',
        viewRoutes: 'View Routes',
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
        auth: 'Auth',
        rate: 'Rate',
        tokens: 'Tokens',
        apiUsd: 'API USD',
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
        payload: 'Payload'
      },
      zh: {
        overview: '总览',
        core: 'Core',
        base: 'Core',
        ai: 'AI',
        setup: 'Setup',
        configuration: '配置',
        mcp: 'MCP',
        gatewayGroup: 'GATEWAY',
        llm: 'LLM',
        a2a: 'A2A',
        resources: '资源',
        services: 'APIs',
        routes: '路由',
        subscription: '订阅',
        provider: '供应商',
        model: '模型',
        server: '服务器',
        search: '搜索',
        themeLight: '亮色',
        themeDark: '暗色',
        apiServices: 'API 服务',
        mcpPlugins: 'MCP 插件',
        agents: '智能体',
        llmProviders: 'LLM 供应商',
        overviewSubtitle: '监控网关配置、网关资产与实时运行状态',
        viewServices: '查看 APIs',
        viewRoutes: '查看路由',
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
        payload: '报文负载'
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
        configuration: ['', '']
      };
      const pair = descriptions[state.tab] || descriptions.overview;
      $('page-description').textContent = uiText(...pair);
      const isLlmTab = ['llm', 'subscription', 'provider', 'model'].includes(state.tab);
      const isMcpTab = ['mcp', 'server'].includes(state.tab);
      const isA2aTab = state.tab === 'a2a';
      const aiPage = isMcpTab || isA2aTab;
      if (isLlmTab) {
        // preserve LLM actions
      } else if ($('page-actions')) {
        $('page-actions').innerHTML = '';
      }
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
      const scopes = {
        llm: ['/debug/llm'],
        subscription: ['/debug/llm'],
        provider: ['/debug/llm'],
        model: ['/debug/llm'],
        mcp: [],
        server: [],
        services: ['/debug/services']
      };
      const relevant = ['/debug/config', ...(scopes[state.tab] || [])];
      const failures = relevant.filter(url => state.endpointErrors[url]).map(url => state.endpointErrors[url]);
      const syncText = state.loading ? uiText('Syncing…', '同步中…') : failures.length ? uiText('Update incomplete', '更新不完整') : state.paused ? uiText('Updates paused', '自动更新已暂停') : state.syncedAt[state.tab] ? uiText('Synced ', '已同步 ') + state.syncedAt[state.tab] : uiText('Awaiting data', '等待数据');
      if ($('sync-state')) $('sync-state').textContent = syncText;
      $('error').textContent = failures.length ? uiText('Some data could not be updated. Retaining the last available values. ', '部分数据更新失败，保留上次可用结果。') + failures.join(' · ') : '';
      $('error').classList.toggle('hidden', !failures.length);
      const notice = $('data-notice');
      notice.classList.toggle('hidden', !aiPage);
      notice.dataset.kind = 'config';
      notice.textContent = uiText('Configuration view · Unreported metrics are shown as “—”. Runtime controls are not connected.', '配置视图 · 未上报的指标显示为「—」，运行控制尚未接入。');
    }

    function applyPageFilter() {
      let page = $('tab-' + state.tab);
      if (!page) {
        if (['subscription', 'provider', 'model'].includes(state.tab)) page = $('tab-llm');
        else if (state.tab === 'server') page = $('tab-mcp');
      }
      if (!page) return;
      const selector = ['llm', 'subscription', 'provider', 'model'].includes(state.tab) ? '.llm-account-card' : state.tab === 'overview' ? '.ov-kpi-card' : '.table-wrap tbody tr';
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
              policies: [],
              replace_prefix_match: null,
              health_ratio: routeTotalEps > 0 ? (routeHealthyEps + '/' + routeTotalEps) : '1/1',
              status: status
            });
          });
        });
      });

      // 2. Routes from config.routes (listeners with targets or flat route items)
      let rawRouteItems = [];
      let ambientHostname = null;

      if (config.routes) {
        if (Array.isArray(config.routes)) {
          rawRouteItems = config.routes;
        } else if (Array.isArray(config.routes.listeners)) {
          rawRouteItems = config.routes.listeners;
        } else if (typeof config.routes.listeners === 'object') {
          rawRouteItems = Object.values(config.routes.listeners);
        }
      } else if (Array.isArray(config.listeners)) {
        rawRouteItems = config.listeners;
      }

      // First pass: extract ambient hostname if present
      rawRouteItems.forEach(item => {
        if (item && typeof item === 'object' && item.hostname && !item.name && !item.matches && !item.backends && !item.targets && !item.rules && !item.port) {
          ambientHostname = item.hostname;
        }
      });

      rawRouteItems.forEach((listenerOrRoute, lIdx) => {
        if (!listenerOrRoute || typeof listenerOrRoute !== 'object') return;
        if (listenerOrRoute.hostname && !listenerOrRoute.name && !listenerOrRoute.matches && !listenerOrRoute.backends && !listenerOrRoute.targets && !listenerOrRoute.rules && !listenerOrRoute.port) {
          return; // Skip ambient hostname entry
        }

        const port = listenerOrRoute.port || 6010;
        const protocol = (listenerOrRoute.protocol || 'http').toUpperCase();
        const listenerName = listenerOrRoute.name || ('listener-' + port);
        const listenerStr = listenerName + ':' + port;
        const domain = listenerOrRoute.hostname || ambientHostname || 'mesh.local';
        domainsSet.add(domain);

        const targets = listenerOrRoute.targets || listenerOrRoute.rules || [listenerOrRoute];
        targets.forEach((target, tIdx) => {
          const targetName = target.name || ('target-' + (tIdx + 1));
          const id = 'route:' + listenerName + ':' + targetName;
          let pathStr = '/';
          let matchType = 'prefix';

          let matchObj = null;
          if (Array.isArray(target.matches) && target.matches.length > 0) {
            matchObj = target.matches[0];
          } else if (target.matches && typeof target.matches === 'object') {
            matchObj = target.matches;
          } else if (typeof target.matches === 'string') {
            pathStr = target.matches;
          }

          if (matchObj) {
            if (matchObj.pathPrefix || matchObj.path_prefix || matchObj.prefix) {
              pathStr = matchObj.pathPrefix || matchObj.path_prefix || matchObj.prefix;
              matchType = 'prefix';
            } else if (matchObj.pathExact || matchObj.path_exact || matchObj.exact) {
              pathStr = matchObj.pathExact || matchObj.path_exact || matchObj.exact;
              matchType = 'exact';
            } else if (matchObj.pathRegex || matchObj.path_regex || matchObj.regex) {
              pathStr = matchObj.pathRegex || matchObj.path_regex || matchObj.regex;
              matchType = 'regex';
            } else if (matchObj.path) {
              if (typeof matchObj.path === 'string') {
                pathStr = matchObj.path;
              } else if (matchObj.path.prefix) {
                pathStr = matchObj.path.prefix;
                matchType = 'prefix';
              } else if (matchObj.path.exact) {
                pathStr = matchObj.path.exact;
                matchType = 'exact';
              } else if (matchObj.path.regex) {
                pathStr = matchObj.path.regex;
                matchType = 'regex';
              }
            }
          }

          const methods = target.methods || (target.method ? [target.method] : ['*']);

          const clusterItems = [];
          const rawBackends = target.backends || target.endpoints || target.weighted_backends || [];
          rawBackends.forEach((wb, bIdx) => {
            let addr = '127.0.0.1:8080';
            let weight = 100;
            let bName = 'backend-' + (bIdx + 1);
            if (typeof wb === 'string') {
              addr = wb;
              bName = wb;
            } else if (wb && typeof wb === 'object') {
              addr = wb.address || wb.endpoint || wb.name || '127.0.0.1:8080';
              weight = wb.weight || 100;
              bName = wb.name || addr;
            }
            clusterItems.push({
              name: bName,
              weight: weight,
              percent: 100,
              http2: protocol.includes('H2') || protocol.includes('GRPC'),
              tls_mode: listenerOrRoute.tls ? 'tls' : 'plaintext',
              circuit_breaker: null,
              outlier_detection: null,
              endpoints: [{
                address: addr,
                weight: weight,
                health_status: 'healthy'
              }]
            });
            totalHealthyEps++;
            totalAllEps++;
          });

          const policiesList = [];
          if (target.policies) {
            if (Array.isArray(target.policies)) {
              policiesList.push(...target.policies);
            } else if (typeof target.policies === 'object') {
              Object.keys(target.policies).forEach(k => policiesList.push(k));
            }
          }
          if (target.a2a) {
            policiesList.push('a2a');
          }

          unified.push({
            id: id,
            name: targetName,
            source: target.a2a || (target.policies && target.policies.a2a) ? 'a2a' : 'route',
            listener: listenerStr,
            listener_port: port,
            domain: domain,
            path: pathStr,
            match_type: matchType,
            methods: methods,
            headers: [],
            protocol: protocol.startsWith('HTTP') ? protocol : 'HTTP/1.1',
            tls_mode: listenerOrRoute.tls ? 'tls' : 'plaintext',
            clusters: clusterItems,
            metrics: { requests: 0, failures: 0, in_flight: 0, p95_ms: 0, error_rate_pct: 0.0 },
            policies: policiesList,
            replace_prefix_match: target.replace_prefix_match || null,
            health_ratio: '1/1',
            status: 'healthy'
          });
        });
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
          polList.innerHTML = '<div style="color:var(--muted);font-size:11.5px;padding:8px">No policies attached</div>';
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
      const renderers = {
        routes: renderServices,
        services: renderServices,
        llm: renderLlm,
        subscription: renderLlm,
        provider: renderLlm,
        model: renderLlm,
        mcp: renderMcp,
        server: renderMcp,
        a2a: renderA2a,
        configuration: renderConfiguration
      };
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
      let lookupTab = tab;
      if ((lookupTab === 'services' || lookupTab === 'core') && !document.querySelector('.nav button[data-tab="' + lookupTab + '"]')) {
        lookupTab = 'routes';
      }
      const button = document.querySelector('.nav button[data-tab="' + lookupTab + '"]') || document.querySelector('.nav button[data-tab="overview"]');
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

        let targetId = 'tab-' + button.dataset.tab;
        if (button.dataset.tab === 'routes' || button.dataset.tab === 'services') {
          targetId = $('tab-routes') ? 'tab-routes' : 'tab-services';
        } else if (button.dataset.tab === 'subscription') {
          targetId = 'tab-llm';
          state.llmMode = 'subscription';
        } else if (button.dataset.tab === 'provider' || button.dataset.tab === 'model') {
          targetId = 'tab-llm';
          state.llmMode = 'api';
        } else if (button.dataset.tab === 'server') {
          targetId = 'tab-mcp';
        }

        const targetSec = $(targetId);
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
        if (['llm', 'subscription', 'provider', 'model'].includes(state.tab)) endpoints.push({ key: 'llmData', url: '/debug/llm' });
        if (state.tab === 'services' || state.tab === 'routes') endpoints.push({ key: 'servicesData', url: '/debug/services' });
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
