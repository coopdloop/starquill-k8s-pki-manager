export const Notification = ({ type, message, onClose }) => (
  <div className={`flex items-center space-x-3 px-4 py-3 rounded-lg shadow-lg ${
    type === 'success' ? 'bg-green-500' :
    type === 'error' ? 'bg-red-500' :
    'bg-blue-500'
  }`}>
    <span className="text-white">{message}</span>
    <button onClick={onClose} className="text-white/80 hover:text-white">
      <X className="w-4 h-4" />
    </button>
  </div>
);
