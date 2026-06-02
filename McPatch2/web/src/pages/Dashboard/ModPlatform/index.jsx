import {modPlatformStatusRequest, modPlatformSearchRequest} from "@/api/modplatform.js";
import {useEffect, useState} from "react";
import {Button, Card, Input, Select, Tag, message} from "antd";
import {CloudIcon, GlobeIcon, SearchIcon} from "lucide-react";

const Index = () => {
  const [status, setStatus] = useState(null);
  const [loading, setLoading] = useState(false);
  const [query, setQuery] = useState("");
  const [platform, setPlatform] = useState("all");
  const [gameVersion, setGameVersion] = useState("");
  const [modLoader, setModLoader] = useState("");
  const [results, setResults] = useState([]);
  const [messageApi, contextHolder] = message.useMessage();

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

  useEffect(() => {
    getStatus();
  }, []);

  const platformColor = (p) => {
    return p === "curseforge" ? "orange" : "green";
  };

  const platformIcon = (p) => {
    return p === "curseforge" ? "CF" : "MR";
  };

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
                <span className="text-sm text-gray-500">CurseForge:</span>
                <Tag color={status.curseforge_configured ? "success" : "error"}>
                  {status.curseforge_configured ? "已配置" : "未配置"}
                </Tag>
              </div>
            </Card>
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
                <Select.Option value="all">全部</Select.Option>
                <Select.Option value="curseforge">CurseForge</Select.Option>
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
                    </div>
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
            {!status?.curseforge_configured && !status?.modrinth_configured && (
              <p className="text-sm mt-1">请先在配置文件中设置 CurseForge API Key 或 Modrinth API Token</p>
            )}
          </div>
        )}
      </div>
    </>
  );
};

export default Index;
