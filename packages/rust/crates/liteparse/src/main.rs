use clap::{Args, Parser, Subcommand};
use liteparse_rs::extract;
use liteparse_rs::ocr::tesseract::TesseractOcrEngine;
use liteparse_rs::ocr_merge;
use liteparse_rs::projection;
use liteparse_rs::render;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Extract raw text items from a PDF file (no grid projection)
    Extract(PdfCommand),
    /// Parse a PDF file: extract + grid projection, output projected pages as JSON
    Parse(ParseCommand),
    /// Render a page to a PNG screenshot (file output)
    Screenshot(ScreenshotCommand),
    /// Render a page to PNG and write to stdout
    ScreenshotStdout(ScreenshotStdoutCommand),
    /// Extract embedded image bounding boxes from a page
    ImageBounds(PdfCommand),
}

#[derive(Args, Debug)]
struct PdfCommand {
    /// Specify the path to the PDF file
    #[arg(long)]
    pdf_path: String,

    /// Optionally specify a target page number
    #[arg(long)]
    page_num: Option<u32>,
}

#[derive(Args, Debug)]
struct ParseCommand {
    /// Specify the path to the PDF file
    #[arg(long)]
    pdf_path: String,

    /// Optionally specify a target page number
    #[arg(long)]
    page_num: Option<u32>,

    /// Enable OCR for text-sparse pages and embedded images
    #[arg(long)]
    ocr: bool,

    /// OCR language (Tesseract format, e.g. "eng", "fra", "deu")
    #[arg(long, default_value = "eng")]
    ocr_language: String,

    /// Path to tessdata directory (overrides TESSDATA_PREFIX env var)
    #[arg(long)]
    tessdata_path: Option<String>,

    /// DPI for rendering pages for OCR (default: 150)
    #[arg(long, default_value = "150")]
    ocr_dpi: f32,
}

#[derive(Args, Debug)]
struct ScreenshotCommand {
    /// Specify the path to the PDF file
    #[arg(long)]
    pdf_path: String,

    /// Target page number (1-based)
    #[arg(long)]
    page_num: u32,

    /// Output PNG file path
    #[arg(long)]
    output: String,

    /// DPI for rendering (default: 150)
    #[arg(long, default_value = "150")]
    dpi: f32,
}

#[derive(Args, Debug)]
struct ScreenshotStdoutCommand {
    /// Specify the path to the PDF file
    #[arg(long)]
    pdf_path: String,

    /// Target page number (1-based)
    #[arg(long)]
    page_num: u32,

    /// DPI for rendering (default: 150)
    #[arg(long, default_value = "150")]
    dpi: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Extract(cmd) => {
            extract::extract(&cmd.pdf_path, cmd.page_num)?;
        }
        Commands::Parse(cmd) => {
            let t0 = std::time::Instant::now();
            let mut pages = extract::extract_pages(&cmd.pdf_path, cmd.page_num)?;
            let t1 = std::time::Instant::now();

            if cmd.ocr {
                let engine = TesseractOcrEngine::new(cmd.tessdata_path);
                ocr_merge::ocr_and_merge_pages(
                    &mut pages,
                    &cmd.pdf_path,
                    cmd.ocr_dpi,
                    &engine,
                    &cmd.ocr_language,
                )?;
            }
            let t_ocr = std::time::Instant::now();

            let parsed_pages = projection::project_pages_to_grid(pages);
            let t2 = std::time::Instant::now();
            let json = serde_json::to_string(&parsed_pages)?;
            let t3 = std::time::Instant::now();
            println!("{}", json);
            eprintln!(
                "[rust-bin] extract: {:.1}ms, ocr: {:.1}ms, project: {:.1}ms, serialize: {:.1}ms, total: {:.1}ms",
                t1.duration_since(t0).as_secs_f64() * 1000.0,
                t_ocr.duration_since(t1).as_secs_f64() * 1000.0,
                t2.duration_since(t_ocr).as_secs_f64() * 1000.0,
                t3.duration_since(t2).as_secs_f64() * 1000.0,
                t3.duration_since(t0).as_secs_f64() * 1000.0,
            );
        }
        Commands::Screenshot(cmd) => {
            render::screenshot(&cmd.pdf_path, cmd.page_num, cmd.dpi, &cmd.output)?;
        }
        Commands::ScreenshotStdout(cmd) => {
            render::screenshot_to_stdout(&cmd.pdf_path, cmd.page_num, cmd.dpi)?;
        }
        Commands::ImageBounds(cmd) => {
            render::image_bounds(&cmd.pdf_path, cmd.page_num)?;
        }
    }

    Ok(())
}
