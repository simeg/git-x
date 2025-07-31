use console::Color;
use git_x::core::output::{BufferedOutput, Format, ProgressIndicator, TableFormatter};

// Helper function to strip ANSI escape codes for testing
fn strip_ansi_codes(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' {
            // Found escape character, skip until 'm'
            for next_ch in chars.by_ref() {
                if next_ch == 'm' {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}

// Tests for BufferedOutput

#[test]
fn test_buffered_output_new() {
    let output = BufferedOutput::new();
    assert!(output.is_empty());
    assert_eq!(output.len(), 0);
    assert_eq!(output.content(), "");
}

#[test]
fn test_buffered_output_default() {
    let output = BufferedOutput::default();
    assert!(output.is_empty());
    assert_eq!(output.len(), 0);
    assert_eq!(output.content(), "");
}

#[test]
fn test_buffered_output_add_line() {
    let mut output = BufferedOutput::new();
    output.add_line("First line".to_string());

    assert!(!output.is_empty());
    assert_eq!(output.len(), 1);
    assert_eq!(output.content(), "First line");
}

#[test]
fn test_buffered_output_add_multiple_lines() {
    let mut output = BufferedOutput::new();
    output.add_line("First line".to_string());
    output.add_line("Second line".to_string());
    output.add_line("Third line".to_string());

    assert_eq!(output.len(), 3);
    assert_eq!(output.content(), "First line\nSecond line\nThird line");
}

#[test]
fn test_buffered_output_add_lines() {
    let mut output = BufferedOutput::new();
    let lines = vec![
        "Line 1".to_string(),
        "Line 2".to_string(),
        "Line 3".to_string(),
    ];
    output.add_lines(lines);

    assert_eq!(output.len(), 3);
    assert_eq!(output.content(), "Line 1\nLine 2\nLine 3");
}

#[test]
fn test_buffered_output_add_formatted() {
    let mut output = BufferedOutput::new();
    output.add_formatted("Formatted line".to_string());

    assert_eq!(output.len(), 1);
    assert_eq!(output.content(), "Formatted line");
}

#[test]
fn test_buffered_output_mixed_operations() {
    let mut output = BufferedOutput::new();
    output.add_line("Single line".to_string());
    output.add_lines(vec!["Batch 1".to_string(), "Batch 2".to_string()]);
    output.add_formatted("Formatted".to_string());

    assert_eq!(output.len(), 4);
    assert_eq!(output.content(), "Single line\nBatch 1\nBatch 2\nFormatted");
}

#[test]
fn test_buffered_output_display_trait() {
    let mut output = BufferedOutput::new();
    output.add_line("Display test".to_string());
    output.add_line("Second line".to_string());

    let display_output = format!("{output}");
    assert_eq!(display_output, "Display test\nSecond line");
}

#[test]
fn test_buffered_output_empty_content() {
    let output = BufferedOutput::new();
    assert_eq!(output.content(), "");
    assert_eq!(format!("{output}"), "");
}

#[test]
fn test_buffered_output_single_empty_line() {
    let mut output = BufferedOutput::new();
    output.add_line("".to_string());

    assert_eq!(output.len(), 1);
    assert_eq!(output.content(), "");
    assert!(!output.is_empty());
}

#[test]
fn test_buffered_output_lines_with_newlines() {
    let mut output = BufferedOutput::new();
    output.add_line("Line with\nembedded newline".to_string());
    output.add_line("Normal line".to_string());

    assert_eq!(output.len(), 2);
    assert_eq!(output.content(), "Line with\nembedded newline\nNormal line");
}

// Tests for Format

#[test]
fn test_format_error() {
    let result = Format::error("Something went wrong");
    let clean_result = strip_ansi_codes(&result);
    assert!(clean_result.contains("❌"));
    assert!(clean_result.contains("Something went wrong"));
}

#[test]
fn test_format_success() {
    let result = Format::success("Operation completed");
    let clean_result = strip_ansi_codes(&result);
    assert!(clean_result.contains("✅"));
    assert!(clean_result.contains("Operation completed"));
}

#[test]
fn test_format_info() {
    let result = Format::info("Information message");
    let clean_result = strip_ansi_codes(&result);
    assert!(clean_result.contains("ℹ️"));
    assert!(clean_result.contains("Information message"));
}

#[test]
fn test_format_warning() {
    let result = Format::warning("Warning message");
    let clean_result = strip_ansi_codes(&result);
    assert!(clean_result.contains("⚠️"));
    assert!(clean_result.contains("Warning message"));
}

#[test]
fn test_format_bold() {
    let result = Format::bold("Bold text");
    let clean_result = strip_ansi_codes(&result);
    assert_eq!(clean_result, "Bold text");
    // The original might contain ANSI codes for bold formatting (environment dependent)
    // In non-TTY environments, no formatting codes may be applied
    assert!(result.contains("Bold text"));
}

#[test]
fn test_format_colored_red() {
    let result = Format::colored("Red text", Color::Red);
    let clean_result = strip_ansi_codes(&result);
    assert_eq!(clean_result, "Red text");
    // The original might contain ANSI codes for color (environment dependent)
    assert!(result.contains("Red text"));
}

#[test]
fn test_format_colored_blue() {
    let result = Format::colored("Blue text", Color::Blue);
    let clean_result = strip_ansi_codes(&result);
    assert_eq!(clean_result, "Blue text");
    // The original might contain ANSI codes for color (environment dependent)
    assert!(result.contains("Blue text"));
}

#[test]
fn test_format_colored_green() {
    let result = Format::colored("Green text", Color::Green);
    let clean_result = strip_ansi_codes(&result);
    assert_eq!(clean_result, "Green text");
    // The original might contain ANSI codes for color (environment dependent)
    assert!(result.contains("Green text"));
}

#[test]
fn test_format_empty_strings() {
    assert_eq!(strip_ansi_codes(&Format::error("")), "❌ ");
    assert_eq!(strip_ansi_codes(&Format::success("")), "✅ ");
    assert_eq!(strip_ansi_codes(&Format::info("")), "ℹ️ ");
    assert_eq!(strip_ansi_codes(&Format::warning("")), "⚠️ ");
    assert_eq!(strip_ansi_codes(&Format::bold("")), "");
    assert_eq!(strip_ansi_codes(&Format::colored("", Color::Red)), "");
}

#[test]
fn test_format_special_characters() {
    let special_text = "Special: @#$%^&*(){}[]";
    let result = Format::bold(special_text);
    let clean_result = strip_ansi_codes(&result);
    assert_eq!(clean_result, special_text);
}

#[test]
fn test_format_unicode_characters() {
    let unicode_text = "Unicode: 你好 🌟 ñoël";
    let result = Format::colored(unicode_text, Color::Cyan);
    let clean_result = strip_ansi_codes(&result);
    assert_eq!(clean_result, unicode_text);
}

// Tests for TableFormatter

#[test]
fn test_table_formatter_new() {
    let headers = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
    let formatter = TableFormatter::new(headers.clone());

    // We can't directly access the headers, but we can test the structure indirectly
    let result = formatter.format();
    assert_eq!(result, "No data to display");
}

#[test]
fn test_table_formatter_empty_table() {
    let headers = vec!["Col1".to_string(), "Col2".to_string()];
    let formatter = TableFormatter::new(headers);

    let result = formatter.format();
    assert_eq!(result, "No data to display");
}

#[test]
fn test_table_formatter_single_row() {
    let headers = vec!["Name".to_string(), "Age".to_string()];
    let mut formatter = TableFormatter::new(headers);
    formatter.add_row(vec!["Alice".to_string(), "30".to_string()]);

    let result = formatter.format();
    assert!(result.contains("Name"));
    assert!(result.contains("Age"));
    assert!(result.contains("Alice"));
    assert!(result.contains("30"));
    assert!(result.contains("----")); // Separator line
}

#[test]
fn test_table_formatter_multiple_rows() {
    let headers = vec!["Name".to_string(), "Age".to_string(), "City".to_string()];
    let mut formatter = TableFormatter::new(headers);
    formatter.add_row(vec![
        "Alice".to_string(),
        "30".to_string(),
        "New York".to_string(),
    ]);
    formatter.add_row(vec![
        "Bob".to_string(),
        "25".to_string(),
        "Los Angeles".to_string(),
    ]);
    formatter.add_row(vec![
        "Charlie".to_string(),
        "35".to_string(),
        "Chicago".to_string(),
    ]);

    let result = formatter.format();

    // Check headers
    assert!(result.contains("Name"));
    assert!(result.contains("Age"));
    assert!(result.contains("City"));

    // Check data
    assert!(result.contains("Alice"));
    assert!(result.contains("Bob"));
    assert!(result.contains("Charlie"));
    assert!(result.contains("New York"));
    assert!(result.contains("Los Angeles"));
    assert!(result.contains("Chicago"));

    // Check structure
    assert!(result.contains("----")); // Separator line

    // Verify proper formatting - count lines
    let lines: Vec<&str> = result.split('\n').collect();
    assert_eq!(lines.len(), 6); // header + separator + 3 data rows + empty line at end
}

#[test]
fn test_table_formatter_column_width_adjustment() {
    let headers = vec!["Short".to_string(), "Very Long Header".to_string()];
    let mut formatter = TableFormatter::new(headers);
    formatter.add_row(vec!["A".to_string(), "X".to_string()]);
    formatter.add_row(vec!["Very Long Content".to_string(), "Y".to_string()]);

    let result = formatter.format();

    // The table should properly align columns based on the longest content
    assert!(result.contains("Very Long Content"));
    assert!(result.contains("Very Long Header"));

    // Check that spacing is consistent
    let lines: Vec<&str> = result.split('\n').collect();
    let header_line = lines[0];
    let separator_line = lines[1];

    // Headers should be properly spaced
    assert!(header_line.contains("Short"));
    assert!(header_line.contains("Very Long Header"));

    // Separator should match column widths
    assert!(separator_line.contains("-"));
}

#[test]
fn test_table_formatter_uneven_rows() {
    let headers = vec!["Col1".to_string(), "Col2".to_string(), "Col3".to_string()];
    let mut formatter = TableFormatter::new(headers);
    formatter.add_row(vec!["A".to_string(), "B".to_string()]); // Missing third column
    formatter.add_row(vec![
        "X".to_string(),
        "Y".to_string(),
        "Z".to_string(),
        "Extra".to_string(),
    ]); // Extra column

    let result = formatter.format();

    // Should handle uneven rows gracefully
    assert!(result.contains("Col1"));
    assert!(result.contains("Col2"));
    assert!(result.contains("Col3"));
    assert!(result.contains("A"));
    assert!(result.contains("B"));
    assert!(result.contains("X"));
    assert!(result.contains("Y"));
    assert!(result.contains("Z"));
}

#[test]
fn test_table_formatter_empty_cells() {
    let headers = vec!["Name".to_string(), "Value".to_string()];
    let mut formatter = TableFormatter::new(headers);
    formatter.add_row(vec!["".to_string(), "Empty name".to_string()]);
    formatter.add_row(vec!["Non-empty".to_string(), "".to_string()]);
    formatter.add_row(vec!["".to_string(), "".to_string()]);

    let result = formatter.format();

    assert!(result.contains("Name"));
    assert!(result.contains("Value"));
    assert!(result.contains("Empty name"));
    assert!(result.contains("Non-empty"));
}

#[test]
fn test_table_formatter_special_characters() {
    let headers = vec!["Symbol".to_string(), "Unicode".to_string()];
    let mut formatter = TableFormatter::new(headers);
    formatter.add_row(vec!["@#$%".to_string(), "🌟".to_string()]);
    formatter.add_row(vec!["{}[]".to_string(), "ñoël".to_string()]);

    let result = formatter.format();

    assert!(result.contains("@#$%"));
    assert!(result.contains("🌟"));
    assert!(result.contains("{}[]"));
    assert!(result.contains("ñoël"));
}

#[test]
fn test_table_formatter_single_column() {
    let headers = vec!["Only Column".to_string()];
    let mut formatter = TableFormatter::new(headers);
    formatter.add_row(vec!["Row 1".to_string()]);
    formatter.add_row(vec!["Row 2".to_string()]);

    let result = formatter.format();

    assert!(result.contains("Only Column"));
    assert!(result.contains("Row 1"));
    assert!(result.contains("Row 2"));
    assert!(result.contains("-"));
}

// Tests for ProgressIndicator

#[test]
fn test_progress_indicator_new() {
    let progress = ProgressIndicator::new("Testing".to_string());

    // We can't directly test the internal state, but we can verify the constructor works
    // by calling methods that depend on the state
    progress.finish(); // Should not panic
}

#[test]
fn test_progress_indicator_with_total() {
    let progress = ProgressIndicator::with_total("Processing".to_string(), 100);

    // Verify it can be created and finish can be called
    progress.finish(); // Should not panic
}

#[test]
fn test_progress_indicator_increment() {
    let mut progress = ProgressIndicator::new("Counting".to_string());

    // Test that increment can be called multiple times without panic
    progress.increment();
    progress.increment();
    progress.increment();

    progress.finish();
}

#[test]
fn test_progress_indicator_set_current() {
    let mut progress = ProgressIndicator::with_total("Setting".to_string(), 50);

    // Test setting various current values
    progress.set_current(10);
    progress.set_current(25);
    progress.set_current(50);

    progress.finish();
}

#[test]
fn test_progress_indicator_mixed_operations() {
    let mut progress = ProgressIndicator::with_total("Mixed".to_string(), 20);

    // Mix increment and set_current operations
    progress.increment(); // Should be 1
    progress.increment(); // Should be 2
    progress.set_current(10); // Set to 10
    progress.increment(); // Should be 11

    progress.finish();
}

#[test]
fn test_progress_indicator_zero_total() {
    let progress = ProgressIndicator::with_total("Zero".to_string(), 0);
    progress.finish(); // Should not panic even with zero total
}

#[test]
fn test_progress_indicator_large_numbers() {
    let mut progress = ProgressIndicator::with_total("Large".to_string(), 1000000);

    progress.set_current(500000);
    progress.increment();
    progress.finish();
}

#[test]
fn test_progress_indicator_unicode_message() {
    let mut progress = ProgressIndicator::new("Processing 🔄 files".to_string());
    progress.increment();
    progress.finish();
}

#[test]
fn test_progress_indicator_empty_message() {
    let mut progress = ProgressIndicator::new("".to_string());
    progress.increment();
    progress.finish();
}

#[test]
fn test_progress_indicator_long_message() {
    let long_message = "This is a very long progress message that should still work correctly without causing any issues".to_string();
    let mut progress = ProgressIndicator::with_total(long_message, 10);
    progress.set_current(5);
    progress.finish();
}

// Integration tests combining multiple components

#[test]
fn test_buffered_output_with_formatted_content() {
    let mut output = BufferedOutput::new();

    output.add_line(Format::success("Operation completed"));
    output.add_line(Format::error("Something failed"));
    output.add_line(Format::warning("Warning message"));
    output.add_line(Format::info("Information"));

    assert_eq!(output.len(), 4);

    let content = output.content();
    let clean_content = strip_ansi_codes(&content);

    assert!(clean_content.contains("✅"));
    assert!(clean_content.contains("❌"));
    assert!(clean_content.contains("⚠️"));
    assert!(clean_content.contains("ℹ️"));
    assert!(clean_content.contains("Operation completed"));
    assert!(clean_content.contains("Something failed"));
    assert!(clean_content.contains("Warning message"));
    assert!(clean_content.contains("Information"));
}

#[test]
fn test_table_in_buffered_output() {
    let mut output = BufferedOutput::new();

    let headers = vec!["Task".to_string(), "Status".to_string()];
    let mut table = TableFormatter::new(headers);
    table.add_row(vec!["Compile".to_string(), "✅".to_string()]);
    table.add_row(vec!["Test".to_string(), "⚠️".to_string()]);
    table.add_row(vec!["Deploy".to_string(), "❌".to_string()]);

    output.add_line("Build Results:".to_string());
    output.add_formatted(table.format());

    let content = output.content();
    assert!(content.contains("Build Results:"));
    assert!(content.contains("Task"));
    assert!(content.contains("Status"));
    assert!(content.contains("Compile"));
    assert!(content.contains("✅"));
    assert!(content.contains("⚠️"));
    assert!(content.contains("❌"));
}

#[test]
fn test_mixed_formatting_in_buffered_output() {
    let mut output = BufferedOutput::new();

    output.add_line(Format::bold("REPORT"));
    output.add_line("".to_string()); // Empty line
    output.add_line(Format::info("Analysis complete"));
    output.add_line(Format::colored("Files processed: 42", Color::Green));
    output.add_line(Format::warning("2 warnings found"));

    let content = output.content();
    let clean_content = strip_ansi_codes(&content);

    assert!(clean_content.contains("REPORT"));
    assert!(clean_content.contains("ℹ️"));
    assert!(clean_content.contains("Analysis complete"));
    assert!(clean_content.contains("Files processed: 42"));
    assert!(clean_content.contains("⚠️"));
    assert!(clean_content.contains("2 warnings found"));

    // Verify that both content forms contain the expected text
    // ANSI codes may or may not be present depending on environment
    assert!(content.contains("REPORT"));
    assert!(content.contains("Analysis complete"));
    assert!(content.contains("Files processed: 42"));
    assert!(content.contains("2 warnings found"));
}

#[test]
fn test_buffered_output_flush_methods() {
    let mut output = BufferedOutput::new();
    output.add_line("Test output to stdout".to_string());
    output.add_line("Another line".to_string());

    // Test flush to stdout (should not panic)
    output.flush();

    // Test flush to stderr (should not panic)
    output.flush_err();

    // Content should still be accessible after flushing
    let content = output.content();
    assert!(content.contains("Test output to stdout"));
    assert!(content.contains("Another line"));
}

#[test]
fn test_buffered_output_add_formatted_method() {
    let mut output = BufferedOutput::new();

    // Test add_formatted method specifically
    output.add_formatted(Format::bold("Formatted Header"));
    output.add_formatted(Format::info("Some info"));
    output.add_formatted("Plain text".to_string());

    let content = output.content();
    let clean_content = strip_ansi_codes(&content);

    assert!(clean_content.contains("Formatted Header"));
    assert!(clean_content.contains("ℹ️"));
    assert!(clean_content.contains("Some info"));
    assert!(clean_content.contains("Plain text"));
}

#[test]
fn test_buffered_output_add_lines_method() {
    let mut output = BufferedOutput::new();

    // Test add_lines method with multiple lines at once
    let lines = vec![
        "First line".to_string(),
        "Second line".to_string(),
        Format::warning("Warning line"),
        "Fourth line".to_string(),
    ];

    output.add_lines(lines);

    let content = output.content();
    let clean_content = strip_ansi_codes(&content);

    assert!(clean_content.contains("First line"));
    assert!(clean_content.contains("Second line"));
    assert!(clean_content.contains("⚠️"));
    assert!(clean_content.contains("Warning line"));
    assert!(clean_content.contains("Fourth line"));

    // Test that length is correct
    assert_eq!(output.len(), 4);
    assert!(!output.is_empty());
}

#[test]
fn test_progress_indicator_edge_cases() {
    // Test ProgressIndicator with various edge cases

    // Test with zero total
    let mut progress = ProgressIndicator::with_total("Processing".to_string(), 0);
    progress.set_current(0);
    progress.finish(); // Should not panic

    // Test setting current beyond total
    let mut progress = ProgressIndicator::with_total("Processing".to_string(), 5);
    progress.set_current(10); // Should handle gracefully
    progress.finish();

    // Test multiple increments
    let mut progress = ProgressIndicator::with_total("Processing".to_string(), 3);
    progress.increment();
    progress.increment();
    progress.increment();
    progress.increment(); // One beyond total
    progress.finish();
}
