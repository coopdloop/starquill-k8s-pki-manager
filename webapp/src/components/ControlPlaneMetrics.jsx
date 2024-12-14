import React, { useState, useEffect } from 'react';
import {
    Shield, Activity, CpuIcon, Database, Clock, AlertCircle,
    CheckCircle, XCircle, RefreshCw, Settings, TrendingUp
} from 'lucide-react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

const MetricsChart = ({ data }) => {
    return (
        <div className="h-64 w-full">
            <ResponsiveContainer width="100%" height="100%">
                <LineChart data={data}>
                    <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                    <XAxis
                        dataKey="time"
                        stroke="#9CA3AF"
                        fontSize={12}
                    />
                    <YAxis
                        stroke="#9CA3AF"
                        fontSize={12}
                    />
                    <Tooltip
                        contentStyle={{
                            backgroundColor: '#1F2937',
                            border: 'none',
                            borderRadius: '0.375rem',
                            color: '#F3F4F6'
                        }}
                    />
                    <Line
                        type="monotone"
                        dataKey="requests"
                        stroke="#60A5FA"
                        strokeWidth={2}
                    />
                    <Line
                        type="monotone"
                        dataKey="latency"
                        stroke="#34D399"
                        strokeWidth={2}
                    />
                </LineChart>
            </ResponsiveContainer>
        </div>
    );
};

export default () => {
    const [metrics] = useState({
        current: {
            requestRate: '1.2k/s',
            errorRate: '0.01%',
            p99Latency: '120ms',
            cpuUsage: '45%',
            memoryUsage: '60%',
            goroutines: '2.5k',
            openConnections: '150'
        },
        history: [
            { time: '5m', requests: 1100, latency: 110 },
            { time: '4m', requests: 1250, latency: 115 },
            { time: '3m', requests: 1400, latency: 125 },
            { time: '2m', requests: 1300, latency: 118 },
            { time: '1m', requests: 1200, latency: 120 },
            { time: 'now', requests: 1150, latency: 115 }
        ]
    });

    return (
        <div className="bg-slate-800 p-6 rounded-lg">
            <h3 className="text-lg font-medium text-white mb-4 flex items-center gap-2">
                <Activity className="w-5 h-5 text-blue-400" />
                API Server Metrics
            </h3>

            {/* Real-time metrics grid */}
            <div className="grid grid-cols-4 gap-4 mb-6">
                <div className="bg-slate-700/50 p-4 rounded-lg">
                    <div className="text-sm text-slate-400">Request Rate</div>
                    <div className="text-xl font-medium text-white mt-1">
                        {metrics.current.requestRate}
                    </div>
                </div>
                <div className="bg-slate-700/50 p-4 rounded-lg">
                    <div className="text-sm text-slate-400">Error Rate</div>
                    <div className="text-xl font-medium text-white mt-1">
                        {metrics.current.errorRate}
                    </div>
                </div>
                <div className="bg-slate-700/50 p-4 rounded-lg">
                    <div className="text-sm text-slate-400">P99 Latency</div>
                    <div className="text-xl font-medium text-white mt-1">
                        {metrics.current.p99Latency}
                    </div>
                </div>
                <div className="bg-slate-700/50 p-4 rounded-lg">
                    <div className="text-sm text-slate-400">Open Connections</div>
                    <div className="text-xl font-medium text-white mt-1">
                        {metrics.current.openConnections}
                    </div>
                </div>
            </div>

            {/* Resource usage meters */}
            <div className="grid grid-cols-2 gap-4 mb-6">
                <div className="bg-slate-700/50 p-4 rounded-lg">
                    <div className="flex justify-between mb-2">
                        <span className="text-sm text-slate-400">CPU Usage</span>
                        <span className="text-sm text-white">{metrics.current.cpuUsage}</span>
                    </div>
                    <div className="h-2 bg-slate-600 rounded-full overflow-hidden">
                        <div
                            className="h-full bg-blue-400 transition-all duration-500"
                            style={{ width: metrics.current.cpuUsage }}
                        />
                    </div>
                </div>
                <div className="bg-slate-700/50 p-4 rounded-lg">
                    <div className="flex justify-between mb-2">
                        <span className="text-sm text-slate-400">Memory Usage</span>
                        <span className="text-sm text-white">{metrics.current.memoryUsage}</span>
                    </div>
                    <div className="h-2 bg-slate-600 rounded-full overflow-hidden">
                        <div
                            className="h-full bg-green-400 transition-all duration-500"
                            style={{ width: metrics.current.memoryUsage }}
                        />
                    </div>
                </div>
            </div>

            {/* Performance graphs */}
            <div className="bg-slate-700/50 p-4 rounded-lg">
                <div className="flex justify-between items-center mb-4">
                    <h4 className="text-sm font-medium text-white">Performance Trends</h4>
                    <div className="flex items-center gap-4 text-xs">
                        <div className="flex items-center gap-2">
                            <div className="w-3 h-3 bg-blue-400 rounded-full"></div>
                            <span className="text-slate-400">Requests/s</span>
                        </div>
                        <div className="flex items-center gap-2">
                            <div className="w-3 h-3 bg-green-400 rounded-full"></div>
                            <span className="text-slate-400">Latency (ms)</span>
                        </div>
                    </div>
                </div>
                <MetricsChart data={metrics.history} />
            </div>

            {/* Additional system metrics */}
            <div className="grid grid-cols-2 gap-4 mt-6">
                <div className="bg-slate-700/50 p-4 rounded-lg">
                    <div className="flex items-center gap-2 mb-2">
                        <TrendingUp className="w-4 h-4 text-blue-400" />
                        <span className="text-sm text-slate-400">Active Goroutines</span>
                    </div>
                    <div className="text-xl font-medium text-white">
                        {metrics.current.goroutines}
                    </div>
                </div>
                <div className="bg-slate-700/50 p-4 rounded-lg">
                    <div className="flex items-center gap-2 mb-2">
                        <Database className="w-4 h-4 text-green-400" />
                        <span className="text-sm text-slate-400">etcd Operations</span>
                    </div>
                    <div className="text-xl font-medium text-white">
                        2.3k/s
                    </div>
                </div>
            </div>
        </div>
    );
}
