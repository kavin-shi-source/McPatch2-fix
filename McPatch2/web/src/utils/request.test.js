import { beforeEach, describe, expect, it, vi } from 'vitest'

const mockCreate = vi.fn()
const mockGetState = vi.fn()

vi.mock('axios', () => ({
  default: {
    create: mockCreate
  }
}))

vi.mock('@/store/index.js', () => ({
  default: {
    getState: mockGetState
  }
}))

describe('request interceptors', () => {
  beforeEach(() => {
    vi.resetModules()
    vi.clearAllMocks()
  })

  it('rejects request interceptor errors', async () => {
    const instance = {
      interceptors: {
        request: {
          use: vi.fn((fulfilled, rejected) => {
            instance.onRequestRejected = rejected
          })
        },
        response: {
          use: vi.fn()
        }
      }
    }

    mockCreate.mockReturnValue(instance)
    mockGetState.mockReturnValue({ user: { token: 'token' } })

    await import('./request.js')

    const error = new Error('request failed')
    await expect(instance.onRequestRejected(error)).rejects.toThrow('request failed')
  })

  it('rejects response interceptor errors', async () => {
    const instance = {
      interceptors: {
        request: {
          use: vi.fn()
        },
        response: {
          use: vi.fn((fulfilled, rejected) => {
            instance.onResponseRejected = rejected
          })
        }
      }
    }

    mockCreate.mockReturnValue(instance)
    mockGetState.mockReturnValue({ user: { token: 'token' } })

    await import('./request.js')

    const error = new Error('response failed')
    await expect(instance.onResponseRejected(error)).rejects.toThrow('response failed')
  })
})
