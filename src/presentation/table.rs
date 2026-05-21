use crate::domain::check_result::CheckResult;
use comfy_table::{presets::UTF8_FULL, Attribute, Cell, CellAlignment, Color, ContentArrangement, Table};

pub struct TableRenderer {
    pub show_work_column: bool,
}

impl TableRenderer {
    pub fn render(&self, results: &[CheckResult]) -> String {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic);

        let mut header = vec![
            Cell::new("Application").add_attribute(Attribute::Bold),
            Cell::new("Version").add_attribute(Attribute::Bold),
            Cell::new("License").add_attribute(Attribute::Bold),
            Cell::new("Activation").add_attribute(Attribute::Bold),
        ];
        if self.show_work_column {
            header.push(Cell::new("Work Use").add_attribute(Attribute::Bold));
        }
        header.push(Cell::new("Notes").add_attribute(Attribute::Bold));
        table.set_header(header);

        for result in results {
            let app_cell = if result.crack_suspected {
                Cell::new(format!("⚠ {}", result.entry.name))
                    .fg(Color::Red)
                    .add_attribute(Attribute::Bold)
            } else {
                Cell::new(&result.entry.name)
            };

            let version_str = result.entry.version.as_deref().unwrap_or("—");

            let license_cell = Cell::new(result.license_model.to_string());

            let activation_cell = Cell::new(result.activation_status.to_string());

            let notes_str = result.notes.join("; ");

            let mut row = vec![app_cell, Cell::new(version_str), license_cell, activation_cell];

            if self.show_work_column {
                let work_cell = if result.work_allowed {
                    Cell::new("Yes")
                        .fg(Color::Green)
                        .set_alignment(CellAlignment::Center)
                } else {
                    Cell::new("No")
                        .fg(Color::Yellow)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Center)
                };
                row.push(work_cell);
            }

            row.push(Cell::new(notes_str));
            table.add_row(row);
        }

        table.to_string()
    }

    pub fn render_summary(&self, results: &[CheckResult], check_work: bool) -> String {
        let total = results.len();
        let cracked: Vec<_> = results.iter().filter(|r| r.crack_suspected).collect();
        let work_violations: Vec<_> = results.iter().filter(|r| !r.work_allowed).collect();

        let mut lines = vec![format!("Scanned {} application(s).", total)];

        if cracked.is_empty() {
            lines.push("No suspected crack software detected.".to_string());
        } else {
            lines.push(format!(
                "{} suspected crack(s): {}",
                cracked.len(),
                cracked.iter().map(|r| r.entry.name.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }

        if check_work && !work_violations.is_empty() {
            lines.push(format!(
                "{} app(s) not permitted for commercial use: {}",
                work_violations.len(),
                work_violations.iter().map(|r| r.entry.name.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }

        lines.join("\n")
    }
}
