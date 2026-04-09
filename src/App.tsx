import { Routes, Route } from 'react-router-dom';
import Layout from './pages/Layout';
import Home from './pages/Home';
import Accounts from './pages/Accounts';
import Instances from './pages/Instances';
import Market from './pages/Market';

const App = () => {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Home />} />
        <Route path="instances" element={<Instances />} />
        <Route path="market" element={<Market />} />
        <Route path="accounts" element={<Accounts />} />
        <Route path="settings" element={<div><h1>全局设置</h1><p>Java 路径配置等设置将在这里显示。</p></div>} />
      </Route>
    </Routes>
  );
};

export default App;
