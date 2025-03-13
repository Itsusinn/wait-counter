# wait-counter

```rust
let counter = WaitCounter::new();
let cloned = counter.clone();
tokio::spawn(async move {
    // After simulating time-consuming operations, drop the cloned instance
    tokio::time::sleep(Duration::from_millis(1000)).await;
    drop(cloned);
});

counter.wait().await;
println!("Counter reached 1");
```