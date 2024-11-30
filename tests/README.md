# How to Run
```
cargo kani --tests
```

# Potential issues:
Concurrent features are currently out of scope for Kani. In general, the verification of concurrent programs continues to be an open research problem where most tools that analyze concurrent code lack support for other features. Because of this, Kani emits a warning whenever it encounters concurrent code and compiles as if it was sequential code.
https://model-checking.github.io/kani/rust-feature-support.html