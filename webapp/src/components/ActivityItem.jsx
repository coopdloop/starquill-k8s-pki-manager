export const ActivityItem = ({ type, message, timestamp }) => (
  <div className="flex items-center space-x-3 text-sm">
    <div className={`w-2 h-2 rounded-full ${
      type === 'success' ? 'bg-green-400' :
      type === 'warning' ? 'bg-yellow-400' :
      'bg-blue-400'
    }`} />
    <span className="text-slate-200">{message}</span>
    <span className="text-slate-500">{timestamp}</span>
  </div>
);
