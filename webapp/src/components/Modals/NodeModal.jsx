import { X } from 'lucide-react';
import { CertificateList } from '../CertificateList';
import { Button } from '../ui/Button';

export const NodeModal = ({ node, onClose }) => {
    console.log(node);
    return (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
            <div className="bg-slate-800 rounded-xl p-6 w-full max-w-2xl">
                <div className="flex justify-between items-center mb-4">
                    <h2 className="text-xl font-bold text-white">{node.name}</h2>
                    <button onClick={onClose} className="text-slate-400 hover:text-white">
                        <X className="w-6 h-6" />
                    </button>
                </div>
                <div className="space-y-4">
                    <CertificateList certificates={node.certs} />
                    <div className="flex justify-end space-x-3 mt-6">
                        <Button variant="secondary" onClick={onClose}>Cancel</Button>
                        <Button onClick={() => console.log('Generate Cert')}>Generate Certificate</Button>
                    </div>
                </div>
            </div>
        </div>
    )
}
