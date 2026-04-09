import { Routes, Route } from 'react-router-dom';
import Layout from './pages/Layout';
import Home from './pages/Home';
import Accounts from './pages/Accounts';
import Instances from './pages/Instances';
import Market from './pages/Market';
import Settings from './pages/Settings';

const App = () => {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Home />} />
        <Route path="instances" element={<Instances />} />
        <Route path="market" element={<Market />} />
        <Route path="accounts" element={<Accounts />} />
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  );
};

export default App;
