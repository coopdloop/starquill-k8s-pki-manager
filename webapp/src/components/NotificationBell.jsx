import { useState } from "react";
import { Bell } from 'lucide-react';

export const NotificationBell = () => {
    const [unread, setUnread] = useState(0);

    return (
        <div className="relative">
            <button className="p-2 hover:bg-slate-700 rounded-lg">
                <Bell className="w-5 h-5 text-slate-300" />
                {unread > 0 && (
                    <span className="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full w-4 h-4 flex items-center justify-center">
                        {unread}
                    </span>
                )}
            </button>
        </div>
    );
};
