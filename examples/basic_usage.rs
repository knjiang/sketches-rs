use ddsketch_rs::DDSketch;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new DDSketch with 2% relative accuracy
    let mut sketch = DDSketch::new(0.02)?;
    
    // Add some values
    for i in 1..=1000 {
        sketch.add(i as f64);
    }
    
    // Get some quantiles
    let median = sketch.get_quantile_value(0.5)?;
    let p90 = sketch.get_quantile_value(0.9)?;
    let p99 = sketch.get_quantile_value(0.99)?;
    
    println!("Total count: {}", sketch.count());
    println!("Median (50th percentile): {:.2}", median);
    println!("90th percentile: {:.2}", p90);
    println!("99th percentile: {:.2}", p99);
    println!("Min: {:.2}", sketch.min().unwrap());
    println!("Max: {:.2}", sketch.max().unwrap());
    
    // Test merging
    let mut sketch2 = DDSketch::new(0.02)?;
    for i in 1001..=2000 {
        sketch2.add(i as f64);
    }
    
    sketch.merge(&sketch2)?;
    
    println!("\nAfter merging:");
    println!("Total count: {}", sketch.count());
    println!("Median: {:.2}", sketch.get_quantile_value(0.5)?);
    println!("Min: {:.2}", sketch.min().unwrap());
    println!("Max: {:.2}", sketch.max().unwrap());
    
    Ok(())
}
