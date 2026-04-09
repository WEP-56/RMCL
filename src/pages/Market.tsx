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
} from '@fluentui/react-components';
import { Search, Download, Box } from 'lucide-react';

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
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<Project[]>([]);
  
  // Install Dialog State
  const [selectedProject, setSelectedProject] = useState<Project | null>(null);
  const [versions, setVersions] = useState<ModVersion[]>([]);
  const [versionsLoading, setVersionsLoading] = useState(false);
  const [instances, setInstances] = useState<Instance[]>([]);
  const [selectedInstanceId, setSelectedInstanceId] = useState<string>('');
  const [selectedVersionId, setSelectedVersionId] = useState<string>('');

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
    handleSearch();
  }, []);

  const handleSearch = async () => {
    if (!query.trim()) return;
    try {
      setLoading(true);
      const res = await invoke<SearchResult>('search_modrinth', { 
        query, 
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
    if (!selectedInstanceId || !selectedVersionId) return;
    const versionData = versions.find(v => v.id === selectedVersionId);
    if (!versionData) return;

    try {
      await invoke('install_mod', { 
        instanceId: selectedInstanceId, 
        version: versionData 
      });
      setSelectedProject(null);
      alert('模组安装成功！');
    } catch (e) {
      console.error(e);
      alert('安装失败: ' + e);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px', height: '100%' }}>
      <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
        <Input 
          contentBefore={<Search size={16} />}
          placeholder="搜索 Modrinth 模组、资源包..." 
          value={query}
          onChange={(_e, data) => setQuery(data.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
          style={{ flex: 1, maxWidth: '400px' }}
        />
        <Button appearance="primary" onClick={handleSearch} disabled={loading}>
          搜索
        </Button>
      </div>

      {loading ? (
        <Spinner size="large" style={{ marginTop: '40px' }} />
      ) : results.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '60px 0', color: 'rgba(255,255,255,0.5)' }}>
          <Text size={400}>未找到相关模组</Text>
        </div>
      ) : (
        <div style={{ 
          display: 'grid', 
          gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))', 
          gap: '16px',
          overflowY: 'auto',
          paddingBottom: '20px'
        }}>
          {results.map((project) => (
            <Card key={project.project_id} style={{ backgroundColor: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.1)' }}>
              <CardHeader
                image={
                  project.icon_url ? 
                  <img src={project.icon_url} alt="icon" style={{ width: 48, height: 48, borderRadius: 8 }} /> : 
                  <div style={{ width: 48, height: 48, backgroundColor: '#333', borderRadius: 8, display: 'flex', alignItems: 'center', justifyContent: 'center' }}><Box size={24} /></div>
                }
                header={<Text weight="semibold" size={400}>{project.title}</Text>}
                description={
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    <Text size={200} style={{ color: 'gray' }}>by {project.author}</Text>
                    <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                      <Badge size="small" color="brand" appearance="tint">{(project.downloads / 1000000).toFixed(1)}M 下载</Badge>
                      {project.categories.slice(0, 2).map(c => <Badge key={c} size="small" color="informative" appearance="tint">{c}</Badge>)}
                    </div>
                  </div>
                }
              />
              <Text size={200} style={{ color: '#ccc', marginTop: '8px', display: '-webkit-box', WebkitLineClamp: 2, WebkitBoxOrient: 'vertical', overflow: 'hidden' }}>
                {project.description}
              </Text>
              
              <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '12px' }}>
                <Dialog open={selectedProject?.project_id === project.project_id} onOpenChange={(_e, data) => !data.open && setSelectedProject(null)}>
                  <DialogTrigger disableButtonEnhancement>
                    <Button appearance="secondary" icon={<Download size={16} />} onClick={() => handleOpenInstall(project)}>
                      安装
                    </Button>
                  </DialogTrigger>
                  <DialogSurface>
                    <DialogBody>
                      <DialogTitle>安装 {project.title}</DialogTitle>
                      <DialogContent>
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', paddingTop: '12px' }}>
                          <div>
                            <Text weight="semibold">1. 选择目标实例</Text>
                            <Dropdown 
                              value={instances.find(i => i.id === selectedInstanceId)?.name || '选择实例...'}
                              onOptionSelect={(_e, data) => setSelectedInstanceId(data.optionValue as string)}
                              style={{ width: '100%', marginTop: '8px' }}
                            >
                              {instances.map(i => <Option key={i.id} value={i.id} text={`${i.name} (${i.mc_version})`}>{i.name} ({i.mc_version})</Option>)}
                            </Dropdown>
                          </div>

                          <div>
                            <Text weight="semibold">2. 选择模组版本</Text>
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
                        <Button appearance="primary" onClick={handleInstall} disabled={!selectedInstanceId || !selectedVersionId}>
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
