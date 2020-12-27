# Threadpool
## Usage
```rust
fn main() {
    {
        // create thread pool with 4 threads
        let pool = Threadpool::new(4);

        //create connection
        let (tx, rx) = unbounded();

        // submit new task
        pool.submit(Box::new(move || {
            let x = heavy_computation(42);
            tx.send(x).unwrap();
        }));

        // submit new task with nothing to do
        pool.submit(Box::new(move || {
            thread::sleep(...);
        }));

        // submit many tasks
        (0..4).for_each(|_| {
            pool.submit(Box::new(move || {
                thread::sleep(...);
            }));
        });
    } // pool drops - waits till all queued tasks are done and then waits till all threads exit
}
```
