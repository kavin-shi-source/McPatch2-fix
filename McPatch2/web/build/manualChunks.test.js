import { describe, expect, it } from 'vitest'

import { getManualChunkName } from './manualChunks.js'

describe('manual chunk strategy', () => {
  it('splits antd theme infrastructure away from page-level components', () => {
    expect(getManualChunkName('E:/project/node_modules/antd/es/config-provider/index.js')).toBe('vendor-antd-core')
    expect(getManualChunkName('E:/project/node_modules/@ant-design/cssinjs/es/index.js')).toBe('vendor-antd-core')
  })

  it('splits react router related packages into framework chunk', () => {
    expect(getManualChunkName('E:/project/node_modules/react-router-dom/dist/index.js')).toBe('vendor-react')
    expect(getManualChunkName('E:/project/node_modules/react/index.js')).toBe('vendor-react')
  })

  it('splits icon and date packages into independent chunks', () => {
    expect(getManualChunkName('E:/project/node_modules/@ant-design/icons/es/index.js')).toBe('vendor-icons')
    expect(getManualChunkName('E:/project/node_modules/lucide-react/dist/esm/icons/sun.js')).toBe('vendor-icons')
    expect(getManualChunkName('E:/project/node_modules/dayjs/dayjs.min.js')).toBeUndefined()
  })

  it('splits heavy antd feature modules into separate chunks', () => {
    expect(getManualChunkName('E:/project/node_modules/antd/es/form/index.js')).toBe('vendor-antd-form')
    expect(getManualChunkName('E:/project/node_modules/rc-field-form/es/index.js')).toBe('vendor-antd-form')
    expect(getManualChunkName('E:/project/node_modules/antd/es/modal/index.js')).toBe('vendor-antd-feedback')
    expect(getManualChunkName('E:/project/node_modules/rc-dialog/es/index.js')).toBe('vendor-antd-feedback')
  })

  it('keeps unrelated files untouched', () => {
    expect(getManualChunkName('E:/project/src/pages/Home/index.jsx')).toBeUndefined()
  })
})
