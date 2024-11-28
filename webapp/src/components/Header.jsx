import { Bell, Settings } from 'lucide-react';
import { NotificationBell } from './NotificationBell';

export const Header = () => (
  <header className="bg-slate-800 border-b border-slate-700 p-4">
    <div className="flex justify-between items-center">
      <h1 className="text-xl font-bold text-white">Starquill</h1>
      <div className="flex items-center space-x-4">
        <NotificationBell />
        <button className="p-2 hover:bg-slate-700 rounded-lg">
          <Settings className="w-5 h-5 text-slate-300" />
        </button>
      </div>
    </div>
  </header>
);
