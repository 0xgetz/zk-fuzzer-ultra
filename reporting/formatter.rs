// Report formatter for multiple output formats
// Supports JSON, CSV, Markdown, and HTML output

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::io::{self, Write as IoWrite};
use std::path::Path;

use super::bug_reporter::{BugReport, Severity, TestInput};

/// Output format for report generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Json,
    Csv,
    Markdown,
    Html,
    Text,
}

impl OutputFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(OutputFormat::Json),
            "csv" => Some(OutputFormat::Csv),
            "md" | "markdown" => Some(OutputFormat::Markdown),
            "html" => Some(OutputFormat::Html),
            "txt" | "text" => Some(OutputFormat::Text),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::Csv => "csv",
            OutputFormat::Markdown => "md",
            OutputFormat::Html => "html",
            OutputFormat::Text => "txt",
        }
    }
}

/// Summary statistics for a fuzzing campaign
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzingSummary {
    pub total_inputs_generated: u64,
    pub total_inputs_tested: u64,
    pub unique_bugs_found: u64,
    pub branch_coverage_percent: f64,
    pub constraint_coverage_percent: f64,
    pub total_execution_time_seconds: f64,
    pub inputs_per_second: f64,
    pub crash_rate: f64,
    pub timeout_rate: f64,
    pub start_time: String,
    pub end_time: String,
}

/// Report formatter for generating outputs in multiple formats
pub struct ReportFormatter {
    format: OutputFormat,
}

impl ReportFormatter {
    pub fn new(format: OutputFormat) -> Self {
        ReportFormatter { format }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        OutputFormat::from_extension(ext).map(ReportFormatter::new)
    }

    /// Format a single bug report
    pub fn format_bug_report(&self, report: &BugReport) -> io::Result<String> {
        match self.format {
            OutputFormat::Json => self.format_bug_report_json(report),
            OutputFormat::Csv => self.format_bug_report_csv(report),
            OutputFormat::Markdown => self.format_bug_report_markdown(report),
            OutputFormat::Html => self.format_bug_report_html(report),
            OutputFormat::Text => self.format_bug_report_text(report),
        }
    }

    /// Format multiple bug reports
    pub fn format_bug_reports(&self, reports: &[BugReport]) -> io::Result<String> {
        match self.format {
            OutputFormat::Json => self.format_bug_reports_json(reports),
            OutputFormat::Csv => self.format_bug_reports_csv(reports),
            OutputFormat::Markdown => self.format_bug_reports_markdown(reports),
            OutputFormat::Html => self.format_bug_reports_html(reports),
            OutputFormat::Text => self.format_bug_reports_text(reports),
        }
    }

    /// Format fuzzing summary
    pub fn format_summary(&self, summary: &FuzzingSummary) -> io::Result<String> {
        match self.format {
            OutputFormat::Json => self.format_summary_json(summary),
            OutputFormat::Csv => self.format_summary_csv(summary),
            OutputFormat::Markdown => self.format_summary_markdown(summary),
            OutputFormat::Html => self.format_summary_html(summary),
            OutputFormat::Text => self.format_summary_text(summary),
        }
    }

    /// Format complete campaign report (summary + bugs)
    pub fn format_campaign_report(
        &self,
        summary: &FuzzingSummary,
        reports: &[BugReport],
    ) -> io::Result<String> {
        match self.format {
            OutputFormat::Json => self.format_campaign_report_json(summary, reports),
            OutputFormat::Csv => self.format_campaign_report_csv(summary, reports),
            OutputFormat::Markdown => self.format_campaign_report_markdown(summary, reports),
            OutputFormat::Html => self.format_campaign_report_html(summary, reports),
            OutputFormat::Text => self.format_campaign_report_text(summary, reports),
        }
    }

    // ==================== JSON Formatting ====================

    fn format_bug_report_json(&self, report: &BugReport) -> io::Result<String> {
        serde_json::to_string_pretty(report)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn format_bug_reports_json(&self, reports: &[BugReport]) -> io::Result<String> {
        serde_json::to_string_pretty(reports)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn format_summary_json(&self, summary: &FuzzingSummary) -> io::Result<String> {
        serde_json::to_string_pretty(summary)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn format_campaign_report_json(
        &self,
        summary: &FuzzingSummary,
        reports: &[BugReport],
    ) -> io::Result<String> {
        #[derive(Serialize)]
        struct CampaignReport<'a> {
            summary: &'a FuzzingSummary,
            bugs: &'a [BugReport],
        }

        let report = CampaignReport { summary, bugs: reports };
        serde_json::to_string_pretty(&report)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    // ==================== CSV Formatting ====================

    fn format_bug_report_csv(&self, report: &BugReport) -> io::Result<String> {
        let mut output = String::new();

        // Header
        writeln!(
            output,
            "ID,Title,Severity,Circuit,Constraint,Branch Coverage,Constraint Satisfaction,Crash Count,Timeout Count,Found At"
        )?;

        // Row
        writeln!(
            output,
            "{},{},{},{},{:.2}%,{:.2}%,{},{},{}",
            report.id,
            self.escape_csv(&report.title),
            report.severity,
            self.escape_csv(&report.circuit_name),
            report.metrics.branch_coverage * 100.0,
            report.metrics.constraint_satisfaction * 100.0,
            report.metrics.crash_count,
            report.metrics.timeout_count,
            report.found_at
        )?;

        Ok(output)
    }

    fn format_bug_reports_csv(&self, reports: &[BugReport]) -> io::Result<String> {
        let mut output = String::new();

        // Header
        writeln!(
            output,
            "ID,Title,Severity,Circuit,Constraint,Branch Coverage,Constraint Satisfaction,Crash Count,Timeout Count,Found At"
        )?;

        // Rows
        for report in reports {
            writeln!(
                output,
                "{},{},{},{},{:.2}%,{:.2}%,{},{},{}",
                report.id,
                self.escape_csv(&report.title),
                report.severity,
                self.escape_csv(&report.circuit_name),
                report.metrics.branch_coverage * 100.0,
                report.metrics.constraint_satisfaction * 100.0,
                report.metrics.crash_count,
                report.metrics.timeout_count,
                report.found_at
            )?;
        }

        Ok(output)
    }

    fn format_summary_csv(&self, summary: &FuzzingSummary) -> io::Result<String> {
        let mut output = String::new();

        writeln!(
            output,
            "Metric,Value"
        )?;
        writeln!(output, "Total Inputs Generated,{}", summary.total_inputs_generated)?;
        writeln!(output, "Total Inputs Tested,{}", summary.total_inputs_tested)?;
        writeln!(output, "Unique Bugs Found,{}", summary.unique_bugs_found)?;
        writeln!(output, "Branch Coverage,{}%", summary.branch_coverage_percent)?;
        writeln!(output, "Constraint Coverage,{}%", summary.constraint_coverage_percent)?;
        writeln!(output, "Execution Time (s),{}", summary.total_execution_time_seconds)?;
        writeln!(output, "Inputs/Second,{}", summary.inputs_per_second)?;
        writeln!(output, "Crash Rate,{}%", summary.crash_rate)?;
        writeln!(output, "Timeout Rate,{}%", summary.timeout_rate)?;
        writeln!(output, "Start Time,{}", summary.start_time)?;
        writeln!(output, "End Time,{}", summary.end_time)?;

        Ok(output)
    }

    fn format_campaign_report_csv(
        &self,
        summary: &FuzzingSummary,
        reports: &[BugReport],
    ) -> io::Result<String> {
        // CSV doesn't handle multiple sections well, so we output summary and bugs separately
        let summary_csv = self.format_summary_csv(summary)?;
        let bugs_csv = self.format_bug_reports_csv(reports)?;
        Ok(format!("=== SUMMARY ===
{}

=== BUGS ===
{}", summary_csv, bugs_csv))
    }

    fn escape_csv(&self, value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!(""{}"", value.replace('"', """"))
        } else {
            value.to_string()
        }
    }

    // ==================== Markdown Formatting ====================

    fn format_bug_report_markdown(&self, report: &BugReport) -> io::Result<String> {
        let mut output = String::new();

        writeln!(output, "# Bug Report: {}", report.title)?;
        writeln!(output)?;
        writeln!(output, "**ID:** {}", report.id)?;
        writeln!(output, "**Severity:** {:?}", report.severity)?;
        writeln!(output, "**Circuit:** {}", report.circuit_name)?;
        writeln!(output, "**Found at:** {}", report.found_at)?;
        writeln!(output)?;
        writeln!(output, "## Description")?;
        writeln!(output, "{}", report.description)?;
        writeln!(output)?;
        writeln!(output, "## Metrics")?;
        writeln!(output, "- Branch Coverage: {:.2}%", report.metrics.branch_coverage * 100.0)?;
        writeln!(output, "- Constraint Satisfaction: {:.2}%", report.metrics.constraint_satisfaction * 100.0)?;
        writeln!(output, "- Crash Count: {}", report.metrics.crash_count)?;
        writeln!(output, "- Timeout Count: {}", report.metrics.timeout_count)?;
        writeln!(output)?;
        writeln!(output, "## Failing Inputs")?;
        for (i, input) in report.failing_inputs.iter().enumerate() {
            writeln!(output, "### Input {}", i + 1)?;
            writeln!(output, "```")?;
            writeln!(output, "{:?}", input.raw_bytes)?;
            writeln!(output, "```")?;
            writeln!(output, "**Type:** {:?}", input.input_type)?;
            writeln!(output, "**Result:** {:?}", input.result)?;
            writeln!(output)?;
        }
        writeln!(output, "## Stack Trace")?;
        writeln!(output, "```")?;
        if let Some(trace) = &report.stack_trace {
            writeln!(output, "{}", trace)?;
        }
        writeln!(output, "```")?;

        Ok(output)
    }

    fn format_bug_reports_markdown(&self, reports: &[BugReport]) -> io::Result<String> {
        let mut output = String::new();

        writeln!(output, "# Bug Reports")?;
        writeln!(output)?;
        writeln!(output, "Found **{}** bugs.", reports.len())?;
        writeln!(output)?;

        for report in reports {
            output.push_str(&self.format_bug_report_markdown(report)?);
            writeln!(output)?;
            writeln!(output, "---")?;
            writeln!(output)?;
        }

        Ok(output)
    }

    fn format_summary_markdown(&self, summary: &FuzzingSummary) -> io::Result<String> {
        let mut output = String::new();

        writeln!(output, "# Fuzzing Campaign Summary")?;
        writeln!(output)?;
        writeln!(output, "## Overview")?;
        writeln!(output, "| Metric | Value |")?;
        writeln!(output, "|--------|-------|")?;
        writeln!(output, "| Total Inputs Generated | {} |", summary.total_inputs_generated)?;
        writeln!(output, "| Total Inputs Tested | {} |", summary.total_inputs_tested)?;
        writeln!(output, "| Unique Bugs Found | {} |", summary.unique_bugs_found)?;
        writeln!(output, "| Branch Coverage | {:.2}% |", summary.branch_coverage_percent)?;
        writeln!(output, "| Constraint Coverage | {:.2}% |", summary.constraint_coverage_percent)?;
        writeln!(output, "| Execution Time | {:.2}s |", summary.total_execution_time_seconds)?;
        writeln!(output, "| Inputs/Second | {:.2} |", summary.inputs_per_second)?;
        writeln!(output, "| Crash Rate | {:.2}% |", summary.crash_rate)?;
        writeln!(output, "| Timeout Rate | {:.2}% |", summary.timeout_rate)?;
        writeln!(output, "| Start Time | {} |", summary.start_time)?;
        writeln!(output, "| End Time | {} |", summary.end_time)?;

        Ok(output)
    }

    fn format_campaign_report_markdown(
        &self,
        summary: &FuzzingSummary,
        reports: &[BugReport],
    ) -> io::Result<String> {
        let summary_md = self.format_summary_markdown(summary)?;
        let bugs_md = self.format_bug_reports_markdown(reports)?;
        Ok(format!("{}

{}
", summary_md, bugs_md))
    }

    // ==================== HTML Formatting ====================

    fn format_bug_report_html(&self, report: &BugReport) -> io::Result<String> {
        let mut output = String::new();

        writeln!(output, "<!DOCTYPE html>")?;
        writeln!(output, "<html>")?;
        writeln!(output, "<head>")?;
        writeln!(output, "  <title>Bug Report: {}</title>", self.escape_html(&report.title))?;
        writeln!(output, "  <style>")?;
        writeln!(output, "    body {{ font-family: Arial, sans-serif; margin: 20px; }}")?;
        writeln!(output, "    h1 {{ color: #d32f2f; }}")?;
        writeln!(output, "    .severity-critical {{ color: #d32f2f; font-weight: bold; }}")?;
        writeln!(output, "    .severity-high {{ color: #f57c00; font-weight: bold; }}")?;
        writeln!(output, "    .severity-medium {{ color: #fbc02d; }}")?;
        writeln!(output, "    .severity-low {{ color: #388e3c; }}")?;
        writeln!(output, "    .metric {{ margin: 5px 0; }}")?;
        writeln!(output, "    pre {{ background: #f5f5f5; padding: 10px; border-radius: 4px; }}")?;
        writeln!(output, "    table {{ border-collapse: collapse; width: 100%; }}")?;
        writeln!(output, "    th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}")?;
        writeln!(output, "    th {{ background: #4caf50; color: white; }}")?;
        writeln!(output, "  </style>")?;
        writeln!(output, "</head>")?;
        writeln!(output, "<body>")?;
        writeln!(output, "  <h1>Bug Report: {}</h1>", self.escape_html(&report.title))?;
        writeln!(output, "  <p><strong>ID:</strong> {}</p>", report.id)?;
        writeln!(
            output,
            "  <p><strong>Severity:</strong> <span class=\"severity-{:?}\">{:?}</span></p>",
            report.severity.to_string().to_lowercase(),
            report.severity
        )?;
        writeln!(output, "  <p><strong>Circuit:</strong> {}</p>", self.escape_html(&report.circuit_name))?;
        writeln!(output, "  <p><strong>Found at:</strong> {}</p>", report.found_at)?;

        writeln!(output, "  <h2>Description</h2>")?;
        writeln!(output, "  <p>{}</p>", self.escape_html(&report.description))?;

        writeln!(output, "  <h2>Metrics</h2>")?;
        writeln!(output, "  <table>")?;
        writeln!(output, "    <tr><th>Metric</th><th>Value</th></tr>")?;
        writeln!(
            output,
            "    <tr><td>Branch Coverage</td><td>{:.2}%</td></tr>",
            report.metrics.branch_coverage * 100.0
        )?;
        writeln!(
            output,
            "    <tr><td>Constraint Satisfaction</td><td>{:.2}%</td></tr>",
            report.metrics.constraint_satisfaction * 100.0
        )?;
        writeln!(
            output,
            "    <tr><td>Crash Count</td><td>{}</td></tr>",
            report.metrics.crash_count
        )?;
        writeln!(
            output,
            "    <tr><td>Timeout Count</td><td>{}</td></tr>",
            report.metrics.timeout_count
        )?;
        writeln!(output, "  </table>")?;

        writeln!(output, "  <h2>Failing Inputs</h2>")?;
        for (i, input) in report.failing_inputs.iter().enumerate() {
            writeln!(output, "    <h3>Input {}</h3>", i + 1)?;
            writeln!(output, "    <pre>{:?}</pre>", input.raw_bytes)?;
            writeln!(output, "    <p><strong>Type:</strong> {:?}</p>", input.input_type)?;
            writeln!(output, "    <p><strong>Result:</strong> {:?}</p>", input.result)?;
        }

        writeln!(output, "  <h2>Stack Trace</h2>")?;
        writeln!(output, "  <pre>")?;
        if let Some(trace) = &report.stack_trace {
            writeln!(output, "{}", self.escape_html(trace))?;
        }
        writeln!(output, "  </pre>")?;

        writeln!(output, "</body>")?;
        writeln!(output, "</html>")?;

        Ok(output)
    }

    fn format_bug_reports_html(&self, reports: &[BugReport]) -> io::Result<String> {
        let mut output = String::new();

        writeln!(output, "<!DOCTYPE html>")?;
        writeln!(output, "<html>")?;
        writeln!(output, "<head>")?;
        writeln!(output, "  <title>Bug Reports</title>")?;
        writeln!(output, "  <style>")?;
        writeln!(output, "    body {{ font-family: Arial, sans-serif; margin: 20px; }}")?;
        writeln!(output, "    h1 {{ color: #d32f2f; }}")?;
        writeln!(output, "    .bug-card {{ border: 1px solid #ddd; padding: 15px; margin: 10px 0; border-radius: 5px; }}")?;
        writeln!(output, "    .severity-critical {{ color: #d32f2f; font-weight: bold; }}")?;
        writeln!(output, "    .severity-high {{ color: #f57c00; font-weight: bold; }}")?;
        writeln!(output, "    .severity-medium {{ color: #fbc02d; }}")?;
        writeln!(output, "    .severity-low {{ color: #388e3c; }}")?;
        writeln!(output, "  </style>")?;
        writeln!(output, "</head>")?;
        writeln!(output, "<body>")?;
        writeln!(output, "  <h1>Bug Reports</h1>")?;
        writeln!(output, "  <p>Found <strong>{}</strong> bugs.</p>", reports.len())?;

        for report in reports {
            writeln!(output, "  <div class=\"bug-card\">")?;
            writeln!(output, "    <h2>{}</h2>", self.escape_html(&report.title))?;
            writeln!(output, "    <p><strong>ID:</strong> {}</p>", report.id)?;
            writeln!(
                output,
                "    <p><strong>Severity:</strong> <span class=\"severity-{:?}\">{:?}</span></p>",
                report.severity.to_string().to_lowercase(),
                report.severity
            )?;
            writeln!(output, "    <p><strong>Circuit:</strong> {}</p>", self.escape_html(&report.circuit_name))?;
            writeln!(output, "    <p><strong>Found at:</strong> {}</p>", report.found_at)?;
            writeln!(output, "    <p>{}</p>", self.escape_html(&report.description))?;
            writeln!(output, "  </div>")?;
        }

        writeln!(output, "</body>")?;
        writeln!(output, "</html>")?;

        Ok(output)
    }

    fn format_summary_html(&self, summary: &FuzzingSummary) -> io::Result<String> {
        let mut output = String::new();

        writeln!(output, "<!DOCTYPE html>")?;
        writeln!(output, "<html>")?;
        writeln!(output, "<head>")?;
        writeln!(output, "  <title>Fuzzing Campaign Summary</title>")?;
        writeln!(output, "  <style>")?;
        writeln!(output, "    body {{ font-family: Arial, sans-serif; margin: 20px; }}")?;
        writeln!(output, "    h1 {{ color: #2196f3; }}")?;
        writeln!(output, "    table {{ border-collapse: collapse; width: 100%; max-width: 600px; }}")?;
        writeln!(output, "    th, td {{ border: 1px solid #ddd; padding: 10px; text-align: left; }}")?;
        writeln!(output, "    th {{ background: #2196f3; color: white; }}")?;
        writeln!(output, "    tr:nth-child(even) {{ background: #f9f9f9; }}")?;
        writeln!(output, "  </style>")?;
        writeln!(output, "</head>")?;
        writeln!(output, "<body>")?;
        writeln!(output, "  <h1>Fuzzing Campaign Summary</h1>")?;
        writeln!(output, "  <table>")?;
        writeln!(output, "    <tr><th>Metric</th><th>Value</th></tr>")?;
        writeln!(output, "    <tr><td>Total Inputs Generated</td><td>{}</td></tr>", summary.total_inputs_generated)?;
        writeln!(output, "    <tr><td>Total Inputs Tested</td><td>{}</td></tr>", summary.total_inputs_tested)?;
        writeln!(output, "    <tr><td>Unique Bugs Found</td><td>{}</td></tr>", summary.unique_bugs_found)?;
        writeln!(output, "    <tr><td>Branch Coverage</td><td>{:.2}%</td></tr>", summary.branch_coverage_percent)?;
        writeln!(output, "    <tr><td>Constraint Coverage</td><td>{:.2}%</td></tr>", summary.constraint_coverage_percent)?;
        writeln!(output, "    <tr><td>Execution Time</td><td>{:.2}s</td></tr>", summary.total_execution_time_seconds)?;
        writeln!(output, "    <tr><td>Inputs/Second</td><td>{:.2}</td></tr>", summary.inputs_per_second)?;
        writeln!(output, "    <tr><td>Crash Rate</td><td>{:.2}%</td></tr>", summary.crash_rate)?;
        writeln!(output, "    <tr><td>Timeout Rate</td><td>{:.2}%</td></tr>", summary.timeout_rate)?;
        writeln!(output, "    <tr><td>Start Time</td><td>{}</td></tr>", summary.start_time)?;
        writeln!(output, "    <tr><td>End Time</td><td>{}</td></tr>", summary.end_time)?;
        writeln!(output, "  </table>")?;
        writeln!(output, "</body>")?;
        writeln!(output, "</html>")?;

        Ok(output)
    }

    fn format_campaign_report_html(
        &self,
        summary: &FuzzingSummary,
        reports: &[BugReport],
    ) -> io::Result<String> {
        let summary_html = self.format_summary_html(summary)?;
        let bugs_html = self.format_bug_reports_html(reports)?;

        // Combine into a single HTML document with navigation
        let mut output = String::new();
        writeln!(output, "<!DOCTYPE html>")?;
        writeln!(output, "<html>")?;
        writeln!(output, "<head>")?;
        writeln!(output, "  <title>Campaign Report</title>")?;
        writeln!(output, "  <style>")?;
        writeln!(output, "    body {{ font-family: Arial, sans-serif; margin: 20px; }}")?;
        writeln!(output, "    nav {{ background: #f5f5f5; padding: 10px; margin-bottom: 20px; }}")?;
        writeln!(output, "    nav a {{ margin-right: 15px; color: #2196f3; text-decoration: none; }}")?;
        writeln!(output, "    h1 {{ color: #2196f3; }}")?;
        writeln!(output, "    table {{ border-collapse: collapse; width: 100%; max-width: 600px; }}")?;
        writeln!(output, "    th, td {{ border: 1px solid #ddd; padding: 10px; text-align: left; }}")?;
        writeln!(output, "    th {{ background: #2196f3; color: white; }}")?;
        writeln!(output, "    .bug-card {{ border: 1px solid #ddd; padding: 15px; margin: 10px 0; border-radius: 5px; }}")?;
        writeln!(output, "    .severity-critical {{ color: #d32f2f; font-weight: bold; }}")?;
        writeln!(output, "    .severity-high {{ color: #f57c00; font-weight: bold; }}")?;
        writeln!(output, "    .severity-medium {{ color: #fbc02d; }}")?;
        writeln!(output, "    .severity-low {{ color: #388e3c; }}")?;
        writeln!(output, "  </style>")?;
        writeln!(output, "</head>")?;
        writeln!(output, "<body>")?;
        writeln!(output, "  <nav>")?;
        writeln!(output, "    <a href=\"#summary\">Summary</a>")?;
        writeln!(output, "    <a href=\"#bugs\">Bugs ({})</a>", reports.len())?;
        writeln!(output, "  </nav>")?;

        // Summary section
        writeln!(output, "  <section id=\"summary\">")?;
        writeln!(output, "    <h1>Fuzzing Campaign Summary</h1>")?;
        writeln!(output, "    <table>")?;
        writeln!(output, "      <tr><th>Metric</th><th>Value</th></tr>")?;
        writeln!(output, "      <tr><td>Total Inputs Generated</td><td>{}</td></tr>", summary.total_inputs_generated)?;
        writeln!(output, "      <tr><td>Total Inputs Tested</td><td>{}</td></tr>", summary.total_inputs_tested)?;
        writeln!(output, "      <tr><td>Unique Bugs Found</td><td>{}</td></tr>", summary.unique_bugs_found)?;
        writeln!(output, "      <tr><td>Branch Coverage</td><td>{:.2}%</td></tr>", summary.branch_coverage_percent)?;
        writeln!(output, "      <tr><td>Constraint Coverage</td><td>{:.2}%</td></tr>", summary.constraint_coverage_percent)?;
        writeln!(output, "      <tr><td>Execution Time</td><td>{:.2}s</td></tr>", summary.total_execution_time_seconds)?;
        writeln!(output, "      <tr><td>Inputs/Second</td><td>{:.2}</td></tr>", summary.inputs_per_second)?;
        writeln!(output, "      <tr><td>Crash Rate</td><td>{:.2}%</td></tr>", summary.crash_rate)?;
        writeln!(output, "      <tr><td>Timeout Rate</td><td>{:.2}%</td></tr>", summary.timeout_rate)?;
        writeln!(output, "      <tr><td>Start Time</td><td>{}</td></tr>", summary.start_time)?;
        writeln!(output, "      <tr><td>End Time</td><td>{}</td></tr>", summary.end_time)?;
        writeln!(output, "    </table>")?;
        writeln!(output, "  </section>")?;

        // Bugs section
        writeln!(output, "  <section id=\"bugs\">")?;
        writeln!(output, "    <h1>Bug Reports</h1>")?;
        for report in reports {
            writeln!(output, "    <div class=\"bug-card\">")?;
            writeln!(output, "      <h2>{}</h2>", self.escape_html(&report.title))?;
            writeln!(output, "      <p><strong>ID:</strong> {}</p>", report.id)?;
            writeln!(
                output,
                "      <p><strong>Severity:</strong> <span class=\"severity-{:?}\">{:?}</span></p>",
                report.severity.to_string().to_lowercase(),
                report.severity
            )?;
            writeln!(output, "      <p><strong>Circuit:</strong> {}</p>", self.escape_html(&report.circuit_name))?;
            writeln!(output, "      <p><strong>Found at:</strong> {}</p>", report.found_at)?;
            writeln!(output, "      <p>{}</p>", self.escape_html(&report.description))?;
            writeln!(output, "    </div>")?;
        }
        writeln!(output, "  </section>")?;

        writeln!(output, "</body>")?;
        writeln!(output, "</html>")?;

        Ok(output)
    }

    fn escape_html(&self, value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace("'", "&#39;")
    }

    // ==================== Text Formatting ====================

    fn format_bug_report_text(&self, report: &BugReport) -> io::Result<String> {
        let mut output = String::new();

        writeln!(output, "Bug Report: {}", report.title)?;
        writeln!(output, "{}", "=".repeat(60))?;
        writeln!(output, "ID: {}", report.id)?;
        writeln!(output, "Severity: {:?}", report.severity)?;
        writeln!(output, "Circuit: {}", report.circuit_name)?;
        writeln!(output, "Found at: {}", report.found_at)?;
        writeln!(output)?;
        writeln!(output, "Description:")?;
        writeln!(output, "{}", report.description)?;
        writeln!(output)?;
        writeln!(output, "Metrics:")?;
        writeln!(output, "  Branch Coverage: {:.2}%", report.metrics.branch_coverage * 100.0)?;
        writeln!(output, "  Constraint Satisfaction: {:.2}%", report.metrics.constraint_satisfaction * 100.0)?;
        writeln!(output, "  Crash Count: {}", report.metrics.crash_count)?;
        writeln!(output, "  Timeout Count: {}", report.metrics.timeout_count)?;
        writeln!(output)?;
        writeln!(output, "Failing Inputs:")?;
        for (i, input) in report.failing_inputs.iter().enumerate() {
            writeln!(output, "  Input {}:", i + 1)?;
            writeln!(output, "    Bytes: {:?}", input.raw_bytes)?;
            writeln!(output, "    Type: {:?}", input.input_type)?;
            writeln!(output, "    Result: {:?}", input.result)?;
        }
        writeln!(output)?;
        writeln!(output, "Stack Trace:")?;
        if let Some(trace) = &report.stack_trace {
            writeln!(output, "{}", trace)?;
        }

        Ok(output)
    }

    fn format_bug_reports_text(&self, reports: &[BugReport]) -> io::Result<String> {
        let mut output = String::new();

        writeln!(output, "Bug Reports")?;
        writeln!(output, "{}", "=".repeat(60))?;
        writeln!(output, "Found {} bugs.", reports.len())?;
        writeln!(output)?;

        for report in reports {
            output.push_str(&self.format_bug_report_text(report)?);
            writeln!(output)?;
            writeln!(output, "{}", "-".repeat(60))?;
            writeln!(output)?;
        }

        Ok(output)
    }

    fn format_summary_text(&self, summary: &FuzzingSummary) -> io::Result<String> {
        let mut output = String::new();

        writeln!(output, "Fuzzing Campaign Summary")?;
        writeln!(output, "{}", "=".repeat(60))?;
        writeln!(output)?;
        writeln!(output, "Total Inputs Generated: {}", summary.total_inputs_generated)?;
        writeln!(output, "Total Inputs Tested: {}", summary.total_inputs_tested)?;
        writeln!(output, "Unique Bugs Found: {}", summary.unique_bugs_found)?;
        writeln!(output, "Branch Coverage: {:.2}%", summary.branch_coverage_percent)?;
        writeln!(output, "Constraint Coverage: {:.2}%", summary.constraint_coverage_percent)?;
        writeln!(output, "Execution Time: {:.2}s", summary.total_execution_time_seconds)?;
        writeln!(output, "Inputs/Second: {:.2}", summary.inputs_per_second)?;
        writeln!(output, "Crash Rate: {:.2}%", summary.crash_rate)?;
        writeln!(output, "Timeout Rate: {:.2}%", summary.timeout_rate)?;
        writeln!(output, "Start Time: {}", summary.start_time)?;
        writeln!(output, "End Time: {}", summary.end_time)?;

        Ok(output)
    }

    fn format_campaign_report_text(
        &self,
        summary: &FuzzingSummary,
        reports: &[BugReport],
    ) -> io::Result<String> {
        let summary_text = self.format_summary_text(summary)?;
        let bugs_text = self.format_bug_reports_text(reports)?;
        Ok(format!("{}

{}
", summary_text, bugs_text))
    }

    /// Save formatted output to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, content: &str, path: P) -> io::Result<()> {
        fs::write(path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn create_test_bug_report() -> BugReport {
        BugReport {
            id: "BUG-001".to_string(),
            title: "Test Bug".to_string(),
            description: "This is a test bug".to_string(),
            severity: Severity::High,
            circuit_name: "test_circuit".to_string(),
            failing_inputs: vec![
                TestInput {
                    raw_bytes: vec![1, 2, 3, 4],
                    input_type: super::super::bug_reporter::InputType::Random,
                    result: super::super::bug_reporter::TestResult::Crash,
                },
            ],
            stack_trace: Some("stack trace here".to_string()),
            metrics: super::super::bug_reporter::BugMetrics {
                branch_coverage: 0.75,
                constraint_satisfaction: 0.60,
                crash_count: 5,
                timeout_count: 2,
            },
            found_at: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap().to_rfc3339(),
        }
    }

    fn create_test_summary() -> FuzzingSummary {
        FuzzingSummary {
            total_inputs_generated: 10000,
            total_inputs_tested: 9500,
            unique_bugs_found: 3,
            branch_coverage_percent: 85.5,
            constraint_coverage_percent: 72.3,
            total_execution_time_seconds: 3600.0,
            inputs_per_second: 2.64,
            crash_rate: 0.5,
            timeout_rate: 1.2,
            start_time: Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap().to_rfc3339(),
            end_time: Utc.with_ymd_and_hms(2024, 1, 15, 11, 0, 0).unwrap().to_rfc3339(),
        }
    }

    #[test]
    fn test_output_format_from_extension() {
        assert_eq!(OutputFormat::from_extension("json"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_extension("csv"), Some(OutputFormat::Csv));
        assert_eq!(OutputFormat::from_extension("md"), Some(OutputFormat::Markdown));
        assert_eq!(OutputFormat::from_extension("html"), Some(OutputFormat::Html));
        assert_eq!(OutputFormat::from_extension("txt"), Some(OutputFormat::Text));
        assert_eq!(OutputFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_format_bug_report_json() {
        let formatter = ReportFormatter::new(OutputFormat::Json);
        let report = create_test_bug_report();
        let result = formatter.format_bug_report(&report).unwrap();
        assert!(result.contains(""id": "BUG-001""));
        assert!(result.contains(""title": "Test Bug""));
    }

    #[test]
    fn test_format_bug_report_csv() {
        let formatter = ReportFormatter::new(OutputFormat::Csv);
        let report = create_test_bug_report();
        let result = formatter.format_bug_report(&report).unwrap();
        assert!(result.contains("ID,Title,Severity"));
        assert!(result.contains("BUG-001"));
    }

    #[test]
    fn test_format_bug_report_markdown() {
        let formatter = ReportFormatter::new(OutputFormat::Markdown);
        let report = create_test_bug_report();
        let result = formatter.format_bug_report(&report).unwrap();
        assert!(result.contains("# Bug Report: Test Bug"));
        assert!(result.contains("**Severity:** High"));
    }

    #[test]
    fn test_format_bug_report_html() {
        let formatter = ReportFormatter::new(OutputFormat::Html);
        let report = create_test_bug_report();
        let result = formatter.format_bug_report(&report).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<h1>Bug Report: Test Bug</h1>"));
    }

    #[test]
    fn test_format_bug_report_text() {
        let formatter = ReportFormatter::new(OutputFormat::Text);
        let report = create_test_bug_report();
        let result = formatter.format_bug_report(&report).unwrap();
        assert!(result.contains("Bug Report: Test Bug"));
        assert!(result.contains("Severity: High"));
    }

    #[test]
    fn test_format_summary_json() {
        let formatter = ReportFormatter::new(OutputFormat::Json);
        let summary = create_test_summary();
        let result = formatter.format_summary(&summary).unwrap();
        assert!(result.contains(""total_inputs_generated": 10000"));
        assert!(result.contains(""unique_bugs_found": 3"));
    }

    #[test]
    fn test_format_campaign_report_json() {
        let formatter = ReportFormatter::new(OutputFormat::Json);
        let summary = create_test_summary();
        let reports = vec![create_test_bug_report()];
        let result = formatter.format_campaign_report(&summary, &reports).unwrap();
        assert!(result.contains(""summary""));
        assert!(result.contains(""bugs""));
    }

    #[test]
    fn test_escape_csv() {
        let formatter = ReportFormatter::new(OutputFormat::Csv);
        assert_eq!(formatter.escape_csv("simple"), "simple");
        assert_eq!(formatter.escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(formatter.escape_csv("with\"quote"), "\"with\"\"quote\"");
    }

    #[test]
    fn test_escape_html() {
        let formatter = ReportFormatter::new(OutputFormat::Html);
        assert_eq!(formatter.escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(formatter.escape_html("A & B"), "A &amp; B");
    }
}
