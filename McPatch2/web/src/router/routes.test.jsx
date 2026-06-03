import { describe, expect, it } from 'vitest'

import { routes } from './routes.jsx'

describe('router lazy routes', () => {
  it('loads top-level pages lazily except the shared app shell', () => {
    expect(routes[0].children[0].lazy).toBeTypeOf('function')
    expect(routes[0].children[1].lazy).toBeTypeOf('function')
    expect(routes[0].children[2].lazy).toBeTypeOf('function')
  })

  it('loads dashboard child pages lazily', () => {
    const dashboardChildren = routes[0].children[2].children

    for (const child of dashboardChildren) {
      expect(child.lazy).toBeTypeOf('function')
      expect(child.element).toBeUndefined()
    }
  })
})
