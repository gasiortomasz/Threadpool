extern crate threadpool;

#[cfg(test)]
mod tests {
    use crossbeam_channel::unbounded;
    use ntest::timeout;
    use threadpool::Threadpool;

    #[test]
    #[timeout(200)]
    fn smoke_test() {
        let (tx, rx) = unbounded();
        let pool = Threadpool::new(1);

        pool.submit(Box::new(move || {
            tx.send(14).unwrap();
        }));

        assert_eq!(14, rx.recv().unwrap());
    }

    #[test]
    #[timeout(200)]
    fn four_threads_test() {
        let pool = Threadpool::new(4);
        let (tx, rx) = unbounded();

        (1..=10).for_each(|i| {
            let tx_copy = tx.clone();
            pool.submit(Box::new(move || {
                let computed_value = i * 10;
                tx_copy.send(computed_value).unwrap();
            }));
        });

        let mut sum = 0;
        (1..=10).for_each(|_| {
            sum += rx.recv().unwrap();
        });
        assert_eq!(sum, 550);
    }
}
