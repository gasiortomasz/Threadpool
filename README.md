# Threadpool

Don't use this crate. There are many good threadpool implementations on crate.io.
This one was written for educational purpose

## Usage
```rust
fn main() {
    {
        // create thread pool with 4 threads
        let pool = Threadpool::new(4);

        //create connection
        let (tx, rx) = unbounded();

        // submit new task
        pool.submit(move || {
            let x = heavy_computation(42);
            tx.send(x).unwrap();
        });

        // submit new task with nothing to do
        pool.submit(move || {
            thread::sleep(...);
        });

        // submit many tasks
        (0..4).for_each(|_| {
            pool.submit(move || {
                thread::sleep(...);
            });
        });
    } // pool drops - waits till all queued tasks are done and then waits till all threads exit
}
```
