use futures::{Async, Future, Poll};
use std::time::{Duration, Instant};
use tokio::timer::Delay;

type Task<Out, Error> = Box<Future<Item = Out, Error = Error> + Send>;
type FutureCreator<Out, Error> = Box<Fn() -> Task<Out, Error> + Send + 'static>;

enum Inner<Out, Error>
where
    Error: ::std::error::Error,
{
    WaitForRetry { delay: Delay },
    Retrying { task: Task<Out, Error> },
}

impl<Out, Error> Inner<Out, Error>
where
    Error: ::std::error::Error,
{
    fn wait(delay: Duration) -> Self {
        Inner::WaitForRetry {
            delay: Delay::new(Instant::now() + delay),
        }
    }

    fn retry(task: Task<Out, Error>) -> Self {
        Inner::Retrying { task }
    }
}

pub struct RetryFuture<Out, Error>
where
    Error: ::std::error::Error,
{
    action_title: Option<String>,
    retry_interval: Duration,
    retry_count: u64,
    retry_limit: u64,
    create_future: FutureCreator<Out, Error>,
    inner: Inner<Out, Error>,
}

impl<Out, Error> RetryFuture<Out, Error>
where
    Error: ::std::error::Error,
{
    pub fn new(
        action_title: Option<String>,
        retry_interval: Duration,
        retry_limit: u64,
        create_future: FutureCreator<Out, Error>,
    ) -> RetryFuture<Out, Error> {
        let retry_count = 1;
        let inner = Inner::retry(create_future());
        if let Some(ref action_title) = action_title {
            info!("Try to {} ({}/{})", action_title, retry_count, retry_limit);
        }

        RetryFuture {
            action_title,
            retry_interval,
            retry_count,
            retry_limit,
            create_future,
            inner,
        }
    }
}

impl<Out, Error> Future for RetryFuture<Out, Error>
where
    Error: ::std::error::Error,
{
    type Item = Out;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match &mut self.inner {
                Inner::WaitForRetry { delay } => match delay.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(_)) => {
                        self.inner = Inner::retry((self.create_future)());
                    }
                    Err(_err) => {
                        unreachable!();
                    }
                },
                Inner::Retrying { task } => match task.poll() {
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Ok(Async::Ready(out)) => return Ok(Async::Ready(out)),
                    Err(err) => {
                        if self.retry_count <= self.retry_limit {
                            self.retry_count += 1;
                            if let Some(ref action_title) = self.action_title {
                                info!(
                                    "Try to {} after {:?} ({}/{})",
                                    action_title,
                                    self.retry_interval,
                                    self.retry_count,
                                    self.retry_limit,
                                );
                            }
                            self.inner = Inner::wait(self.retry_interval);
                        } else {
                            if let Some(ref action_title) = self.action_title {
                                warn!("Failed to {}, error: {}", action_title, err);
                            }

                            return Err(err);
                        }
                    }
                },
            }
        }
    }
}
