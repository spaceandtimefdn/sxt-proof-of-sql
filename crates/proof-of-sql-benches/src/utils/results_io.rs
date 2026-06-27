use csv::{Writer, WriterBuilder};
use std::{fs::OpenOptions, io::BufWriter, path::Path};

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

#[cfg(test)]
mod tests {
    use super::append_to_csv;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_csv_path(test_name: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after Unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "proof_of_sql_benches_{test_name}_{unique_suffix}"
        ));
        fs::create_dir_all(&dir).expect("failed to create temporary test directory");
        dir.join("results.csv")
    }

    #[test]
    fn append_to_csv_writes_header_for_new_file() {
        let file_path = temp_csv_path("writes_header");
        let row = [
            "dory".to_string(),
            "Filter".to_string(),
            "128".to_string(),
            "10".to_string(),
            "2".to_string(),
            "1".to_string(),
        ];

        append_to_csv(&file_path, &row);

        let contents = fs::read_to_string(&file_path).expect("failed to read CSV file");
        let lines: Vec<_> = contents.lines().collect();
        assert_eq!(
            lines[0],
            "commitment_scheme,query,table_size,generate_proof (ms),verify_proof (ms),iteration"
        );
        assert_eq!(lines[1], "dory,Filter,128,10,2,1");
    }

    #[test]
    fn append_to_csv_does_not_repeat_header_for_existing_file() {
        let file_path = temp_csv_path("does_not_repeat_header");
        let first_row = [
            "dory".to_string(),
            "Filter".to_string(),
            "128".to_string(),
            "10".to_string(),
            "2".to_string(),
            "1".to_string(),
        ];
        let second_row = [
            "hyperkzg".to_string(),
            "Aggregate".to_string(),
            "256".to_string(),
            "20".to_string(),
            "4".to_string(),
            "2".to_string(),
        ];

        append_to_csv(&file_path, &first_row);
        append_to_csv(&file_path, &second_row);

        let contents = fs::read_to_string(&file_path).expect("failed to read CSV file");
        let lines: Vec<_> = contents.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.starts_with("commitment_scheme,query"))
                .count(),
            1
        );
        assert_eq!(lines[1], "dory,Filter,128,10,2,1");
        assert_eq!(lines[2], "hyperkzg,Aggregate,256,20,4,2");
    }
}
