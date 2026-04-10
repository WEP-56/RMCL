import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Button,
  Input,
  Card,
  CardHeader,
  Text,
  Spinner,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent
} from '@fluentui/react-components';
import { User, UserPlus, Trash2 } from 'lucide-react'; // We use Github icon as placeholder for MS if no Windows icon, but let's just use Box or UserPlus

interface Account {
  uuid: string;
  username: string;
  account_type: 'Offline' | 'Microsoft';
}

const Accounts = () => {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);
  const [newUsername, setNewUsername] = useState('');
  const [isOfflineDialogOpen, setIsOfflineDialogOpen] = useState(false);
  const [isMsaDialogOpen, setIsMsaDialogOpen] = useState(false);
  const [msaCode, setMsaCode] = useState('');
  const [msaUrl, setMsaUrl] = useState('');
  const [msaPolling, setMsaPolling] = useState(false);

  const fetchAccounts = async () => {
    try {
      setLoading(true);
      const res = await invoke<Account[]>('get_accounts');
      setAccounts(res);
    } catch (e) {
      console.error('Failed to fetch accounts:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchAccounts();
  }, []);

  const handleAddOfflineAccount = async () => {
    if (!newUsername.trim()) return;
    
    try {
      await invoke('add_offline_account', { username: newUsername });
      setNewUsername('');
      setIsOfflineDialogOpen(false);
      fetchAccounts();
    } catch (e) {
      console.error('Failed to add account:', e);
    }
  };

  const handleDeleteAccount = async (uuid: string) => {
    if (confirm("确定要删除这个账号吗？")) {
      try {
        await invoke('delete_account', { uuid });
        fetchAccounts();
      } catch (e) {
        console.error('Failed to delete account:', e);
      }
    }
  };

  const handleStartMsaLogin = async () => {
    try {
      setIsMsaDialogOpen(true);
      setMsaPolling(true);
      setMsaCode('获取中...');
      
      const res = await invoke<{ user_code: string, device_code: string, verification_uri: string, interval: number }>('start_msa_login');
      setMsaCode(res.user_code);
      setMsaUrl(res.verification_uri);
      
      // poll
      try {
        await invoke('poll_msa_token', { deviceCode: res.device_code, interval: res.interval });
        setIsMsaDialogOpen(false);
        fetchAccounts();
      } catch (err) {
        console.error("MSA Poll error", err);
        setMsaCode('登录失败或超时');
      }
      
    } catch (e) {
      console.error(e);
      setMsaCode('无法连接到微软服务器');
    } finally {
      setMsaPolling(false);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h1 style={{ margin: 0, fontSize: '28px', fontWeight: 600 }}>账号管理</h1>
        
        <div style={{ display: 'flex', gap: '8px' }}>
          <Button appearance="primary" style={{ backgroundColor: '#107c10', color: 'white' }} onClick={handleStartMsaLogin}>微软登录</Button>

          <Dialog open={isOfflineDialogOpen} onOpenChange={(_e, data) => setIsOfflineDialogOpen(data.open)}>
            <DialogTrigger disableButtonEnhancement>
              <Button appearance="secondary" icon={<UserPlus size={16} />}>离线账号</Button>
            </DialogTrigger>
            <DialogSurface>
              <DialogBody>
                <DialogTitle>添加离线账号</DialogTitle>
                <DialogContent>
                  <div style={{ padding: '16px 0' }}>
                    <Input 
                      placeholder="输入游戏内玩家名称" 
                      value={newUsername}
                      onChange={(_e, data) => setNewUsername(data.value)}
                      style={{ width: '100%' }}
                    />
                    <Text size={200} style={{ color: 'gray', marginTop: '8px', display: 'block' }}>
                      离线账号仅用于单机模式或支持离线验证的第三方服务器。
                    </Text>
                  </div>
                </DialogContent>
                <DialogActions>
                  <DialogTrigger disableButtonEnhancement>
                    <Button appearance="secondary">取消</Button>
                  </DialogTrigger>
                  <Button appearance="primary" onClick={handleAddOfflineAccount}>确认添加</Button>
                </DialogActions>
              </DialogBody>
            </DialogSurface>
          </Dialog>

          <Dialog open={isMsaDialogOpen} onOpenChange={(_e, data) => { if (!msaPolling) setIsMsaDialogOpen(data.open); }}>
            <DialogSurface>
              <DialogBody>
                <DialogTitle>微软账号登录</DialogTitle>
                <DialogContent>
                  <div style={{ padding: '16px 0', textAlign: 'center' }}>
                    <Text size={400} style={{ display: 'block', marginBottom: '16px' }}>
                      请在浏览器中打开以下链接，并输入代码进行授权：
                    </Text>
                    {msaUrl && (
                      <a href={msaUrl} target="_blank" rel="noreferrer" style={{ color: '#60CDFF', textDecoration: 'underline', marginBottom: '16px', display: 'block' }}>
                        {msaUrl}
                      </a>
                    )}
                    <div style={{ 
                      fontSize: '32px', 
                      letterSpacing: '4px', 
                      fontWeight: 'bold', 
                      padding: '16px', 
                      backgroundColor: 'rgba(0,0,0,0.2)', 
                      borderRadius: '8px',
                      userSelect: 'all'
                    }}>
                      {msaCode}
                    </div>
                    {msaPolling && (
                      <div style={{ marginTop: '24px', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '8px' }}>
                        <Spinner size="small" />
                        <Text>正在等待授权...</Text>
                      </div>
                    )}
                  </div>
                </DialogContent>
                <DialogActions>
                  <Button appearance="secondary" onClick={() => setIsMsaDialogOpen(false)} disabled={msaPolling}>关闭</Button>
                </DialogActions>
              </DialogBody>
            </DialogSurface>
          </Dialog>
        </div>
      </div>

      {loading ? (
        <Spinner size="large" label="加载账号中..." />
      ) : accounts.length === 0 ? (
        <div style={{ textAlign: 'center', padding: '60px 0', color: 'rgba(255,255,255,0.5)' }}>
          <User size={48} style={{ marginBottom: '16px', opacity: 0.5 }} />
          <Text size={400} style={{ display: 'block' }}>暂无账号</Text>
          <Text size={300}>点击右上角按钮添加一个账号开始游戏</Text>
        </div>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '16px' }}>
          {accounts.map((acc) => (
            <Card key={acc.uuid} style={{ backgroundColor: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.1)' }}>
              <CardHeader
                image={<img src={`https://minotar.net/helm/${acc.username}/64.png`} alt="avatar" style={{ borderRadius: '4px' }} />}
                header={<Text weight="semibold" size={400}>{acc.username}</Text>}
                description={<Text size={200} style={{ color: 'gray' }}>{acc.account_type === 'Offline' ? '离线模式' : '微软账号'}</Text>}
                action={
                  <Button appearance="transparent" icon={<Trash2 size={16} color="#ff6b6b" />} onClick={() => handleDeleteAccount(acc.uuid)} />
                }
              />
            </Card>
          ))}
        </div>
      )}
    </div>
  );
};

export default Accounts;
