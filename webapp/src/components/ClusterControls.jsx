// components/ClusterControls.jsx
export const ClusterControls = ({ onRearrange }) => {
  return (
    <div className="absolute top-4 right-4 z-10 space-x-2">
      <button
        onClick={() => onRearrange('circle')}
        className="bg-slate-700 hover:bg-slate-600 text-white rounded-lg px-3 py-2 text-sm"
      >
        Circle Layout
      </button>
      <button
        onClick={() => onRearrange('grid')}
        className="bg-slate-700 hover:bg-slate-600 text-white rounded-lg px-3 py-2 text-sm"
      >
        Grid Layout
      </button>
      <button
        onClick={() => onRearrange('random')}
        className="bg-slate-700 hover:bg-slate-600 text-white rounded-lg px-3 py-2 text-sm"
      >
        Random Layout
      </button>
    </div>
  );
};
