// components/ChatInput.jsx
import { useState } from 'react';
import { Send } from 'lucide-react';

export const ChatInput = () => {
  const [message, setMessage] = useState('');

  const handleSubmit = (e) => {
    e.preventDefault();
    // Handle message submission
    console.log('Message sent:', message);
    setMessage('');
  };

  return (
    <div className="fixed bottom-0 left-0 right-0 p-4 bg-slate-800/95 backdrop-blur-sm border-t border-slate-700">
      <form onSubmit={handleSubmit} className="max-w-4xl mx-auto flex gap-2">
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          placeholder="Ask about your cluster..."
          className="flex-1 bg-slate-700 rounded-lg px-4 py-2 text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          type="submit"
          className="bg-blue-600 hover:bg-blue-700 text-white rounded-lg px-4 py-2 flex items-center gap-2"
        >
          <Send className="w-4 h-4" />
          Send
        </button>
      </form>
    </div>
  );
};
