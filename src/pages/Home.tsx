import { Text, Card, CardHeader } from '@fluentui/react-components';
import { Play } from 'lucide-react';

const Home = () => {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '32px' }}>
      <div>
        <h1 style={{ margin: 0, fontSize: '42px', fontWeight: 700, backgroundImage: 'linear-gradient(90deg, #60A5FA, #34D399)', WebkitBackgroundClip: 'text', color: 'transparent' }}>
          RustMC Launcher
        </h1>
        <Text size={500} style={{ color: 'rgba(255,255,255,0.7)', marginTop: '8px', display: 'block' }}>
          极速、轻量、原生的 Minecraft 启动器。
        </Text>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '20px' }}>
        <Card style={{ backgroundColor: 'rgba(255,255,255,0.03)', border: '1px solid rgba(255,255,255,0.1)' }}>
          <CardHeader
            header={<Text weight="semibold" size={500}>快速开始</Text>}
            description={<Text size={300} style={{ color: 'gray' }}>点击侧边栏的「实例」创建你的第一个游戏环境，然后尽情探索像素世界吧！</Text>}
            action={<Play size={32} color="#60A5FA" />}
          />
        </Card>
      </div>
    </div>
  );
};

export default Home;
