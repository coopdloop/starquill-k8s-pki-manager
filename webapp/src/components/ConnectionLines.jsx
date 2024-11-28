import React from "react";

export const ConnectionLines = React.memo(({ nodes }) => {
    if (!nodes?.controlPlane || !nodes?.workers) return null;

    return (
        <svg className="absolute inset-0"
            style={{ zIndex: 0 }}
            width="100%"
            height="100%"
            preserveAspectRatio="none"
        >
            {nodes.workers.map(worker => (
                <g key={worker.id}>
                    <defs>
                        <linearGradient
                            id={`line-gradient-${worker.id}`}
                            gradientUnits="userSpaceOnUse"
                            x1={nodes.controlPlane.x}
                            y1={nodes.controlPlane.y}
                            x2={worker.x}
                            y2={worker.y}
                        >
                            <stop offset="0%" stopColor="#4f46e5" />
                            <stop offset="100%" stopColor="#06b6d4" />
                        </linearGradient>
                    </defs>
                    <line
                        x1={nodes.controlPlane.x}
                        y1={nodes.controlPlane.y}
                        x2={worker.x}
                        y2={worker.y}
                        stroke={`url(#line-gradient-${worker.id})`}
                        strokeWidth="2"
                        className="animate-pulse"
                    />
                </g>
            ))}
        </svg>
    );
});
