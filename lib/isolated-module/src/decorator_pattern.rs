trait Greeting {
    fn greet(&self) -> String;
}

struct HelloWorld;

impl Greeting for HelloWorld {
    fn greet(&self) -> String {
        "Hello, world!".to_string()
    }
}

struct ExcitedGreeting<T> {
    inner: T,
}

impl<T> ExcitedGreeting<T> {
    fn greet(&self) -> String
    where
        T: Greeting,
    {
        let mut greeting = self.inner.greet();
        greeting.push_str("I'm so excited to be in Rust!");
        greeting
    }
}

#[tokio::test]
async fn test_main() {
    #[cfg(feature = "logging_decorator")]
    let hello = ExcitedGreeting { inner: HelloWorld };

    #[cfg(not(feature = "logging_decorator"))]
    let hello = HelloWorld;

    println!("{}", hello.greet());
}
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

trait Logging {
    fn log(&self);
}

struct LoggingFuture<F: Future + Logging> {
    inner: F,
}

impl<F: Future + Logging> Future for LoggingFuture<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = unsafe { self.map_unchecked_mut(|s| &mut s.inner) };
        inner.log();
        inner.poll(cx)
    }
}

impl<F: Future> Logging for F {
    fn log(&self) {
        println!("Polling the future!");
    }
}

async fn my_async_function() -> String {
    "Result of async computation".to_string()
}

#[cfg(test)]
mod tests {
    use crate::decorator_pattern::{LoggingFuture, my_async_function};

    #[tokio::test]
    async fn test_name() {
        let logged_future = LoggingFuture {
            inner: my_async_function(),
        };
        let result = logged_future.await;
        println!("{}", result);
    }
}
