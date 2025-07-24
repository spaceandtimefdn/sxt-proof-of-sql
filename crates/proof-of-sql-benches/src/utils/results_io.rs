use csv::{ReaderBuilder, Writer, WriterBuilder};
use plotters::prelude::{SeriesLabelPosition::LowerRight, *};
use plotters_bitmap::BitMapBackend;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::OpenOptions,
    io::BufWriter,
    path::Path,
};

/// Writes the header to the CSV file.
///
/// # Arguments
/// * `writer` - A mutable reference to the CSV writer.
///
/// # Panics
/// * If the header cannot be written to the CSV file.
fn write_csv_header(writer: &mut Writer<BufWriter<std::fs::File>>) {
    writer
        .write_record([
            "commitment_scheme",
            "query",
            "table_size",
            "generate_proof (ms)",
            "verify_proof (ms)",
            "iteration",
        ])
        .expect("Failed to write headers to CSV file.");
}

/// Appends values to an existing CSV file or creates a new one if it doesn't exist.
///
/// # Arguments
/// * `file_path` - The path to the CSV file.
/// * `new_row` - A vector of strings to append to the file.
///
/// # Panics
/// * If the file cannot be opened, read, or appended.
pub fn append_to_csv(file_path: &Path, new_row: &[String]) {
    // Open the file in append mode or create it if it doesn't exist
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open or create the CSV file.");

    // Check if the file is empty to determine if we need to write headers
    let is_empty = file.metadata().map(|m| m.len() == 0).unwrap_or(true);

    // Create a CSV writer
    let mut writer = WriterBuilder::new().from_writer(BufWriter::new(file));

    // Write headers if the file is empty
    if is_empty {
        write_csv_header(&mut writer);
    }

    // Write new row to the CSV file
    writer
        .write_record(new_row)
        .expect("Failed to write row to CSV file.");

    writer.flush().expect("Failed to flush CSV writer.");
}

/// Generates a color based on the .
///
/// # Returns
/// * An `RGBColor` object representing the generated color.
#[expect(clippy::cast_sign_loss)]
fn sxt_colors() -> Vec<RGBColor> {
    // List of hex color values
    let hex_colors = vec![
        0x00CC_0AAC,
        0x0050_00BF,
        0x006F_4D80,
        0x0099_CCFF,
        0x00FF_96E8,
        0x00C4_9AFF,
        0x009C_D5CF,
        0x0090_91FC,
    ];

    // Convert each hex color to an RGBColor and collect into a vector
    hex_colors
        .into_iter()
        .map(|hex_color| {
            let r = ((hex_color >> 16) & 0xFF) as u8; // Extract red
            let g = ((hex_color >> 8) & 0xFF) as u8; // Extract green
            let b = (hex_color & 0xFF) as u8; // Extract blue
            RGBColor(r, g, b)
        })
        .collect()
}

/// Calculates the median of a vector of f64 values.
///
/// # Arguments
/// * `data` - A mutable reference to a vector of f64 values.
///
/// # Returns
/// * The median value of the vector.
///
/// # Panics
/// * If the vector is empty.
fn calculate_median(data: &mut [f64]) -> f64 {
    data.sort_by(|a, b| a.partial_cmp(b).unwrap()); // Sort the data
    let len = data.len();
    if len == 0 {
        return 0.0;
    }
    if len % 2 == 1 {
        data[len / 2]
    } else {
        f64::midpoint(data[len / 2 - 1], data[len / 2])
    }
}

/// Computes the median data vector from a vector of tuples.
///
/// # Arguments
/// * `data` - A reference to a vector of tuples, where each tuple contains a size (u64) and a time (f64).
///
/// # Returns
/// * A vector of tuples, where each tuple contains a size (u64) and the median time (f64) for that size.
fn compute_median_data_vector(data: &[(u64, f64)]) -> Vec<(u64, f64)> {
    let mut grouped_data: HashMap<u64, Vec<f64>> = HashMap::new();

    // Group data by size
    for &(size, time) in data {
        grouped_data.entry(size).or_default().push(time);
    }

    // Compute the median for each size
    let mut median_data: Vec<(u64, f64)> = grouped_data
        .into_iter()
        .map(|(size, mut times)| (size, calculate_median(&mut times)))
        .collect();

    // Sort the result by size
    median_data.sort_by_key(|&(size, _)| size);

    median_data
}

struct BenchResult {
    schemes: Vec<String>,
    query: Vec<String>,
    table_size: Vec<String>,
    generate_proof_times: Vec<f64>,
    verify_proof_times: Vec<f64>,
}

impl BenchResult {
    fn new() -> Self {
        Self {
            schemes: Vec::new(),
            query: Vec::new(),
            table_size: Vec::new(),
            generate_proof_times: Vec::new(),
            verify_proof_times: Vec::new(),
        }
    }

    fn read_csv(&mut self, csv_file_path: &Path) -> Result<(), Box<dyn Error>> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_path(csv_file_path)?;

        for result in reader.records() {
            let record = result?;
            self.schemes.push(record[0].to_string());
            self.query.push(record[1].to_string());
            self.table_size.push(record[2].to_string());
            self.generate_proof_times.push(record[3].parse::<f64>()?);
            self.verify_proof_times.push(record[4].parse::<f64>()?);
        }

        Ok(())
    }
}

/// Computes the median time data for a given scheme and query.
///
/// # Arguments
/// * `scheme` - A reference to a string representing the scheme.
/// * `query` - A reference to a string representing the query.
/// * `csv_rows` - A reference to a `BenchResult` object containing the benchmark results.
///
/// # Returns
/// * A tuple containing two vectors: the first vector contains the median generate proof times,
///   and the second vector contains the median verify proof times.
///
/// # Panics
/// * If the scheme or query is not found in the CSV data.
/// * If the table size cannot be parsed as u64.
/// * If the generate or verify proof times cannot be parsed as f64.
fn median_time_data(
    scheme: &String,
    query: &String,
    time_data: &[f64],
    csv_rows: &BenchResult,
) -> Vec<(u64, f64)> {
    let mut data: Vec<(u64, f64)> = Vec::new();

    // Find rows that match the current scheme and query
    for (i, s) in csv_rows.schemes.iter().enumerate() {
        if s == scheme && csv_rows.query[i] == *query {
            let size = csv_rows.table_size[i].parse::<u64>().unwrap();
            data.push((size, time_data[i]));
        }
    }

    // Sort the data by table size (x-axis)
    data.sort_by_key(|&(size, _)| size);

    // Compute the median data vector for data
    compute_median_data_vector(&data)
}

/// Finds all queries for a given scheme.
///
/// # Arguments
/// * `scheme` - A reference to a string representing the scheme.
/// * `csv_rows` - A reference to a `BenchResult` object containing the benchmark results.
///
/// # Returns
/// * A vector of strings representing the queries for the given scheme.
fn ordered_queries(scheme: &String, csv_rows: &BenchResult) -> Vec<String> {
    // Get all queries for the current scheme
    let mut query_set: HashSet<String> = HashSet::new();
    for (i, s) in csv_rows.schemes.iter().enumerate() {
        if s == scheme {
            query_set.insert(csv_rows.query[i].clone());
        }
    }

    // Sort the queries in alphabetical order
    let mut queries: Vec<String> = query_set.into_iter().collect();
    queries.sort();
    queries
}

/// Finds the minimum and maximum table sizes for a given scheme.
///
/// # Arguments
/// * `scheme` - A reference to a string representing the scheme.
/// * `csv_rows` - A reference to a `BenchResult` object containing the benchmark results.
///
/// # Returns
/// * A tuple containing the minimum and maximum table sizes (in rows) for the given scheme.
fn min_max_table_size(scheme: &String, csv_rows: &BenchResult) -> (u64, u64) {
    // Find the table sizes for the current scheme
    let filtered_table_sizes: Vec<u64> = csv_rows
        .schemes
        .iter()
        .enumerate()
        .filter(|(_, s)| *s == scheme)
        .filter_map(|(i, _)| csv_rows.table_size[i].parse::<u64>().ok())
        .collect();

    // Calculate the minimum and maximum table sizes
    let min_table_size = filtered_table_sizes.iter().copied().min().unwrap_or(0);
    let max_table_size = filtered_table_sizes.iter().copied().max().unwrap_or(0);

    (min_table_size, max_table_size)
}

/// Finds the maximum execution time for a given scheme.
///
/// # Arguments
/// * `scheme` - A reference to a string representing the scheme.
/// * `csv_rows` - A reference to a `BenchResult` object containing the benchmark results.
///
/// # Returns
/// * The maximum execution time (in milliseconds) for the given scheme.
///
/// # Panics
/// * If the scheme is not found in the CSV data.
/// * If the execution times cannot be parsed as f64.
fn max_execution_time(scheme: &String, csv_rows: &BenchResult) -> f64 {
    // Find the generate proof execution times for the current scheme
    let filtered_generate_times: Vec<f64> = csv_rows
        .schemes
        .iter()
        .enumerate()
        .filter(|(_, s)| *s == scheme)
        .map(|(i, _)| csv_rows.generate_proof_times[i])
        .collect();

    // Find the verify proof execution times for the current scheme
    let filtered_verify_times: Vec<f64> = csv_rows
        .schemes
        .iter()
        .enumerate()
        .filter(|(_, s)| *s == scheme)
        .map(|(i, _)| csv_rows.verify_proof_times[i])
        .collect();

    // The max will likely be the generate proof time, but result is added in case we end up
    // with an error or inefficient code in the verify proof time.
    f64::max(
        filtered_generate_times
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0),
        filtered_verify_times
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0),
    )
}

/// Draws a chart from a CSV file containing benchmark results.
///
/// # Arguments
/// * `csv_file_path` - The path to the CSV file.
/// * `graph_file_path` - The path to the output graph file folder.
///
/// # Returns
/// * A `Result` indicating success or failure.
///
/// # Panics
/// * If the CSV file cannot be read.
/// * If the graph cannot be created or drawn.
/// * If the chart cannot be configured or drawn.
/// * If the data cannot be parsed or processed.
/// * If the file cannot be opened or created.
/// * If the CSV writer cannot be created or flushed.
#[expect(clippy::too_many_lines)]
pub fn draw_chart_from_csv(
    csv_file_path: &Path,
    graph_file_path: &Path,
    in_seconds: bool,
    with_verify: bool,
) -> Result<(), Box<dyn Error>> {
    println!("Drawing chart from CSV file: {}", csv_file_path.display());

    // Read the CSV file into memory
    let mut csv_rows = BenchResult::new();
    csv_rows.read_csv(csv_file_path)?;

    // Get all commitment schemes in data
    let mut schemes_set: HashSet<String> = HashSet::new();
    for scheme in &csv_rows.schemes {
        schemes_set.insert(scheme.clone());
    }

    // For each scheme, create a chart for each query
    for scheme in &schemes_set {
        // Get all queries for the current scheme
        let queries = ordered_queries(scheme, &csv_rows);

        // Create the file name related to the scheme for the chart
        let scheme_graph_path = graph_file_path.to_path_buf().join(format!("{scheme}.png"));

        println!("Drawing chart for scheme: {}", scheme_graph_path.display());

        // Create the chart root
        let root = BitMapBackend::new(&scheme_graph_path, (1280, 720)).into_drawing_area();
        root.fill(&BLACK)?;

        // Find the min and max table sizes for the x-axis
        let (min_table_size, max_table_size) = min_max_table_size(scheme, &csv_rows);

        // Find the min and max execution times and adjust the y-axis label based on the `in_seconds` flag
        let (max_time, y_label) = if in_seconds {
            (
                max_execution_time(scheme, &csv_rows) / 1000.0,
                "Generate Proof Time (s)",
            )
        } else {
            (
                max_execution_time(scheme, &csv_rows),
                "Generate Proof Time (ms)",
            )
        };

        // Create the chart
        let vm_name = String::from("Multi-A100");
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("Proof of SQL Query Performance - {scheme} - {vm_name}"),
                ("sans-serif", 30).into_font().color(&WHITE),
            )
            .margin(10)
            .margin_right(50)
            .x_label_area_size(50)
            .y_label_area_size(70)
            .build_cartesian_2d(
                (min_table_size..max_table_size).log_scale(),
                (0.1_f64..max_time).log_scale(),
            )?;

        // 200,000 rows and under
        /*
        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("Proof of SQL Query Performance - {scheme} - {vm_name}"),
                ("sans-serif", 30).into_font().color(&WHITE),
            )
            .margin(10)
            .margin_right(50)
            .x_label_area_size(50)
            .y_label_area_size(70)
            .build_cartesian_2d(
                min_table_size..max_table_size,
                0._f64..max_time,
            )?;
        */

        chart
            .configure_mesh()
            .y_desc(y_label)
            .x_desc("Table Size (# of rows)")
            .x_label_style(("sans-serif", 20).into_font().color(&WHITE))
            .y_label_style(("sans-serif", 20).into_font().color(&WHITE))
            .axis_style(WHITE)
            .disable_mesh()
            .x_labels(16) // Set the number of x-axis labels to 10 (evenly spaced)
            .x_label_formatter(&|x| format!("{x:.0}")) // Format x-axis labels as integers
            .draw()?;

        let sxt_colors: Vec<RGBColor> = sxt_colors();
        for (i, q) in queries.iter().enumerate() {
            let query_color: RGBColor = sxt_colors[i % sxt_colors.len()];

            println!("index: {i:?}");
            println!("Drawing chart for query: {q:?}");

            // Vectors to store data for the current query
            let mut generate_median_data = median_time_data(
                scheme,
                q,
                csv_rows.generate_proof_times.as_slice(),
                &csv_rows,
            );

            let mut verify_median_data =
                median_time_data(scheme, q, csv_rows.verify_proof_times.as_slice(), &csv_rows);

            // Convert to seconds
            if in_seconds {
                for (_, time) in &mut generate_median_data {
                    *time /= 1000_f64;
                }

                for (_, time) in &mut verify_median_data {
                    *time /= 1000_f64;
                }
            }

            // Draw the generate and verify line data for the current query
            chart.draw_series(LineSeries::new(
                generate_median_data.iter().copied(),
                &query_color,
            ))?;

            if with_verify {
                chart.draw_series(LineSeries::new(
                    verify_median_data.iter().copied(),
                    &query_color,
                ))?;
            }

            // Draw generate proof time data points on chart
            chart
                .draw_series(
                    generate_median_data
                        .iter()
                        .map(|&(x, y)| Circle::new((x, y), 5, query_color.filled())),
                )?
                .label(q.to_string())
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], query_color));

            // Draw verify proof time data points on chart
            if with_verify {
                chart.draw_series(verify_median_data.iter().map(|&(x, y)| {
                    Circle::new((x, y), 5, ShapeStyle::from(query_color).filled())
                }))?;
            }
        }

        // Draw the legend
        chart
            .configure_series_labels()
            .label_font(("sans-serif", 20).into_font())
            .position(LowerRight)
            .background_style(WHITE)
            .border_style(BLACK)
            .draw()?;
    }

    Ok(())
}
