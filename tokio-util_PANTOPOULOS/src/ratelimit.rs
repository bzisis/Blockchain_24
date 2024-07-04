//! A rate limit implementation to enforce a specific rate.
/// This module provides tools for rate limiting, to control the frequency of operations, such as API requests, transaction submissions. 

/// Imports components for futures, pinning task context management and time handling,
/// for implementing asynchronous rate limiting.
use std::{
    future::{poll_fn, Future},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

/// Imports the "Sleep" type from the Tokio crate, used for asynchronously waiting until a specified instant in time.
use tokio::time::Sleep;

/// Given a [Rate] this type enforces a rate limit.
#[derive(Debug)]

/// "RateLimit" is a structure that enforces rate limits based on the provided "Rate".
pub struct RateLimit {

    /// Stores the "Rate" which defines the limit (number of allowed actions) and the time duration.
    rate: Rate,
    /// Tracks the current state of the "RateLimit", whether it is "Ready" to accept new actions or "Limited" due to reaching the rate limit.
    state: State,
    /// An async sleep timer used to pause until the rate limit window resets. This is pinned to ensure it stays at the same memory location.
    sleep: Pin<Box<Sleep>>,
}

// === impl RateLimit ===

impl RateLimit {
    /// Create a new rate limiter
    /// This function initializes a "RateLimit" with the specified "Rate", setting up the initial state and timer.
    pub fn new(rate: Rate) -> Self {

        /// Sets the initial time to the current moment.
        let until = tokio::time::Instant::now();
        /// Sets the initial state to "Ready", with the full number of allowed actions remaining
        let state = State::Ready { until, remaining: rate.limit() };

        /// Initializes the sleep timer to expire at until (which it is initalized to now).
        /// So it expires immidiatly
        Self { rate, state, sleep: Box::pin(tokio::time::sleep_until(until)) }
    }

    /// Returns the configured limit of the [`RateLimit`]
    /// This function provides access to the rate limit's maximum number of allowed actions per time window.
    pub const fn limit(&self) -> u64 {
        self.rate.limit()
    }

    /// Checks if the [`RateLimit`] is ready to handle a new call
    /// This function determines if the "RateLimit" can accept a new action by polling the state and the sleep timer.
    pub fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        match self.state {

            /// If the state is `Ready`, it means there are remaining actions allowed in the current time window.
            State::Ready { .. } => return Poll::Ready(()),
            ///If the state is `Limited`, it checks if the sleep timer has completed.
            State::Limited => {

                /// If the timer is still pending, return "Poll::Pending" indicating the rate limit is not ready yet.
                if Pin::new(&mut self.sleep).poll(cx).is_pending() {
                    return Poll::Pending
                }
            }
        }
        
        /// If the timer has completed, reset the state to `Ready` and start a new time window.
        self.state = State::Ready {
            until: tokio::time::Instant::now() + self.rate.duration(),
            remaining: self.rate.limit(),
        };

        Poll::Ready(())
    }

    /// Wait until the [`RateLimit`] is ready.
    pub async fn wait(&mut self) {
        poll_fn(|cx| self.poll_ready(cx)).await
    }

    /// Updates the [`RateLimit`] when a new call was triggered
    ///
    /// # Panics
    ///
    /// Panics if [`RateLimit::poll_ready`] returned [`Poll::Pending`]
    /// This function should only be called when "poll_ready" indicates the rate limit is ready to handle a new action.
    pub fn tick(&mut self) {
        match self.state {

            /// If the state is "Ready", update the remaining quantity and adjust the state if necessary.
            State::Ready { mut until, remaining: mut rem } => {
                let now = tokio::time::Instant::now();

                /// If the period has elapsed, reset it.(reset the period)
                if now >= until {
                    until = now + self.rate.duration();
                    rem = self.rate.limit();
                }

                /// Decrement the remaining quantity and update the state.
                if rem > 1 {
                    rem -= 1;
                    self.state = State::Ready { until, remaining: rem };
                } else {
                    /// If no remaining actions are allowed, set the state to "Limited" and reset the sleep timer.
                    self.sleep.as_mut().reset(until);
                    self.state = State::Limited;
                }
            }
            /// If the state is "Limited", this function should not be called, "poll_ready" should be called first to check readiness.
            State::Limited => panic!("RateLimit limited; poll_ready must be called first"),
        }
    }
}

/// Tracks the state of the [`RateLimit`]
#[derive(Debug)]

///  Enum to represent the state of the `RateLimit`, either "Limited" when rate limiting is active,
/// or "Ready" with details about the allowed time period and remaining actions.
/// "until" indicates when the current rate limit period ends,
/// and "remaining" showes the number of actions left in the current period.
enum State {
    /// Currently limited
    Limited,
    Ready {
        until: tokio::time::Instant,
        remaining: u64,
    },
}

/// A rate of requests per time period.
#[derive(Debug, Copy, Clone)]

///The "Rate" struct defines a rate limit as a number of allowed actions ("limit") over a specific duration ("duration").
/// This is used to configure the "RateLimit" and control how frequently actions can be performed.
pub struct Rate {
    limit: u64,
    duration: Duration,
}

impl Rate {
    /// Create a new [Rate] with the given `limit/duration` ratio.
    pub const fn new(limit: u64, duration: Duration) -> Self {
        Self { limit, duration }
    }

    /// Provides the limit of actions per period for this "Rate".
    const fn limit(&self) -> u64 {
        self.limit
    }

    /// Provides the duration of the rate limit period.
    const fn duration(&self) -> Duration {
        self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asynchronous test to verify the functionality of the "RateLimit" struct.
    /// This test checks if the rate limiter enforces the specified limits correctly.
    #[tokio::test]
    async fn test_rate_limit() {

         /// Creates a "RateLimit" allowing 2 actions per 500 milliseconds.
        let mut limit = RateLimit::new(Rate::new(2, Duration::from_millis(500)));

        /// Polls the rate limiter to check if it is ready.
        poll_fn(|cx| {
            /// it should initially be ready
            assert!(limit.poll_ready(cx).is_ready());
            Poll::Ready(())
        })
        .await;

        /// Processes the first action and decrements the remaining quantity
        limit.tick();

        /// Polls the rate limiter again to check if it is ready, which it should be after the first action.
        poll_fn(|cx| {
            assert!(limit.poll_ready(cx).is_ready());
            Poll::Ready(())
        })
        .await;

        /// Processes the second action, reaching the limit.
        limit.tick();

        /// Polls the rate limiter to check if it is ready; it should now be pending due to the rate limit being hit.
        poll_fn(|cx| {
            assert!(limit.poll_ready(cx).is_pending());
            Poll::Ready(())
        })
        .await;

        /// Waits for the duration of the rate limit period to expire.
        tokio::time::sleep(limit.rate.duration).await;

        /// Polls the rate limiter again to check if it is ready after the wait period, which it should be.
        poll_fn(|cx| {
            assert!(limit.poll_ready(cx).is_ready());
            Poll::Ready(())
        })
        .await;
    }
}
