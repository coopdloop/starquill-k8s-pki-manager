import { Shield, Server } from 'lucide-react';
import { Button } from './ui/Button'

export const NodeCard = ({ node, onAction }) => {
    return (
        <div className="bg-slate-700/50 rounded-lg p-4 hover:bg-slate-700/70 transition-colors">
            <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                    {node.type === 'control' ? (
                        <Shield className="w-5 h-5 text-blue-400" />
                    ) : (
                        <Server className="w-5 h-5 text-green-400" />
                    )}
                    <div>
                        <h3 className="text-sm font-medium text-white truncate">{node.name}</h3>
                        <p className="text-xs text-slate-400 truncate">{node.ip || 'Loading...'}</p>
                    </div>
                </div>
                <Button
                    variant="ghost"
                    size="sm"
                    onClick={onAction}
                >
                    Details
                </Button>
            </div>
        </div>
    );
};
