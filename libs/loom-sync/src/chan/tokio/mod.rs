mod receiver;
mod sender;

pub use receiver::*;
pub use sender::*;

/// Create a channel for async communication.
///
/// # Patterns
/// - `open!()` - unbounded channel
/// - `open!(capacity)` - bounded channel with specified capacity
///
/// # Examples
/// ```ignore
/// let (tx, rx) = open!();        // unbounded
/// let (tx, rx) = open!(100);     // bounded with capacity 100
/// ```
#[macro_export]
macro_rules! open {
    () => {{
        let (sender, receiver) = $crate::internal::tokio::sync::mpsc::unbounded_channel();
        (
            $crate::chan::tokio::TokioSender::new($crate::chan::tokio::MpscSender::from(sender)),
            $crate::chan::tokio::TokioReceiver::new($crate::chan::tokio::MpscReceiver::from(
                receiver,
            )),
        )
    }};
    ($capacity:expr) => {{
        let (sender, receiver) = $crate::internal::tokio::sync::mpsc::channel($capacity);
        (
            $crate::chan::tokio::TokioSender::new($crate::chan::tokio::MpscSender::from(sender)),
            $crate::chan::tokio::TokioReceiver::new($crate::chan::tokio::MpscReceiver::from(
                receiver,
            )),
        )
    }};
}

#[cfg(test)]
mod tests {
    use crate::chan::{Channel, Receiver, Sender, Status};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    // === open! Macro Tests ===

    #[test]
    fn open_unbounded_creates_channel() {
        let (tx, rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!();
        assert!(tx.is_unbound());
        assert_eq!(rx.capacity(), None); // Unbounded has no capacity
    }

    #[test]
    fn open_bounded_creates_channel() {
        let (tx, rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(50);
        assert!(tx.is_bound());
        assert_eq!(rx.capacity(), Some(50));
        assert_eq!(tx.capacity(), Some(50));
    }

    #[test]
    fn open_unbounded_no_capacity_limit() {
        let (tx, rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!();
        assert_eq!(tx.capacity(), None);
        assert_eq!(rx.capacity(), None);
    }

    #[test]
    fn open_bounded_capacity() {
        let (tx, rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(100);
        assert_eq!(tx.capacity(), Some(100));
        assert_eq!(rx.capacity(), Some(100));
    }

    #[test]
    fn open_send_receive() {
        let (tx, mut rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(10);

        tx.send(42).unwrap();
        let result = rx.recv();
        assert_eq!(result, Ok(42));
    }

    // === Status Transitions ===

    #[test]
    fn channel_status_open_initially() {
        let (tx, rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(10);
        assert_eq!(tx.status(), Status::Open);
        assert_eq!(rx.status(), Status::Open);
    }

    #[test]
    fn channel_status_closed_after_sender_drop_empty() {
        let (tx, rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(10);
        drop(tx);
        assert_eq!(rx.status(), Status::Closed);
    }

    #[test]
    fn channel_status_draining_with_buffered_items() {
        let (tx, rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(10);

        tx.send(1).unwrap();
        tx.send(2).unwrap();
        drop(tx);

        assert_eq!(rx.status(), Status::Draining);

        let mut rx = rx;
        assert_eq!(rx.recv().unwrap(), 1);
        assert_eq!(rx.recv().unwrap(), 2);
        assert_eq!(rx.status(), Status::Closed);
    }

    #[test]
    fn channel_status_closed_after_receiver_close() {
        let (tx, mut rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(10);

        rx.close();
        assert!(rx.status().is_closing());
        assert!(tx.status().is_closed());
    }

    // === Concurrent Producer Consumer ===

    #[test]
    fn concurrent_producer_consumer() {
        let (tx, mut rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(100);

        let producer_count = Arc::new(AtomicUsize::new(0));
        let consumer_count = Arc::new(AtomicUsize::new(0));

        let producer_count_clone = Arc::clone(&producer_count);
        let producer = thread::spawn(move || {
            for i in 0..100 {
                if tx.send(i).is_ok() {
                    producer_count_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        let consumer_count_clone = Arc::clone(&consumer_count);
        let consumer = thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(_) => {
                        consumer_count_clone.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(crate::chan::error::RecvError::Closed) => break,
                    Err(crate::chan::error::RecvError::Empty) => {
                        thread::sleep(Duration::from_micros(100));
                    }
                }
            }
        });

        producer.join().unwrap();
        consumer.join().unwrap();

        assert_eq!(
            producer_count.load(Ordering::SeqCst),
            consumer_count.load(Ordering::SeqCst)
        );
    }

    #[test]
    fn multiple_producers_single_consumer() {
        let (tx, mut rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(1000);

        let num_producers = 4;
        let items_per_producer = 100;
        let mut producers = vec![];

        for _ in 0..num_producers {
            let tx_clone = tx.clone();
            let handle = thread::spawn(move || {
                for i in 0..items_per_producer {
                    let _ = tx_clone.send(i);
                }
            });
            producers.push(handle);
        }

        drop(tx);

        for handle in producers {
            handle.join().unwrap();
        }

        let mut count = 0;
        loop {
            match rx.recv() {
                Ok(_) => count += 1,
                Err(crate::chan::error::RecvError::Closed) => break,
                Err(crate::chan::error::RecvError::Empty) => continue,
            }
        }

        assert_eq!(count, num_producers * items_per_producer);
    }

    // === Type Parameter Tests ===

    #[test]
    fn channel_with_string() {
        let (tx, mut rx): (super::TokioSender<String>, super::TokioReceiver<String>) = open!(10);
        tx.send("hello".to_string()).unwrap();
        assert_eq!(rx.recv().unwrap(), "hello");
    }

    #[test]
    fn channel_with_vec() {
        let (tx, mut rx): (super::TokioSender<Vec<u8>>, super::TokioReceiver<Vec<u8>>) = open!(10);
        tx.send(vec![1, 2, 3]).unwrap();
        assert_eq!(rx.recv().unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn channel_with_struct() {
        #[derive(Debug, PartialEq)]
        struct Message {
            id: u64,
            data: String,
        }

        let (tx, mut rx): (super::TokioSender<Message>, super::TokioReceiver<Message>) = open!(10);

        let msg = Message {
            id: 42,
            data: "test".to_string(),
        };
        tx.send(msg).unwrap();

        let received = rx.recv().unwrap();
        assert_eq!(received.id, 42);
        assert_eq!(received.data, "test");
    }

    // === Edge Cases ===

    #[test]
    fn channel_capacity_one() {
        let (tx, mut rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(1);

        tx.send(1).unwrap();
        assert_eq!(rx.recv().unwrap(), 1);

        tx.send(2).unwrap();
        assert_eq!(rx.recv().unwrap(), 2);
    }

    #[test]
    fn unbounded_channel_large_volume() {
        let (tx, mut rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!();

        for i in 0..10000 {
            tx.send(i).unwrap();
        }

        drop(tx);

        let mut count = 0;
        loop {
            match rx.recv() {
                Ok(_) => count += 1,
                Err(crate::chan::error::RecvError::Closed) => break,
                Err(crate::chan::error::RecvError::Empty) => continue,
            }
        }

        assert_eq!(count, 10000);
    }

    #[test]
    fn channel_fifo_order() {
        let (tx, mut rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(100);

        for i in 0..100 {
            tx.send(i).unwrap();
        }
        drop(tx);

        for expected in 0..100 {
            assert_eq!(rx.recv().unwrap(), expected);
        }
    }

    // === Race Condition Tests ===

    #[tokio::test]
    async fn close_while_sending() {
        // Use bounded channel with send_timeout
        let (tx, mut rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!();
        let tx_clone = tx.clone();
        let sender = tokio::spawn(async move {
            for i in 0..1000 {
                // Use send_timeout to avoid blocking indefinitely
                match tx_clone.send_timeout(i, Duration::from_millis(100)).await {
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
        });

        tokio::time::sleep(Duration::from_millis(1)).await;
        rx.close();
        drop(tx); // Drop the original sender too

        // Wait for sender to finish (with timeout via tokio)
        let _ = tokio::time::timeout(Duration::from_secs(5), sender).await;
        assert!(rx.status().is_closing());
    }

    #[test]
    fn drop_sender_while_receiving() {
        use std::time::Instant;

        let (tx, mut rx): (super::TokioSender<i32>, super::TokioReceiver<i32>) = open!(10);

        tx.send(1).unwrap();
        tx.send(2).unwrap();

        let receiver = thread::spawn(move || {
            let mut received = vec![];
            let start = Instant::now();
            loop {
                // Timeout after 2 seconds
                if start.elapsed() > Duration::from_secs(2) {
                    break;
                }
                match rx.recv() {
                    Ok(v) => received.push(v),
                    Err(crate::chan::error::RecvError::Closed) => break,
                    Err(crate::chan::error::RecvError::Empty) => {
                        thread::sleep(Duration::from_micros(100));
                    }
                }
            }
            received
        });

        thread::sleep(Duration::from_millis(10));
        drop(tx);

        let received = receiver.join().expect("receiver thread panicked");
        assert!(received.contains(&1));
        assert!(received.contains(&2));
    }
}
