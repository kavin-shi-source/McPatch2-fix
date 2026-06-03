import React from 'react'

import App from '@/pages/App.jsx'

const lazyRoute = (loader) => async () => {
  const module = await loader()
  return { Component: module.default }
}

export const routes = [
  {
    path: '/',
    element: <App />,
    children: [
      {
        index: true,
        lazy: lazyRoute(() => import('@/pages/Home/index.jsx'))
      },
      {
        path: 'login',
        lazy: lazyRoute(() => import('@/pages/Login/index.jsx'))
      },
      {
        path: 'dashboard',
        lazy: lazyRoute(() => import('@/pages/Dashboard/index.jsx')),
        children: [
          {
            index: true,
            lazy: lazyRoute(() => import('@/pages/Dashboard/Overview/index.jsx'))
          },
          {
            path: 'directory',
            lazy: lazyRoute(() => import('@/pages/Dashboard/Directory/index.jsx'))
          },
          {
            path: 'log',
            lazy: lazyRoute(() => import('@/pages/Dashboard/Log/index.jsx'))
          },
          {
            path: 'help',
            lazy: lazyRoute(() => import('@/pages/Dashboard/Help/index.jsx'))
          },
          {
            path: 'settings',
            lazy: lazyRoute(() => import('@/pages/Dashboard/Settings/index.jsx'))
          },
          {
            path: 'modplatform',
            lazy: lazyRoute(() => import('@/pages/Dashboard/ModPlatform/index.jsx'))
          }
        ]
      },
      {
        path: '*',
        lazy: lazyRoute(() => import('@/pages/NotFound/index.jsx'))
      }
    ]
  }
]
