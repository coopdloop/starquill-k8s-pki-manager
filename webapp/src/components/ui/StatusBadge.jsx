export const StatusBadge = ({ status }) => {
  const getStatusColor = () => {
    switch(status.toLowerCase()) {
      case 'distributed': return 'bg-green-500/10 text-green-400 border-green-500/20';
      case 'generated': return 'bg-yellow-500/10 text-yellow-400 border-yellow-500/20';
      default: return 'bg-red-500/10 text-red-400 border-red-500/20';
    }
  };

  return (
    <span className={`px-2 py-1 rounded-full text-xs border ${getStatusColor()}`}>
      {status}
    </span>
  );
};
