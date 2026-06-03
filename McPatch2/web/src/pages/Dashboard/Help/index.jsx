import {Collapse, Tag, Table, Alert} from "antd";

const {Panel} = Collapse;

const steps = [
  {key: 'step1', title: '下载文件'},
  {key: 'step2', title: '配置管理端'},
  {key: 'step3', title: '配置模组平台'},
  {key: 'step4', title: '放入要更新的文件'},
  {key: 'step5', title: '打包'},
  {key: 'step6', title: '后续打包'},
  {key: 'step7', title: '启动服务端'},
  {key: 'step8', title: '配置客户端'},
  {key: 'step9', title: '一键启动'},
  {key: 'step10', title: '常见问题'},
];

const clientComparisonColumns = [
  {key: 'client', title: '客户端', dataIndex: 'client'},
  {key: 'lang', title: '编写语言', dataIndex: 'lang'},
  {key: 'pros', title: '优势', dataIndex: 'pros'},
  {key: 'cons', title: '劣势', dataIndex: 'cons'},
];

const clientComparisonData = [
  {
    key: 'exe',
    client: 'exe 客户端',
    lang: 'Rust',
    pros: 'x86 平台最佳性能和稳定性',
    cons: '不支持 Win7 或其它系统',
  },
  {
    key: 'jar',
    client: 'jar 客户端',
    lang: 'Java',
    pros: '多平台支持（LinuxArm、Win7、macOS、手机端部分支持）',
    cons: '性能略逊于 exe',
  },
];

const cdDownloadColumns = [
  {key: 'type', title: '文件类型', dataIndex: 'type'},
  {key: 'method', title: '下载方式', dataIndex: 'method'},
];

const cdDownloadData = [
  {key: 'cdn', type: '有 CDN 地址的文件', method: '直接从 Modrinth 的 CDN 下载完整文件'},
  {key: 'traditional', type: '无 CDN 地址的文件', method: '走传统的更新包 Range 分片下载'},
];

const Index = () => {
  const scrollToSection = (key) => {
    const el = document.getElementById(key);
    if (el) el.scrollIntoView({behavior: 'smooth', block: 'start'});
  };

  return (
    <div className="p-10 min-h-screen">
      <div className="max-w-[900px] mx-auto">

        {/* Header */}
        <div className="text-center mb-12 px-8 py-12 rounded-2xl text-white relative overflow-hidden"
             style={{
               background: 'linear-gradient(135deg, #4f46e5 0%, #7c3aed 50%, #a855f7 100%)'
             }}>
          <div className="absolute inset-0 opacity-[0.08]"
               style={{
                 background: 'radial-gradient(circle, rgba(255,255,255,0.8) 0%, transparent 60%)',
                 backgroundSize: '200% 200%',
               }}/>
          <span className="inline-block bg-white/20 backdrop-blur px-3.5 py-1 rounded-full text-sm font-semibold mb-4 relative">
            V2.1
          </span>
          <h1 className="text-4xl font-extrabold mb-3 relative">McPatch 安装教程</h1>
          <p className="text-lg opacity-90 relative">
            基于 V2 原版教程，针对 V2.1 新增模组平台集成功能（Modrinth）进行了更新
          </p>
        </div>

        {/* TOC */}
        <div className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-6 mb-10 shadow-sm">
          <div className="text-lg font-bold text-indigo-600 mb-3">目录</div>
          <ol className="list-none">
            {steps.map((s, i) => (
              <li key={s.key} className="py-1">
                <span className="text-indigo-600 font-bold mr-1">{i + 1}.</span>
                <a className="text-gray-700 dark:text-gray-300 no-underline cursor-pointer hover:text-indigo-600 transition-colors"
                   onClick={() => scrollToSection(s.key)}>
                  {s.title}
                </a>
                {(s.key === 'step3' || s.key === 'step10') && (
                  <span className="inline-block bg-green-600 text-white text-xs px-1.5 py-0.5 rounded ml-1.5 font-semibold align-middle">NEW</span>
                )}
              </li>
            ))}
          </ol>
        </div>

        {/* Step 1 */}
        <section id="step1" className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-8 mb-6 shadow-sm">
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">1</span>
            下载文件
          </h2>
          <p className="mb-3.5">安装需要客户端和管理端，可以从这些地方下载：</p>
          <ul className="ml-6 mb-3.5 space-y-1.5">
            <li>
              Github Releases（
              <a href="https://github.com/BalloonUpdate/McPatch2/releases" target="_blank" rel="noreferrer"
                 className="text-indigo-600 hover:text-indigo-400 font-medium">管理端</a>、
              <a href="https://github.com/BalloonUpdate/Mcpatch2JavaClient/releases" target="_blank" rel="noreferrer"
                 className="text-indigo-600 hover:text-indigo-400 font-medium">jar客户端</a>、
              <a href="https://github.com/BalloonUpdate/Mcpatch2RustClient/releases" target="_blank" rel="noreferrer"
                 className="text-indigo-600 hover:text-indigo-400 font-medium">exe客户端</a>）
            </li>
            <li>
              <a href="https://mcpatch.hoshiroko.com/" target="_blank" rel="noreferrer"
                 className="text-indigo-600 hover:text-indigo-400 font-medium">hoshiroko.com</a>
              （感谢<a href="https://hoshiroko.com/" className="text-indigo-600 hover:text-indigo-400 font-medium">@薄荷の尾巴</a>提供）
            </li>
          </ul>
          <p className="mb-3.5">
            管理端程序的文件名通常叫 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">mcpatch-manager</code> 或者直接简写为 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">m</code>。是用来打更新包和进行日常维护工作的。同时也提供一个内置的开箱即用服务端方便上手。管理端通常放在自己电脑上，或者服务器上运行。
          </p>
          <p className="mb-3.5">v2 版本目前有两个客户端可供选择：</p>
          <Table columns={clientComparisonColumns} dataSource={clientComparisonData} pagination={false} size="small"
                 className="mb-4" bordered/>
          <Alert type="success" showIcon message={
            <span><strong>选择建议：</strong>优先选择 exe 客户端，如果不能满足需求，使用 jar 客户端作为备选。</span>
          } className="mb-3"/>
          <Alert type="info" showIcon message={
            <span>如果提示找不到 VCRUNTIME140.dll 文件，说明需要 <a href="https://balloonupdate.github.io/McPatchDocs/docs/v2/faq#%E7%94%B1%E4%BA%8E%E6%89%BE%E4%B8%8D%E5%88%B0VCRUNTIME140.dll%EF%BC%8C%E6%97%A0%E6%B3%95%E7%BB%A7%E7%BB%AD%E6%89%A7%E8%A1%8C%E4%BB%A3%E7%A0%81" target="_blank" rel="noreferrer" className="text-indigo-600 font-medium">安装 VC++ 2015 运行库</a>。</span>
          }/>
        </section>

        {/* Step 2 */}
        <section id="step2" className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-8 mb-6 shadow-sm">
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">2</span>
            配置管理端
          </h2>
          <p className="mb-3.5">管理端的作用是打包和管理更新包，日常维护更新文件时，都需要和它打交道。</p>
          <p className="mb-3.5">
            首先在桌面上（或者任何你喜欢的地方）创建一个目录，叫 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">guanli</code>。然后将刚刚下载好的管理端程序放进去。
          </p>

          <h3 className="text-xl font-bold mt-7 mb-3.5">WebUI 模式（推荐）</h3>
          <p className="mb-3.5">
            双击即可启动管理端。如果启动成功后你看到有 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">&gt; </code> 的字样，说明进入了命令行模式，只需要输入 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">webui</code> 再按 Enter 键就能进入 WebUI 模式。（或者直接使用启动参数 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">m.exe webui</code> 来直接进入 WebUI 模式）
          </p>
          <p className="mb-3.5">
            启动成功之后，会自动创建配置文件和生成一组默认的账号密码。需要复制密码到安全的地方妥善保存，因为<strong>密码只会显示一次</strong>。
          </p>
          <p className="mb-3.5">
            管理端默认会同时启动 WebUI 服务（端口 6710）和私有协议服务（端口 6700）。打开浏览器，输入 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">http://127.0.0.1:6710</code> 即可打开 WebUI 界面。
          </p>
          <Alert type="success" showIcon message={
            <span>如果你忘记了用户名或者是密码，可以删除 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">user.toml</code> 文件后，重启管理端，就可以重新生成账号密码。</span>
          } className="mb-4"/>
          <p className="mb-3.5">
            回到 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">guanli</code> 目录下，你会注意到管理端程序旁边新增了一个名为 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">workspace</code> 的文件夹，这就是工作空间目录。
          </p>
          <p className="mb-2">此时，<code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">guanli</code> 目录大概长这样：</p>
          <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed mb-4 font-mono">
guanli/
├─ workspace/      # 工作空间目录
├─ public/         # 公共目录
├─ config.toml     # 管理端配置文件
├─ user.toml       # 管理端的认证相关配置文件
└─ m.exe           # 管理端程序
          </pre>

          <h3 className="text-xl font-bold mt-7 mb-3.5">命令行模式</h3>
          <p className="mb-3.5">
            双击即可启动管理端。启动成功后可以看到有 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">&gt; </code> 的字样，此时进入了交互式命令行模式。按回车就可以出现管理端支持的命令列表。
          </p>
          <p className="mb-3.5">
            我们输出 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">check</code> 命令按下回车键，它会提示修改的文件数量，我们先忽视这个消息。
          </p>
          <p className="mb-2">此时，<code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">guanli</code> 目录大概长这样：</p>
          <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed mb-4 font-mono">
guanli/
├─ workspace/      # 工作空间目录
├─ public/         # 公共目录
└─ m-0.0.11-x86_64-pc-windows-msvc.exe  # 管理端程序
          </pre>
          <p className="mb-3.5">
            无论是要向客户端添加新文件、修改现有文件、删除文件、移动文件，你只需要在 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">workspace</code> 目录中对相应的文件进行操作。每当管理端进行打包操作时，管理端会对比 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">workspace</code> 目录中文件的变化，来生成更新包。
          </p>
        </section>

        {/* Step 3 - NEW */}
        <section id="step3"
                 className="bg-white dark:bg-dark-2 border border-green-200 dark:border-green-800 rounded-lg p-8 mb-6 shadow-sm"
                 style={{borderLeft: '4px solid #059669'}}>
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">3</span>
            配置模组平台
            <Tag color="green" className="ml-2">V2.1 NEW</Tag>
          </h2>
          <Alert type="info" showIcon message={
            <span>这是 V2.1 版本新增的功能。如果你不需要模组平台集成，可以跳过此步骤，不影响基本使用。</span>
          } className="mb-4"/>

          <p className="mb-3.5">
            V2.1 版本新增了<strong>模组平台集成</strong>功能，支持 <Tag color="green">Modrinth</Tag> 模组平台。配置后，管理端在打包时会自动识别工作空间中的模组文件来源，并在更新元数据中记录 CDN 下载地址，客户端可以直接从平台 CDN 下载模组文件，从而<strong>减少自建服务器的带宽消耗</strong>。
          </p>

          <h3 className="text-xl font-bold mt-7 mb-3.5">3.1 功能概述</h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3.5 mb-5">
            {[
              {icon: '🔍', label: '模组搜索', desc: '在 WebUI 中搜索 Modrinth 上的模组'},
              {icon: '🧬', label: '自动识别', desc: '打包时自动识别 .jar / .zip 文件的模组来源'},
              {icon: '⚡', label: 'CDN 直链下载', desc: '客户端直接从平台 CDN 下载模组，不走自建服务器带宽'},
              {icon: '🔄', label: '双源降级', desc: 'CDN 下载失败时自动降级为传统更新包下载'},
            ].map(item => (
              <div key={item.label}
                   className="bg-gray-50 dark:bg-dark-3 border border-gray-200 dark:border-gray-700 rounded-lg p-4 text-center">
                <div className="text-3xl mb-2">{item.icon}</div>
                <div className="font-bold text-sm mb-1">{item.label}</div>
                <div className="text-xs text-gray-500 dark:text-gray-400">{item.desc}</div>
              </div>
            ))}
          </div>

          <h3 className="text-xl font-bold mt-7 mb-3.5">3.2 获取 API 密钥</h3>
          <h4 className="text-lg font-bold mt-5 mb-2.5"><Tag color="green">Modrinth</Tag></h4>
          <p className="mb-3.5">
            Modrinth 的 API 是公开的，<strong>不需要 API Token 即可使用基本功能</strong>（搜索、版本查询、哈希查找）。如果你需要更高的请求频率，可以在 <a href="https://modrinth.com/settings" target="_blank" rel="noreferrer" className="text-indigo-600 hover:text-indigo-400 font-medium">Modrinth 设置页面</a> 生成一个 API Token。
          </p>

          <h3 className="text-xl font-bold mt-7 mb-3.5">3.3 编辑配置文件</h3>
          <p className="mb-3.5">
            打开管理端的配置文件 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">config.toml</code>，在文件末尾添加以下配置段：
          </p>
          <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed mb-4 font-mono relative">
<span className="text-purple-400">[core.mod-platform]</span>

<span className="text-purple-400">[core.mod-platform.modrinth]</span>
api-token = ""                         <span className="text-gray-500"># 可选，留空使用匿名访问</span>
rate-limit = 30                        <span className="text-gray-500"># 每秒最大请求数，默认30</span>

<span className="text-purple-400">[core.mod-platform.cache]</span>
ttl-searches = 600          <span className="text-gray-500"># 搜索结果缓存时间（秒），默认10分钟</span>
ttl-mod-detail = 3600       <span className="text-gray-500"># 模组详情缓存时间（秒），默认1小时</span>
ttl-versions = 3600         <span className="text-gray-500"># 版本列表缓存时间（秒），默认1小时</span>
ttl-download = 300          <span className="text-gray-500"># 下载URL缓存时间（秒），默认5分钟</span>
ttl-fingerprint = 86400     <span className="text-gray-500"># 指纹缓存时间（秒），默认24小时</span>
max-entries = 500           <span className="text-gray-500"># 每类缓存最大条目数</span>
          </pre>

          <h4 className="text-lg font-bold mt-5 mb-2.5">国内网络环境配置（可选）</h4>
          <p className="mb-3.5">如果服务器在国内网络环境下无法直接访问 Modrinth API，可以配置代理：</p>
          <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed mb-4 font-mono">
<span className="text-purple-400">[core.mod-platform.proxy]</span>
host = "127.0.0.1"
port = 7890
          </pre>

          <h3 className="text-xl font-bold mt-7 mb-3.5">3.4 在 WebUI 中使用模组平台</h3>
          <p className="mb-3.5">
            配置完成后，重启管理端。打开 WebUI 界面，在左侧导航栏中可以看到新增的<strong>「模组平台」</strong>菜单项（云朵图标 ☁️）。
          </p>
          <p className="mb-2">点击进入后，你可以：</p>
          <ol className="ml-6 space-y-1.5 mb-4">
            <li><strong>查看平台状态</strong>：页面顶部会显示 Modrinth 的配置状态</li>
            <li><strong>搜索模组</strong>：输入关键词，可选填游戏版本和模组加载器（Forge / NeoForge / Fabric / Quilt）</li>
            <li><strong>查看搜索结果</strong>：卡片式展示模组图标、名称、描述和下载量</li>
            <li><strong>安装模组</strong>：点击「安装」按钮查看可用版本列表，选择版本后点击「下载」即可将模组文件下载到工作空间目录中</li>
          </ol>
          <Alert type="info" showIcon message={
            <span>如果页面显示平台未配置，请先按照 3.3 节编辑配置文件并重启管理端。</span>
          }/>
        </section>

        {/* Step 4 */}
        <section id="step4" className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-8 mb-6 shadow-sm">
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">4</span>
            放入要更新的文件
          </h2>
          <p className="mb-3.5">
            一般来说，服务器整合包的首包是服主制作好后，通过链接或者QQ群文件分享给玩家的，而后续的文件更新，则是使用 McPatch 进行远程推送。
          </p>
          <p className="mb-3.5">
            现在把目前需要更新的文件全部放到工作空间目录里，暂时不更新的文件以后再放，这能减少更新包的大小。
          </p>
          <Alert type="success" showIcon message={
            <span>如果你使用 WebUI 版本，可以使用左侧菜单中的在线文件管理功能直接上传，也可以将文件手动添加到工作空间目录里。</span>
          } className="mb-4"/>

          <h3 className="text-xl font-bold mt-7 mb-3.5">更新模组</h3>
          <p className="mb-3.5">要更新模组文件的话：</p>
          <ul className="ml-6 space-y-1.5 mb-3.5">
            <li>复制 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">.minecraft/mods</code> 目录下你想要更新的文件</li>
            <li>粘贴到 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">guanli/workspace/.minecraft/mods</code> 下</li>
          </ul>
          <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed mb-4 font-mono">
客户端整合包目录结构：
Minecraft客户端/
└─ .minecraft/
   └─ mods/
      ├─ 模组a.jar （复制）
      ├─ 模组b.jar （复制）
      └─ 模组c.jar （复制）

管理端目录结构：
guanli/
└─ workspace/
   └─ .minecraft/
      └─ mods/
         ├─ 模组a.jar （粘贴过来）
         ├─ 模组b.jar （粘贴过来）
         └─ 模组c.jar （粘贴过来）
          </pre>
          <Alert type="success" showIcon message={
            <span><strong>🆕 V2.1 提示：</strong>如果你配置了模组平台，打包时管理端会自动识别这些 .jar 文件的来源（Modrinth），并在更新元数据中记录 CDN 下载地址。玩家更新时，这些模组文件会直接从平台 CDN 下载，节省你的服务器带宽。</span>
          }/>

          <h3 className="text-xl font-bold mt-7 mb-3.5">更新启动器旁边的文件</h3>
          <p className="mb-3.5">要更新启动器文件旁边的 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">新玩家进服教程.txt</code> 的话：</p>
          <ul className="ml-6 space-y-1.5 mb-3.5">
            <li>复制：<code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">新玩家进服教程.txt</code> 文件</li>
            <li>粘贴到：<code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">guanli/workspace/新玩家进服教程.txt</code></li>
          </ul>

          <h3 className="text-xl font-bold mt-7 mb-3.5">更新模组配置文件（开启版本隔离）</h3>
          <p className="mb-3.5">要更新 JEI 模组的配置文件（开启版本隔离）：</p>
          <ul className="ml-6 space-y-1.5 mb-3.5">
            <li>复制 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">.minecraft/versions/&lt;版本名称&gt;/config/jei/jei.cfg</code> 文件</li>
            <li>粘贴到 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">guanli/workspace/.minecraft/versions/&lt;版本名称&gt;/config/jei/jei.cfg</code></li>
          </ul>
          <Alert type="warning" showIcon message={
            <span>
              如果你开了版本隔离，模组文件通常是在 versions 目录下的。那么你要更新的模组就不是在 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">.minecraft/mods</code> 下的文件了，而是在 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">.minecraft/versions/&lt;版本名称&gt;/mods</code> 目录下。如果你还把文件直接丢到 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">.minecraft/mods</code> 目录下，虽然也能更新，但游戏读取不到。
            </span>
          }/>
        </section>

        {/* Step 5 */}
        <section id="step5" className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-8 mb-6 shadow-sm">
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">5</span>
            打包
          </h2>

          <h3 className="text-xl font-bold mt-7 mb-3.5">WebUI 模式（推荐）</h3>
          <p className="mb-3.5">点击左侧菜单切换到终端界面。点击「打包新版本」按钮，输入版本号和更新记录即可。</p>

          <h3 className="text-xl font-bold mt-7 mb-3.5">命令行模式</h3>
          <p className="mb-3.5">
            切回管理端窗口。（如果你已经退出了，那么再次双击运行即可）看到 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">&gt; </code> 的字样后，输入 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">pack &lt;version&gt;</code> 即可开始打包。
          </p>

          <Alert type="info" showIcon message={
            <span>
              版本号只能包括大小写字母，数字，以及 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">!@#$()_+-=;',.</code> 这几个符号，不要使用中文、空格或其它字符。<br/>
              如果要写版本更新记录，需要在打包开始之前，提前把更新记录用 utf-8 编码写在 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">guanli/logs.txt</code> 文件里。
            </span>
          } className="mb-6"/>

          {/* V2.1 模组识别 */}
          <div className="border-l-4 border-green-600 bg-green-50 dark:bg-green-900/20 p-5 rounded-r-lg mb-6">
            <h3 className="text-xl font-bold text-green-700 dark:text-green-400 mt-0 mb-3">🆕 V2.1 模组识别</h3>
            <p className="mb-3.5">
              如果你配置了模组平台，打包过程中管理端会自动执行<strong>模组识别</strong>步骤：
            </p>
            <div className="flex items-center gap-2 flex-wrap mb-3">
              <span className="bg-indigo-600 text-white px-3.5 py-1.5 rounded-full text-sm font-semibold">.jar 文件</span>
              <span className="text-gray-400 text-lg">→</span>
              <span className="bg-indigo-600 text-white px-3.5 py-1.5 rounded-full text-sm font-semibold">SHA-1 哈希</span>
              <span className="text-gray-400 text-lg">→</span>
              <span className="text-green-600 font-semibold text-sm">✓ Modrinth</span>
            </div>
            <div className="flex items-center gap-2 flex-wrap mb-3">
              <span className="bg-indigo-600 text-white px-3.5 py-1.5 rounded-full text-sm font-semibold">.zip 文件</span>
              <span className="text-gray-400 text-lg">→</span>
              <span className="bg-indigo-600 text-white px-3.5 py-1.5 rounded-full text-sm font-semibold">SHA-1 哈希</span>
              <span className="text-gray-400 text-lg">→</span>
              <span className="text-green-600 font-semibold text-sm">✓ Modrinth</span>
            </div>
            <p className="mb-3.5">识别成功的模组文件，会在更新元数据中记录 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">source</code>（来源平台）和 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">downloadUrl</code>（CDN 下载地址）。</p>
            <p className="mb-2">打包日志中会显示类似以下信息：</p>
            <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed mb-3 font-mono">
模组平台识别已启用
  识别为: Sodium (来自 modrinth)
  识别为: 自定义模组.jar (未识别，将走传统更新包下载)</pre>
            <Alert type="info" showIcon message={
              <span>模组识别是可选步骤，识别失败的文件仍然会正常打包，只是不会记录 CDN 下载地址，客户端会走传统的更新包下载方式。</span>
            }/>
          </div>

          <p className="mb-3.5">
            看到「打包完成！」的字样即代表打包成功了，更新包会存放在 public 目录下，会按版本号进行命名。
          </p>
          <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed mb-4 font-mono">
guanli/
├─ public/          # 存放更新包
│  ├─ v1.tar
│  └─ index.json
├─ workspace/       # 工作空间
│  └─ 各种文件
└─ m.exe            # 管理端程序
          </pre>

          <h3 className="text-xl font-bold mt-7 mb-3.5">校验更新包</h3>
          <ul className="ml-6 space-y-1.5 mb-4">
            <li><strong>WebUI 模式</strong>：点击「测试更新包」按钮对更新包进行校验</li>
            <li><strong>命令行模式</strong>：使用 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">test</code> 命令对更新包进行校验</li>
          </ul>
          <Alert type="error" showIcon message={
            <span><strong>⚠️ 万分小心！！！</strong> 已经打好的更新包文件（也就是 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">public</code> 目录里面的文件）不能手动删除，不能修改里面的文件，也不能重新打包，这会导致数据损坏，应该再打一个新的包来替代删除旧的包。</span>
          }/>
        </section>

        {/* Step 6 */}
        <section id="step6" className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-8 mb-6 shadow-sm">
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">6</span>
            后续打包
          </h2>
          <p className="mb-3.5">
            后续的文件维护很简单！直接对工作空间目录下的文件或者目录进行增加，删除，修改替换，移动，重命名都可。想怎么操作怎么操作。
          </p>
          <ul className="ml-6 space-y-1.5 mb-3.5">
            <li><strong>WebUI 模式</strong>：修改到自己满意为止后，点击「打包新版本」按钮即可。</li>
            <li><strong>命令行模式</strong>：修改到自己满意为止后，输入 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">pack &lt;version&gt;</code> 打包即可。</li>
          </ul>
          <p className="mb-3.5">
            管理端会自动感知到你做的所有文件修改，并打成更新包，再将这些修改同步到客户端。整个过程操作起来方便和直观，只需要把工作空间目录当成普通目录一样修改就好了。
          </p>

          <h3 className="text-xl font-bold mt-7 mb-3.5">回退工作空间</h3>
          <p className="mb-3.5">如果对工作空间目录下的文件进行了修改，但又发现改的不对：</p>
          <ul className="ml-6 space-y-1.5 mb-3.5">
            <li><strong>WebUI 模式</strong>：可以点击「回退工作空间」按钮来丢弃刚刚的修改</li>
            <li><strong>命令行模式</strong>：可以使用管理端的 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">revert</code> 命令来丢弃刚刚的修改</li>
          </ul>

          <h3 className="text-xl font-bold mt-7 mb-3.5">常见问题</h3>
          <ol className="ml-6 space-y-1.5 mb-3.5">
            <li>支持所有文件操作：包括移动，修改，覆盖，新建，删除，复制等等等等，文件夹同理</li>
            <li>支持更新加密的压缩包。只需要当普通文件一样用新版替换覆盖旧版就行</li>
            <li>新旧文件同名，但是文件内容变了也能检测到。因为打包时会检查文件校验，文件里任何字节发生变化，管理端都能感知到</li>
            <li>避免单纯的将 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">Abc.jar</code> 重命名为 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">abc.jar</code> 这种仅修改大小写部分的行为，管理端会出 bug</li>
          </ol>

          <h3 className="text-xl font-bold mt-7 mb-3.5">删除不存在的文件</h3>
          <p className="mb-3.5">
            有些时候，你可能会想删掉一些文件，这些文件存在于玩家那边，但却不存在于管理端的工作空间目录里。
          </p>
          <p className="mb-3.5">
            最简单的方法就是将这些文件先放进工作空间目录正常打个包，这样工作空间目录里就有这个文件了，此时把它们删掉，再打个包，即可从客户端删除这些文件了。
          </p>
          <Alert type="success" showIcon message={
            <span>不用担心副作用。客户端会自动针对这种情况进行下载优化，以节省流量。文件夹也可以使用这个方法来删除，但注意玩家在文件夹中存放的自己的文件也会被顺带删除。</span>
          }/>
        </section>

        {/* Step 7 */}
        <section id="step7" className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-8 mb-6 shadow-sm">
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">7</span>
            启动服务端
          </h2>
          <p className="mb-3.5">
            管理端除了管理更新包以外，还自带一个内置服务端的功能。这是给小白用户准备的功能，不仅能免去额外搭建 HTTP 服务端的繁琐过程，还能免备案使用。
          </p>
          <p className="mb-3.5">
            客户端程序支持多种更新协议，比如 HTTP 协议、WebDAV 协议、私有协议等。这里我们使用管理端自带的服务端功能进行教程演示。
          </p>

          <h3 className="text-xl font-bold mt-7 mb-3.5">WebUI 模式</h3>
          <p className="mb-3.5">
            WebUI 版本默认会自动启动私有协议服务端，监听端口为 6700，默认情况下无需手动开启，可以直接使用。
          </p>

          <h3 className="text-xl font-bold mt-7 mb-3.5">命令行模式</h3>
          <p className="mb-3.5">
            要启动管理端自带的服务端功能，首先需要启动管理端，然后输入 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">serve</code> 按回车即可启动。（注意命令是 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">serve</code> 不是 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">server</code> 不要打错）
          </p>
          <p className="mb-3.5">
            默认的端口是 6700，如果需要调整，可修改管理端的配置文件 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">config.toml</code> 中的 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">[builtin-server].listen-port</code> 或者 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">serve-listen-port</code> 字段。
          </p>
          <p className="mb-3.5">想要关闭服务端可以叉掉终端的窗口，或者按 Ctrl + C 即可。</p>
          <Alert type="success" showIcon message={
            <span>管理端一般都是同时开两个管理端进程实例，一个打包用，用完叉掉。一个跑内置服务端，一直挂在后台。</span>
          } className="mb-4"/>
          <p className="mb-3.5">
            通常我们都习惯从 bat 启动这个自带服务端。在 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">guanli</code> 目录下新建一个 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">serve.bat</code> 文件：
          </p>
          <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed mb-4 font-mono">
@echo off
&lt;mcpatch-manager&gt; serve</pre>
          <Alert type="warning" showIcon message={
            <span>
              <strong>注意：</strong>
              配置文件中的所有 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">listen-addr</code> 参数请勿随意调整，应该始终保持 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">0.0.0.0</code>，否则会导致所有连接都连不上。<br/>
              如果需要在内网穿透环境下使用管理端自带的服务端功能，协议记得选择 TCP 协议。
            </span>
          }/>
        </section>

        {/* Step 8 */}
        <section id="step8" className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-8 mb-6 shadow-sm">
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">8</span>
            配置客户端
          </h2>
          <Alert type="success" showIcon message={
            <span><strong>推荐的步骤，但不强制：</strong>在开始配置客户端程序之前，建议先将 MC 客户端整个文件夹压缩一遍备份，避免调试更新的过程中，误删了什么文件没法恢复了。</span>
          } className="mb-4"/>
          <p className="mb-3.5">
            将客户端程序放到 Minecraft 客户端的 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">.minecraft/mcpatch</code> 目录里面（需要手动创建），然后直接双击运行 exe。
          </p>
          <p className="mb-3.5">
            不出意外的话，客户端会报错无法连接之类的错误，不用管这个报错，直接关掉。此时它会生成一个配置文件 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">mcpatch.yml</code>，我们需要打开编辑。
          </p>
          <p className="mb-3.5">
            这里唯一需要修改的是 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">urls</code> 选项，这个参数控制更新服务器的地址，默认是本机地址 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">mcpatch://127.0.0.1:6700</code>。
          </p>
          <ul className="ml-6 space-y-1.5 mb-3.5">
            <li>如果你的服务端是管理端自带的，且监听的端口也是默认的 6700 的话，那么这里可以不用修改端口</li>
            <li>如果你的服务端没有和客户端程序部署在同一台电脑上，那么记得修改这个 127.0.0.1 的 IP 地址</li>
          </ul>
          <p className="mb-3.5">
            <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">urls</code> 支持 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">base64</code> 编码。（例如 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">mcpatch://127.0.0.1:80</code> 的 base64 编码为 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">bWNwYXRjaDovLzEyNy4wLjAuMTo4MA==</code>）
          </p>

          <div className="border-l-4 border-green-600 bg-green-50 dark:bg-green-900/20 p-5 rounded-r-lg mb-4">
            <h3 className="text-xl font-bold text-green-700 dark:text-green-400 mt-0 mb-3">🆕 V2.1 CDN 直链下载</h3>
            <p className="mb-3.5">
              V2.1 版本的客户端新增了 <strong>CDN 直链下载</strong> 支持，<strong>无需额外配置</strong>。当管理端打包时识别到模组来源并记录了 CDN 下载地址后，客户端会自动优先从 CDN 下载模组文件：
            </p>
            <Table columns={cdDownloadColumns} dataSource={cdDownloadData} pagination={false} size="small"
                   className="mb-4" bordered/>
            <p className="mb-3.5">如果 CDN 下载失败，客户端会自动降级为传统的更新包下载方式，确保更新不会中断。</p>
            <Alert type="info" showIcon message={
              <span>
                <strong>向后兼容：</strong>V2.1 客户端完全兼容 V2.0 管理端生成的更新包。V2.0 管理端生成的元数据不含 CDN 下载地址，客户端会自动走传统下载方式，无需任何修改。
              </span>
            }/>
          </div>
        </section>

        {/* Step 9 */}
        <section id="step9" className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-8 mb-6 shadow-sm">
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">9</span>
            一键启动
          </h2>
          <p className="mb-3.5">客户端每次都要手动双击运行很是麻烦，可以借助一些方法在游戏启动时自动进行更新。</p>

          <h3 className="text-xl font-bold mt-7 mb-3.5">exe 客户端</h3>
          <ol className="ml-6 space-y-1.5 mb-3.5">
            <li>首先到 <a href="https://github.com/BalloonUpdate/McPatch2Loader/releases" target="_blank" rel="noreferrer"
                       className="text-indigo-600 hover:text-indigo-400 font-medium">BalloonUpdate/McPatch2Loader</a> 下载最新版的加载器文件</li>
            <li>打开 Minecraft 客户端的 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">.minecraft/mcpatch</code> 目录。将加载器放在里面，和客户端程序放到一起</li>
            <li>在此处创建「启动列表」文件，叫 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">startlist.txt</code></li>
            <li>将客户端程序的文件名复制（比如 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">mcpatch-client-0.0.0.exe</code>），粘贴到 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">startlist.txt</code> 里，保存关闭</li>
          </ol>
          <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed mb-4 font-mono">
客户端整合包/
├─ .minecraft/
│  ├─ mcpatch/
│  │  ├─ mcpatch-client-0.0.0.exe
│  │  ├─ startlist.txt
│  │  ├─ loader-2.jar
│  │  ├─ mcpatch.yml
│  │  └─ mcpatch.log
│  └─ versions/
└─ PCL启动器.exe</pre>
          <p className="mb-3.5">
            打开 Minecraft 启动器（任意启动器均可，官方启动器除外），调整游戏版本设置，找到 Java 虚拟机参数（或者叫 JVM 参数）。
          </p>
          <p className="mb-3.5">
            在参数的开头插入这串代码 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">-javaagent:mcpatch/《举个栗子》.jar </code>（注意 .jar 的后面还有个空格也不要漏），然后回到启动器主界面。
          </p>
          <Alert type="info" showIcon message={
            <span>如果你发现启动器有「JVM参数头」或者「JVM参数尾」这两个选项，则需要填到「JVM参数头」的<strong>最前面</strong>！</span>
          } className="mb-4"/>

          <h3 className="text-xl font-bold mt-7 mb-3.5">jar 客户端</h3>
          <p className="mb-3.5">
            打开 Minecraft 客户端的 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">.minecraft/mcpatch</code> 目录。将客户端 jar 文件移动到里面去。（记得把配置文件和版本号文件也一起移动）
          </p>
          <p className="mb-3.5">
            打开 Minecraft 启动器，调整游戏版本设置，找到 Java 虚拟机参数。在参数的开头插入 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">-javaagent:mcpatch/《举个栗子》.jar </code>。
          </p>

          <h3 className="text-xl font-bold mt-7 mb-3.5">版本隔离注意事项</h3>
          <p className="mb-3.5">
            如果启动游戏失败，且日志里有 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">Error opening zip file or JAR manifest missing</code> 的字样，说明填写的 jar 文件找不到。这通常和版本隔离选项有关。
          </p>
          <Table columns={[
            {key: 'isolation', title: '版本隔离', dataIndex: 'isolation'},
            {key: 'jvm', title: 'JVM 参数', dataIndex: 'jvm'},
            {key: 'desc', title: '说明', dataIndex: 'desc'},
          ]} dataSource={[
            {key: 'off', isolation: '未开启', jvm: '-javaagent:mcpatch/xxx.jar', desc: '从 .minecraft 目录下开始计算'},
            {key: 'on', isolation: '已开启', jvm: '-javaagent:../../mcpatch/xxx.jar', desc: '从 .minecraft/versions/xxxx/ 目录下开始计算'},
          ]} pagination={false} size="small" bordered/>
        </section>

        {/* Step 10 - FAQ */}
        <section id="step10" className="bg-white dark:bg-dark-2 border border-gray-200 dark:border-gray-700 rounded-lg p-8 mb-6 shadow-sm">
          <h2 className="text-2xl font-extrabold mb-5 pb-3 border-b-2 border-gray-200 dark:border-gray-700 flex items-center gap-2.5">
            <span className="inline-flex items-center justify-center w-9 h-9 bg-indigo-600 text-white rounded-full text-base font-bold shrink-0">10</span>
            常见问题
          </h2>

          <Collapse bordered={false} className="bg-transparent" expandIconPosition="end">
            <Panel header="文件更新位置" key="1">
              <p className="text-gray-500 dark:text-gray-400">
                客户端程序会自动搜索 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">.minecraft</code> 的父目录作为更新起始位置，到处移动客户端程序（不管套几层文件夹）都不会影响更新起始位置。如果要调整这个机制，可以修改配置文件里的 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">base-path</code> 选项。
              </p>
            </Panel>
            <Panel header="版本号不是判断版本新旧的依据" key="2">
              <p className="text-gray-500 dark:text-gray-400">
                版本号只是一个普通的标签，是给人类看的，程序不会解析对比版本号的文字。版本号的判断顺序按打包时间来计算，后打的版本总是新版本。
              </p>
            </Panel>
            <Panel header={<span>🆕 不配置模组平台会影响正常使用吗？</span>} key="3" className="border-l-4 border-green-500">
              <p className="text-gray-500 dark:text-gray-400">
                不会。模组平台集成是可选功能，不配置时管理端和客户端的行为与 V2.0 完全一致。
              </p>
            </Panel>
            <Panel header={<span>🆕 模组识别失败的文件会怎样？</span>} key="4" className="border-l-4 border-green-500">
              <p className="text-gray-500 dark:text-gray-400">
                识别失败的文件仍然会正常打包进更新包，客户端会走传统的更新包下载方式，不会影响更新流程。
              </p>
            </Panel>
            <Panel header={<span>🆕 CDN 下载失败怎么办？</span>} key="5" className="border-l-4 border-green-500">
              <p className="text-gray-500 dark:text-gray-400">
                客户端内置了自动降级机制。如果 CDN 下载失败，会自动重试（次数由 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">config.reties</code> 控制），重试全部失败后降级为传统的更新包 Range 分片下载。
              </p>
            </Panel>

            <Panel header={<span>🆕 Modrinth 需要 API Token 吗？</span>} key="7" className="border-l-4 border-green-500">
              <p className="text-gray-500 dark:text-gray-400">
                不需要。Modrinth 的 API 是公开的，匿名访问即可使用搜索、版本查询和哈希查找功能。只有在需要更高请求频率时才需要配置 API Token。
              </p>
            </Panel>
            <Panel header={<span>🆕 国内服务器无法访问 Modrinth API 怎么办？</span>} key="8" className="border-l-4 border-green-500">
              <div>
                <p className="text-gray-500 dark:text-gray-400 mb-3">
                  可以在 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">config.toml</code> 中配置代理：
                </p>
                <pre className="bg-gray-900 text-gray-200 p-4 rounded-lg overflow-x-auto text-sm leading-relaxed font-mono">
<span className="text-purple-400">[core.mod-platform.proxy]</span>
host = "127.0.0.1"
port = 7890</pre>
              </div>
            </Panel>
            <Panel header={<span>🆕 V2.1 客户端兼容 V2.0 管理端吗？</span>} key="9" className="border-l-4 border-green-500">
              <p className="text-gray-500 dark:text-gray-400">
                完全兼容。V2.0 管理端生成的更新元数据不含 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">source</code> 和 <code className="bg-gray-100 dark:bg-gray-800 text-indigo-600 px-1.5 py-0.5 rounded text-sm font-mono">downloadUrl</code> 字段，V2.1 客户端会自动走传统下载方式。V2.0 客户端也能使用 V2.1 管理端生成的更新包，新增字段会被忽略。
              </p>
            </Panel>
          </Collapse>
        </section>

        {/* Footer */}
        <div className="text-center pt-10 pb-5 text-gray-400 dark:text-gray-500 text-sm">
          <p>McPatch V2.1 安装教程 · 基于 <a href="https://balloonupdate.github.io/McPatchDocs/docs/v2/start" target="_blank" rel="noreferrer" className="text-indigo-600 hover:text-indigo-400 font-medium">McPatch V2 官方文档</a> 更新</p>
        </div>

      </div>
    </div>
  );
};

export default Index;
