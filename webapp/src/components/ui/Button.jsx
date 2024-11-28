export const Button = ({ children, variant = 'primary', ...props }) => (
  <button
    className={`px-4 py-2 rounded-lg font-medium transition-colors ${
      variant === 'primary'
        ? 'bg-blue-500 text-white hover:bg-blue-600'
        : 'bg-slate-700 text-slate-200 hover:bg-slate-600'
    }`}
    {...props}
  >
    {children}
  </button>
);
