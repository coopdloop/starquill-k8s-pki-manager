export const SidebarItem = ({ icon: Icon, text, active, onClick }) => (
  <button
        onClick={onClick}
    className={`w-full flex items-center space-x-3 px-3 py-2 rounded-lg transition-colors ${
      active ? 'bg-blue-500 text-white' : 'text-slate-400 hover:bg-slate-700 hover:text-white'
    }`}
  >
    <Icon className="w-5 h-5" />
    <span>{text}</span>
  </button>
);
