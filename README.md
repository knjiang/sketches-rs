# sketches-rs

A Rust implementation of DDSketch, a distributed quantile sketch that provides relative-error guarantees for quantile estimation.

## What is DDSketch?

DDSketch is a probabilistic data structure for estimating quantiles (percentiles) in data streams. Unlike traditional quantile estimation methods that provide absolute error guarantees, DDSketch provides relative error guarantees, making it particularly useful for metrics that span several orders of magnitude.

Key properties:
- **Relative Error**: If the true quantile is `q`, DDSketch returns a value `q'` such that `|q - q'| / q ≤ α`, where `α` is the relative accuracy parameter
- **Mergeability**: Multiple sketches can be merged to get the quantile of the combined dataset
- **Bounded Memory**: Memory usage is bounded and doesn't grow indefinitely

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
ddsketch-rs = "0.1"
```

### Basic Usage

```rust
use ddsketch_rs::DDSketch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new DDSketch with 2% relative accuracy
    let mut sketch = DDSketch::new(0.02)?;
    
    // Add values
    for i in 1..=1000 {
        sketch.add(i as f64);
    }
    
    // Get quantiles
    let median = sketch.get_quantile_value(0.5)?;
    let p90 = sketch.get_quantile_value(0.9)?;
    let p99 = sketch.get_quantile_value(0.99)?;
    
    println!("Median: {:.2}", median);
    println!("90th percentile: {:.2}", p90);
    println!("99th percentile: {:.2}", p99);
    
    Ok(())
}
```

### Merging Sketches

```rust
use ddsketch_rs::DDSketch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sketch1 = DDSketch::new(0.02)?;
    let mut sketch2 = DDSketch::new(0.02)?;
    
    // Add different data to each sketch
    for i in 1..=500 {
        sketch1.add(i as f64);
    }
    
    for i in 501..=1000 {
        sketch2.add(i as f64);
    }
    
    // Merge sketch2 into sketch1
    sketch1.merge(&sketch2)?;
    
    // Now sketch1 contains data from both sketches
    let median = sketch1.get_quantile_value(0.5)?;
    println!("Combined median: {:.2}", median);
    
    Ok(())
}
```

### Memory-Bounded Sketches

```rust
use ddsketch_rs::DDSketch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sketch with a maximum of 1024 bins
    let mut sketch = DDSketch::with_max_bins(0.02, 1024)?;
    
    // Add many values - the sketch will automatically collapse bins
    // to maintain the memory bound
    for i in 1..=100000 {
        sketch.add(i as f64);
    }
    
    let p99 = sketch.get_quantile_value(0.99)?;
    println!("99th percentile: {:.2}", p99);
    
    Ok(())
}
```

## Examples

Run the basic usage example:

```bash
cargo run --example basic_usage
```

Run benchmarks:

```bash
cargo bench
```

## References

- [DDSketch paper](https://www.vldb.org/pvldb/vol12/p2195-masson.pdf) - The original paper describing the DDSketch algorithm
- [DataDog's Go implementation](https://github.com/DataDog/sketches-go) - Reference implementation in Go
