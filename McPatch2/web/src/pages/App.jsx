import React, {useEffect, useState} from 'react';
import {Outlet} from "react-router-dom";
import {MoonStar, Sun} from "lucide-react";

const App = () => {

  const [darkMode, setDarkMode] = useState(localStorage.getItem("darkMode") === "true");

  useEffect(() => {
    if (darkMode) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    localStorage.setItem('darkMode', darkMode.toString());
  }, [darkMode]);

  return (
    <>
      <div className="dark:bg-gray-950">
        <Outlet context={{darkMode}}/>
        <button
          type="button"
          aria-label="切换深色模式"
          title="深色模式"
          onClick={() => setDarkMode(!darkMode)}
          className="fixed bottom-6 right-6 z-50 flex h-12 w-12 items-center justify-center rounded-full bg-indigo-600 text-white shadow-lg transition hover:bg-indigo-500">
          {darkMode ? <Sun className="h-5 w-5"/> : <MoonStar className="h-5 w-5"/>}
        </button>
      </div>
    </>
  );
};

export default App;
