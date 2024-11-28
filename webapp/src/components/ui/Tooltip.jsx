export const Tooltip = ({ children, content }) => (
  <div className="relative group">
    {children}
    <div className="absolute z-50 invisible group-hover:visible bg-slate-800 text-white text-sm rounded-lg py-2 px-3 -top-2 left-1/2 -translate-x-1/2 -translate-y-full whitespace-nowrap">
      {content}
      <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 border-4 border-transparent border-t-slate-800" />
    </div>
  </div>
);
