// components/CustomAlert.jsx
import { AlertTriangle } from 'lucide-react';
const CustomAlert = ({ title, children, variant = 'warning' }) => {
  const bgColor = {
    warning: 'bg-yellow-900/20 border-yellow-600/50',
    error: 'bg-red-900/20 border-red-600/50',
    success: 'bg-green-900/20 border-green-600/50'
  }[variant];

  return (
    <div className={`p-4 rounded-lg border ${bgColor}`}>
      <div className="flex gap-2 items-start">
        <AlertTriangle className="w-5 h-5 text-yellow-500 mt-0.5" />
        <div>
          <h3 className="font-medium text-yellow-500">{title}</h3>
          <p className="text-slate-300 text-sm mt-1">{children}</p>
        </div>
      </div>
    </div>
  );
};

export default CustomAlert;
