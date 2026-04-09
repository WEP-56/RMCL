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
import { User, UserPlus, Trash2 } from 'lucide-react';

interface Account {
  uuid: string;
  username: string;
  account_type: 'Offline' | 'Microsoft';
}

const Accounts = () => {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [loading, setLoading] = useState(true);
  const [newUsername, setNewUsername] = useState('');
  const [isDialogOpen, setIsDialogOpen] = useState(false);

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
      setIsDialogOpen(false);
      fetchAccounts();
    } catch (e) {
      console.error('Failed to add account:', e);
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h1 style={{ margin: 0, fontSize: '28px', fontWeight: 600 }}>账号管理</h1>
        
        <Dialog open={isDialogOpen} onOpenChange={(_e, data) => setIsDialogOpen(data.open)}>
          <DialogTrigger disableButtonEnhancement>
            <Button appearance="primary" icon={<UserPlus size={16} />}>添加账号</Button>
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
                  <Button appearance="transparent" icon={<Trash2 size={16} color="#ff6b6b" />} />
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
