import {modPlatformStatusRequest, modPlatformSearchRequest, modPlatformVersionsRequest, modPlatformInstallRequest} from "@/api/modplatform.js";
import {useEffect, useState} from "react";
import {Button, Card, Input, Modal, Select, Table, Tag, message} from "antd";
import {CloudIcon, DownloadIcon, GlobeIcon, SearchIcon} from "lucide-react";

const Index = () => {
  const [status, setStatus] = useState(null);
  const [loading, setLoading] = useState(false);
  const [query, setQuery] = useState("");
  const [platform, setPlatform] = useState("all");
  const [gameVersion, setGameVersion] = useState("");
  const [modLoader, setModLoader] = useState("");
  const [results, setResults] = useState([]);
  const [messageApi, contextHolder] = message.useMessage();

  // 版本选择弹窗
  const [versionModalOpen, setVersionModalOpen] = useState(false);
  const [versionLoading, setVersionLoading] = useState(false);
  const [versions, setVersions] = useState([]);
  const [selectedMod, setSelectedMod] = useState(null);
  const [installingId, setInstallingId] = useState(null);

  const getStatus = async () => {
    const {code, msg, data} = await modPlatformStatusRequest();
    if (code === 1) {
      setStatus(data);
    }
  };

  const doSearch = async () => {
    if (!query.trim()) {
      messageApi.warning("请输入搜索关键词");
      return;
    }
    setLoading(true);
    const {code, msg, data} = await modPlatformSearchRequest(query, platform, gameVersion || undefined, modLoader || undefined);
    setLoading(false);
    if (code === 1) {
      setResults(data || []);
    } else {
      messageApi.error(msg || "搜索失败");
    }
  };

  const showVersions = async (item) => {
    setSelectedMod(item);
    setVersionModalOpen(true);
    setVersionLoading(true);
    setVersions([]);
    const {code, msg, data} = await modPlatformVersionsRequest(
      item.platform,
      item.platform_id,
      gameVersion || undefined,
      modLoader || undefined
    );
    setVersionLoading(false);
    if (code === 1) {
      setVersions(data || []);
    } else {
      messageApi.error(msg || "获取版本列表失败");
    }
  };

  const doInstall = async (version) => {
    if (!version?.download_url) {
      messageApi.error("该版本无下载链接");
      return;
    }
    setInstallingId(version.version_id);
    const {code, msg, data} = await modPlatformInstallRequest(
      version.platform,
      version.mod_name,
      version.download_url,
      version.filename
    );
    setInstallingId(null);
    if (code === 1) {
      messageApi.success(`下载成功: ${data.file_path} (${(data.file_size / 1024 / 1024).toFixed(2)} MB)`);
      setVersionModalOpen(false);
      setVersions([]);
    } else {
      messageApi.error(msg || "下载失败");
    }
  };

  useEffect(() => {
    getStatus();
  }, []);

  const platformColor = () => {
    return "green";
  };

  const platformIcon = () => {
    return "MR";
  };

  const versionColumns = [
    {
      title: "版本号",
      dataIndex: "version_number",
      key: "version_number",
      width: 160,
    },
    {
      title: "游戏版本",
      dataIndex: "game_versions",
      key: "game_versions",
      render: (versions) => (
        <div className="flex flex-wrap gap-1">
          {versions?.map((v) => (
            <Tag key={v} className="text-xs">{v}</Tag>
          ))}
        </div>
      ),
    },
    {
      title: "加载器",
      dataIndex: "mod_loaders",
      key: "mod_loaders",
      width: 140,
      render: (loaders) => (
        <div className="flex flex-wrap gap-1">
          {loaders?.map((l) => (
            <Tag key={l} color="blue" className="text-xs">{l}</Tag>
          ))}
        </div>
      ),
    },
    {
      title: "类型",
      dataIndex: "release_type",
      key: "release_type",
      width: 80,
      render: (type) => {
        const color = type === "release" ? "green" : type === "beta" ? "orange" : "red";
        return <Tag color={color}>{type}</Tag>;
      },
    },
    {
      title: "大小",
      dataIndex: "file_size",
      key: "file_size",
      width: 90,
      render: (size) => size ? `${(size / 1024 / 1024).toFixed(1)} MB` : "-",
    },
    {
      title: "操作",
      key: "action",
      width: 120,
      render: (_, record) => (
        <Button
          type="primary"
          size="small"
          icon={<DownloadIcon size={14}/>}
          loading={installingId === record.version_id}
          disabled={!record.download_url}
          onClick={() => doInstall(record)}
        >
          下载
        </Button>
      ),
    },
  ];

  return (
    <>
      {contextHolder}
      <div className="p-10 min-h-screen space-y-6">
        <div className="text-2xl font-bold text-indigo-600 flex items-center gap-2">
          <CloudIcon size={24} strokeWidth={1.5}/>
          模组平台
        </div>

        {status && (
          <div className="flex gap-4">
            <Card size="small" className="shadow-[0_4px_6px_rgba(0,0,0,0.1)]">
              <div className="flex items-center gap-2">
                <span className="text-sm text-gray-500">Modrinth:</span>
                <Tag color={status.modrinth_configured ? "success" : "error"}>
                  {status.modrinth_configured ? "已配置" : "未配置"}
                </Tag>
              </div>
            </Card>
          </div>
        )}

        <Card title="搜索模组" className="shadow-[0_4px_6px_rgba(0,0,0,0.1)]">
          <div className="flex flex-wrap gap-3 items-end">
            <div className="flex-1 min-w-[200px]">
              <div className="text-sm text-gray-500 mb-1">关键词</div>
              <Input
                placeholder="输入模组名称..."
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onPressEnter={doSearch}
                prefix={<SearchIcon size={14}/>}
              />
            </div>
            <div>
              <div className="text-sm text-gray-500 mb-1">平台</div>
              <Select value={platform} onChange={setPlatform} style={{width: 140}}>
                <Select.Option value="modrinth">Modrinth</Select.Option>
              </Select>
            </div>
            <div>
              <div className="text-sm text-gray-500 mb-1">游戏版本</div>
              <Input
                placeholder="如 1.20.1"
                value={gameVersion}
                onChange={(e) => setGameVersion(e.target.value)}
                style={{width: 120}}
              />
            </div>
            <div>
              <div className="text-sm text-gray-500 mb-1">加载器</div>
              <Select value={modLoader} onChange={setModLoader} style={{width: 120}} allowClear>
                <Select.Option value="">不限</Select.Option>
                <Select.Option value="forge">Forge</Select.Option>
                <Select.Option value="neoforge">NeoForge</Select.Option>
                <Select.Option value="fabric">Fabric</Select.Option>
                <Select.Option value="quilt">Quilt</Select.Option>
              </Select>
            </div>
            <div>
              <Button type="primary" onClick={doSearch} loading={loading} icon={<SearchIcon size={14}/>}>
                搜索
              </Button>
            </div>
          </div>
        </Card>

        {results.length > 0 && (
          <Card title="搜索结果" className="shadow-[0_4px_6px_rgba(0,0,0,0.1)]">
            <div className="space-y-3">
              {results.map((item, idx) => (
                <div key={idx}
                     className="flex items-start gap-4 p-4 border rounded-lg hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors">
                  {item.logo_url && (
                    <img src={item.logo_url} alt="" className="w-14 h-14 rounded object-cover flex-none"/>
                  )}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-base truncate">{item.name}</span>
                      <Tag color={platformColor(item.platform)}>{platformIcon(item.platform)}</Tag>
                    </div>
                    {item.description && (
                      <p className="text-sm text-gray-500 mt-1 line-clamp-2">{item.description}</p>
                    )}
                    <div className="flex items-center gap-3 mt-2 text-xs text-gray-400">
                      <span>作者: {item.author || "未知"}</span>
                      {item.downloads > 0 && <span>下载: {item.downloads.toLocaleString()}</span>}
                      {item.game_versions?.length > 0 && (
                        <span>版本: {item.game_versions.slice(0, 3).join(", ")}{item.game_versions.length > 3 ? "..." : ""}</span>
                      )}
                    </div>
                  </div>
                  <div className="flex-none self-center">
                    <Button
                      type="primary"
                      icon={<DownloadIcon size={14}/>}
                      onClick={() => showVersions(item)}
                    >
                      安装
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </Card>
        )}

        {results.length === 0 && !loading && query && (
          <div className="text-center text-gray-400 py-8">
            <GlobeIcon size={48} className="mx-auto mb-2 opacity-40"/>
            <p>暂无搜索结果</p>
            {!status?.modrinth_configured && (
              <p className="text-sm mt-1">请先在配置文件中设置 Modrinth API Token</p>
            )}
          </div>
        )}
      </div>

      <Modal
        title={selectedMod ? `选择版本 - ${selectedMod.name}` : "选择版本"}
        open={versionModalOpen}
        onCancel={() => {
          setVersionModalOpen(false);
          setVersions([]);
        }}
        footer={null}
        width={800}
      >
        <Table
          dataSource={versions}
          columns={versionColumns}
          rowKey="version_id"
          loading={versionLoading}
          size="small"
          pagination={{pageSize: 10, showSizeChanger: false}}
        />
      </Modal>
    </>
  );
};

export default Index;
