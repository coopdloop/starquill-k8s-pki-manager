// Layout.jsx
import React from 'react';
import { Outlet } from 'react-router-dom'; // Add this import
import { Sidebar } from './components/Sidebar';
import { Header } from './components/Header';

export const Layout = () => (  // Remove the children prop
  <div className="flex h-screen bg-slate-900">
    <Sidebar />
    <div className="flex-1 flex flex-col">
      <Header />
      <main className="flex-1 overflow-auto p-6">
        <Outlet /> {/* Replace {children} with <Outlet /> */}
      </main>
    </div>
  </div>
);
