// components/Sidebar.jsx
import { Shield, Server, ShieldCheck, Settings, Home } from 'lucide-react';
import { SidebarItem } from './ui/SidebarItem';
import { useLocation, useNavigate } from 'react-router-dom';

export const Sidebar = ({ clusterData }) => {
  const location = useLocation();
  const navigate = useNavigate();

  const getWorkerCount = () => clusterData?.workers?.length || 0;
  const getCertCount = () => clusterData?.control_plane?.certs?.length || 0;

  return (
    <aside className="hidden md:block md:w-64 bg-slate-800 border-r border-slate-700">
      <div className="p-4">
        <nav className="space-y-2">
          <SidebarItem
            icon={Home}
            text="Dashboard"
            active={location.pathname === '/'}
            onClick={() => navigate('/')}
          />
          <SidebarItem
            icon={Shield}
            text="Control Plane"
            active={location.pathname === '/control-plane'}
            onClick={() => navigate('/control-plane')}
            badge={clusterData?.control_plane?.status === 'healthy' ? '✓' : '!'}
          />
          <SidebarItem
            icon={Server}
            text="Worker Nodes"
            active={location.pathname === '/workers'}
            onClick={() => navigate('/workers')}
            badge={getWorkerCount().toString()}
          />
          <SidebarItem
            icon={ShieldCheck}
            text="Certificates"
            active={location.pathname === '/certificates'}
            onClick={() => navigate('/certificates')}
            badge={getCertCount().toString()}
          />
          <SidebarItem
            icon={Settings}
            text="Settings"
            active={location.pathname === '/settings'}
            onClick={() => navigate('/settings')}
          />
        </nav>
      </div>
    </aside>
  );
};
