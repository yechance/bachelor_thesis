extern crate csv;

use std::error::Error;
use std::fs::File;
use csv::Writer;

fn main() -> Result<(), Box<dyn Error>> {
    // Open or create a new CSV file
    let file = File::create("../../example.csv")?;

    // Create a CSV writer
    let mut writer = Writer::from_writer(file);

    // Write headers to the CSV file
    writer.write_record(&["Name", "Age", "City"])?;

    // Sample data to write to the CSV file
    let data = vec![
        vec!["John Doe", "25", "New York"],
        vec!["Jane Smith", "30", "San Francisco"],
        // Add more rows as needed
    ];

    // Write data to the CSV file
    for row in data {
        writer.write_record(&row)?;
    }

    // Flush the writer to ensure all data is written to the file
    writer.flush()?;

    println!("Data has been written to example.csv");

    Ok(())
}