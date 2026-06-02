use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// 简单的令牌桶限流器
pub struct RateLimiter {
    max_tokens: u64,
    refill_rate: u64,
    refill_interval: Duration,
    tokens: AtomicU64,
    last_refill: Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(max_tokens: u64, refill_per_second: u64) -> Self {
        Self {
            max_tokens,
            refill_rate: refill_per_second,
            refill_interval: Duration::from_secs(1),
            tokens: AtomicU64::new(max_tokens),
            last_refill: Mutex::new(Instant::now()),
        }
    }

    /// 尝试获取一个令牌，成功返回 true，失败返回 false
    pub fn try_acquire(&self) -> bool {
        self.try_acquire_n(1)
    }

    /// 尝试获取 N 个令牌
    pub fn try_acquire_n(&self, n: u64) -> bool {
        self.refill();
        loop {
            let current = self.tokens.load(Ordering::Acquire);
            if current < n {
                return false;
            }
            if self
                .tokens
                .compare_exchange(current, current - n, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }

    /// 等待直到获取到令牌（阻塞当前线程）
    pub fn acquire(&self) {
        self.acquire_n(1)
    }

    /// 等待直到获取到 N 个令牌
    pub fn acquire_n(&self, n: u64) {
        loop {
            self.refill();
            loop {
                let current = self.tokens.load(Ordering::Acquire);
                if current < n {
                    break;
                }
                if self
                    .tokens
                    .compare_exchange(current, current - n, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn refill(&self) {
        let mut last = self.last_refill.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last);

        if elapsed >= self.refill_interval {
            let refill_count = (elapsed.as_secs() * self.refill_rate)
                + if elapsed.subsec_nanos() > 0 { 1 } else { 0 };
            let new_tokens = self
                .tokens
                .load(Ordering::Acquire)
                .saturating_add(refill_count)
                .min(self.max_tokens);
            self.tokens.store(new_tokens, Ordering::Release);
            *last = now;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_initial() {
        let limiter = RateLimiter::new(10, 5);
        assert!(limiter.try_acquire());
    }

    #[test]
    fn test_rate_limiter_exhaust() {
        let limiter = RateLimiter::new(3, 5);
        assert!(limiter.try_acquire_n(3));
        assert!(!limiter.try_acquire());
    }
}
