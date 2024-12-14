// components/CustomTooltip.jsx
import React, { useState } from 'react';
const CustomTooltip = ({ children, content }) => {
  const [show, setShow] = useState(false);

  return (
    <div className="relative inline-block">
      <div
        onMouseEnter={() => setShow(true)}
        onMouseLeave={() => setShow(false)}
      >
        {children}
      </div>
      {show && (
        <div className="absolute z-50 w-64 p-2 text-sm bg-slate-900 text-white rounded-md shadow-lg -translate-x-1/2 left-0 mt-1">
          {content}
        </div>
      )}
    </div>
  );
};

export default CustomTooltip;
