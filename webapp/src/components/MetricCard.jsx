// components/MetricCard.jsx
export const MetricCard = ({ icon: Icon, title, value, status, secondaryValue, onClick }) => {
  return (
    <div
      className="bg-slate-800 rounded-lg p-6 transition-all hover:bg-slate-700/50 cursor-pointer"
      onClick={onClick}
    >
      <div className="flex items-center gap-4 mb-4">
        <Icon className="w-8 h-8 text-blue-400" />
        <div>
          <h3 className="text-lg font-medium text-white">{title}</h3>
          <p className="text-sm text-slate-400">{value}</p>
        </div>
      </div>
      <div className="space-y-2">
        <div className="flex justify-between text-sm">
          <span className="text-slate-400">Status</span>
          <span className={status === 'Healthy' ? 'text-green-400' : 'text-yellow-400'}>
            {status}
          </span>
        </div>
        {secondaryValue && (
          <div className="flex justify-between text-sm">
            <span className="text-slate-400">{secondaryValue.label}</span>
            <span className="text-white">{secondaryValue.value}</span>
          </div>
        )}
      </div>
    </div>
  );
};
