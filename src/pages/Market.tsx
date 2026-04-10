import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Input,
  Button,
  Card,
  CardHeader,
  Text,
  Spinner,
  Badge,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Dropdown,
  Option,
  TabList,
  Tab,
} from '@fluentui/react-components';
import { Search, Download, Box, PackageOpen } from 'lucide-react';

interface Project {
  project_id: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  downloads: number;
  icon_url: string | null;
  categories: string[];
}

interface SearchResult {
  hits: Project[];
  total_hits: number;
}

interface ModVersion {
  id: string;
  name: string;
  version_number: string;
  game_versions: string[];
  loaders: string[];
  date_published: string;
}

interface Instance {
  id: string;
  name: string;
  mc_version: string;
  loader: string;
}

const Market = () => {
  const [query, setQuery] = useState('sodium');
  const [projectType, setProjectType] = useState<'mod' | 'modpack'>('mod');
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<Project[]>([]);
  
  // Install Dialog State
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);
  const [versions, setVersions] = useState<ModVersion[]>([]);
  const [versionsLoading, setVersionsLoading] = useState(false);
  const [instances, setInstances] = useState<Instance[]>([]);
  const [selectedInstanceId, setSelectedInstanceId] = useState<string>('');
  const [selectedVersionId, setSelectedVersionId] = useState<string>('');
  const [newModpackName, setNewModpackName] = useState<string>('');

  const fetchInstances = async () => {
    try {
      const res = await invoke<Instance[]>('get_instances');
      setInstances(res);
      if (res.length > 0) setSelectedInstanceId(res[0].id);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    fetchInstances();
  }, []);

  useEffect(() => {
    handleSearch();
  }, [projectType]);

  const handleSearch = async () => {
    if (!query.trim()) return;
    try {
      setLoading(true);
      const res = await invoke<SearchResult>('search_modrinth', { 
        query,
        projectType: projectType,
        limit: 12, 
        offset: 0 
      });
      setResults(res.hits);
    } catch (e) {
      console.error('Search failed', e);
    } finally {
      setLoading(false);
    }
  };

  const handleOpenInstall = async (project: Project) => {
    setSelectedProject(project);
    setVersions([]);
    setSelectedVersionId('');
    if (projectType === 'modpack') {
      setNewModpackName(project.title);
    }
    try {
      setVersionsLoading(true);
      const res = await invoke<ModVersion[]>('get_modrinth_versions', { 
        projectId: project.project_id 
      });
      setVersions(res);
      if (res.length > 0) setSelectedVersionId(res[0].id);
    } catch (e) {
      console.error('Failed to fetch versions', e);
    } finally {
      setVersionsLoading(false);
    }
  };

  const handleInstall = async () => {
    if (!selectedVersionId) return;
    if (projectType === 'mod' && !selectedInstanceId) return;
    if (projectType === 'modpack' && !newModpackName) return;

    const versionData = versions.find(v => v.id === selectedVersionId);
    if (!versionData) return;

    try {
      if (projectType === 'modpack') {
        alert("整合包文件通常较大且包含大量前置模组，下载与解压可能需要数分钟，请耐心等待...");
        setSelectedProject(null);
        await invoke('install_modpack', { 
          name: newModpackName, 
          version: versionData 
        });
        alert('整合包安装成功！你可以在实例列表中启动它了。');
      } else {
        await invoke('install_mod', { 
          instanceId: selectedInstanceId, 
          version: versionData 
        });
        alert('模组安装成功！');
        setSelectedProject(null);
      }
    } catch (e) {
      console.error(e);
      alert('安装失败: ' + String(e));
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', height: '100%' }}>
      <div>
        <h1 style={{ margin: 0, fontSize: '28px', fontWeight: 600, color: 'rgba(255, 255, 255, 0.9)' }}>市场</h1>
        <p style={{ margin: '4px 0 16px 0', color: 'rgba(255, 255, 255, 0.5)', fontSize: '14px' }}>
          浏览并安装来自 Modrinth 的模组、资源包和整合包。
        </p>
        <TabList 
          selectedValue={projectType} 
          onTabSelect={(_, data) => setProjectType(data.value as 'mod' | 'modpack')}
          style={{ marginBottom: '16px' }}
        >
          <Tab value="mod" icon={<Box size={16} />}>模组 (Mods)</Tab>
          <Tab value="modpack" icon={<PackageOpen size={16} />}>整合包 (Modpacks)</Tab>
        </TabList>
        <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
          <Input 
            contentBefore={<Search size={16} />}
            placeholder={`搜索 ${projectType === 'mod' ? '模组' : '整合包'}...`} 
            value={query}
            onChange={(_e, data) => setQuery(data.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            style={{ flex: 1, maxWidth: '500px' }}
          />
          <Button appearance="primary" onClick={handleSearch} disabled={loading} style={{ backgroundColor: '#60CDFF', color: '#000' }}>
            搜索
          </Button>
        </div>
      </div>

      {loading ? (
        <div style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', flex: 1 }}>
          <Spinner size="huge" />
        </div>
      ) : results.length === 0 ? (
        <div style={{ 
          display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center',
          padding: '80px 0', color: 'rgba(255,255,255,0.4)', backgroundColor: 'rgba(0,0,0,0.2)',
          borderRadius: '12px', border: '1px dashed rgba(255,255,255,0.05)'
        }}>
          <Search size={48} style={{ marginBottom: '16px', opacity: 0.3 }} />
          <Text size={400}>未找到相关{projectType === 'mod' ? '模组' : '整合包'}</Text>
        </div>
      ) : (
        <div style={{ 
          display: 'grid', 
          gridTemplateColumns: 'repeat(auto-fill, minmax(340px, 1fr))', 
          gap: '16px',
          overflowY: 'auto',
          paddingBottom: '20px',
          paddingRight: '8px'
        }}>
          {results.map((project) => (
            <Card key={project.project_id} style={{ 
              backgroundColor: 'rgba(255,255,255,0.03)', 
              border: '1px solid rgba(255,255,255,0.08)',
              borderRadius: '12px',
              transition: 'transform 0.2s, background-color 0.2s',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.06)';
              e.currentTarget.style.transform = 'translateY(-2px)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'rgba(255,255,255,0.03)';
              e.currentTarget.style.transform = 'none';
            }}
            >
              <CardHeader
                image={
                  project.icon_url ? 
                  <img src={project.icon_url} alt="icon" style={{ width: 48, height: 48, borderRadius: 8 }} /> : 
                  <div style={{ width: 48, height: 48, backgroundColor: '#333', borderRadius: 8, display: 'flex', alignItems: 'center', justifyContent: 'center' }}><Box size={24} /></div>
                }
                header={<Text weight="semibold" size={400} style={{ color: 'white' }}>{project.title}</Text>}
                description={
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    <Text size={200} style={{ color: 'rgba(255,255,255,0.5)' }}>by {project.author}</Text>
                    <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                      <Badge size="small" color="brand" appearance="tint" style={{ backgroundColor: 'rgba(96, 205, 255, 0.1)', color: '#60CDFF' }}>
                        {(project.downloads / 1000000).toFixed(1)}M 下载
                      </Badge>
                      {project.categories.slice(0, 2).map(c => 
                        <Badge key={c} size="small" color="informative" appearance="tint" style={{ backgroundColor: 'rgba(255, 255, 255, 0.05)', color: '#ccc' }}>{c}</Badge>
                      )}
                    </div>
                  </div>
                }
              />
              <Text size={200} style={{ color: '#aaa', marginTop: '8px', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden', minHeight: '36px' }}>
                {project.description}
              </Text>
              
              <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '12px' }}>
                <Dialog open={selectedProject?.project_id === project.project_id} onOpenChange={(_e, data) => !data.open && setSelectedProject(null)}>
                  <DialogTrigger disableButtonEnhancement>
                    <Button appearance="secondary" icon={<Download size={16} />} onClick={() => handleOpenInstall(project)}>
                      获取
                    </Button>
                  </DialogTrigger>
                  <DialogSurface style={{ backgroundColor: '#2B2B2B', border: '1px solid rgba(255,255,255,0.1)' }}>
                    <DialogBody>
                      <DialogTitle style={{ color: 'white' }}>{projectType === 'mod' ? '安装模组' : '安装整合包'}: {project.title}</DialogTitle>
                      <DialogContent>
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', paddingTop: '12px' }}>
                          {projectType === 'mod' ? (
                            <div>
                              <Text weight="semibold" style={{ color: '#ccc' }}>1. 选择目标实例</Text>
                              <Dropdown 
                                value={instances.find(i => i.id === selectedInstanceId)?.name || '选择实例...'}
                                onOptionSelect={(_e, data) => setSelectedInstanceId(data.optionValue as string)}
                                style={{ width: '100%', marginTop: '8px' }}
                              >
                                {instances.map(i => <Option key={i.id} value={i.id} text={`${i.name} (${i.mc_version})`}>{i.name} ({i.mc_version})</Option>)}
                              </Dropdown>
                            </div>
                          ) : (
                            <div>
                              <Text weight="semibold" style={{ color: '#ccc' }}>1. 新实例名称</Text>
                              <Input 
                                value={newModpackName}
                                onChange={(_e, data) => setNewModpackName(data.value)}
                                placeholder="输入新建实例的名称"
                                style={{ width: '100%', marginTop: '8px' }}
                              />
                            </div>
                          )}

                          <div>
                            <Text weight="semibold" style={{ color: '#ccc' }}>{projectType === 'mod' ? '2. 选择模组版本' : '选择整合包版本'}</Text>
                            {versionsLoading ? <Spinner size="tiny" style={{ marginLeft: 8 }} /> : (
                              <Dropdown 
                                value={versions.find(v => v.id === selectedVersionId)?.name || '选择版本...'}
                                onOptionSelect={(_e, data) => setSelectedVersionId(data.optionValue as string)}
                                style={{ width: '100%', marginTop: '8px' }}
                              >
                                {versions.map(v => <Option key={v.id} value={v.id} text={`${v.name} [${v.loaders.join(', ')}]`}>{v.name} [{v.loaders.join(', ')}]</Option>)}
                              </Dropdown>
                            )}
                          </div>
                        </div>
                      </DialogContent>
                      <DialogActions>
                        <DialogTrigger disableButtonEnhancement>
                          <Button appearance="secondary">取消</Button>
                        </DialogTrigger>
                        <Button appearance="primary" onClick={handleInstall} disabled={(projectType === 'mod' && !selectedInstanceId) || !selectedVersionId} style={{ backgroundColor: '#60CDFF', color: '#000' }}>
                          确认安装
                        </Button>
                      </DialogActions>
                    </DialogBody>
                  </DialogSurface>
                </Dialog>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
};

export default Market;
