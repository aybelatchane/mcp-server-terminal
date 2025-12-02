use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use terminal_mcp_core::{Dimensions, Position};
use terminal_mcp_detector::{
    BorderDetector, ButtonDetector, CheckboxDetector, DetectionContext, DetectionPipeline,
    ElementDetector, InputDetector, MenuDetector, ProgressDetector, StatusBarDetector,
    TableDetector,
};
use terminal_mcp_emulator::{Grid, Parser};

/// Create a grid with a bordered menu for benchmarking
fn create_menu_parser(rows: u16, cols: u16) -> Parser {
    let grid = Grid::new(Dimensions::new(rows, cols));
    let mut parser = Parser::new(grid);

    let menu_text = "\x1b[0m┌────────────────────┐\r\n│  View Status      │\r\n│\x1b[7m> Start Service\x1b[0m   │\r\n│  Stop Service     │\r\n│  Quit             │\r\n└────────────────────┘";

    parser.process(menu_text.as_bytes());
    parser
}

/// Create a grid with a table for benchmarking
fn create_table_parser(rows: u16, cols: u16) -> Parser {
    let grid = Grid::new(Dimensions::new(rows, cols));
    let mut parser = Parser::new(grid);

    let table_text = "\r\nID │ Name       │ Status  │ Time\r\n─────────────────────────────────\r\n1  │ Alice      │ Active  │ 12:30\r\n2  │ Bob        │ Idle    │ 12:35\r\n3  │ Charlie    │ Active  │ 12:40";

    parser.process(table_text.as_bytes());
    parser
}

/// Create a grid with various UI elements for full pipeline benchmark
fn create_complex_parser(rows: u16, cols: u16) -> Parser {
    let grid = Grid::new(Dimensions::new(rows, cols));
    let mut parser = Parser::new(grid);

    let complex_text = "─────────────────────────────────────────\r\n\r\nName: ____________________\r\n\r\n[ OK ]  [Cancel]\r\n\r\n[X] Enable notifications\r\n\r\nProgress: ████████████████████░░░░░░░░░░\r\n\r\n\x1b[7mStatus: Ready | Time: 12:30 | Items: 5\x1b[0m";

    parser.process(complex_text.as_bytes());
    parser
}

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    for size in [(24, 80), (40, 120), (60, 160)].iter() {
        let (rows, cols) = *size;
        let parser = create_complex_parser(rows, cols);
        let mut pipeline = DetectionPipeline::new();

        // Add all detectors to pipeline
        pipeline.add_detector(std::sync::Arc::new(BorderDetector::new()));
        pipeline.add_detector(std::sync::Arc::new(StatusBarDetector::new()));
        pipeline.add_detector(std::sync::Arc::new(MenuDetector::new()));
        pipeline.add_detector(std::sync::Arc::new(TableDetector::new()));
        pipeline.add_detector(std::sync::Arc::new(InputDetector::new()));
        pipeline.add_detector(std::sync::Arc::new(ButtonDetector::new()));
        pipeline.add_detector(std::sync::Arc::new(CheckboxDetector::new()));
        pipeline.add_detector(std::sync::Arc::new(ProgressDetector::new()));

        let cursor = Position::new(0, 0);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{rows}x{cols}")),
            &parser,
            |b, p| {
                b.iter(|| {
                    let elements = pipeline.detect(black_box(p.grid()), black_box(cursor));
                    black_box(elements);
                });
            },
        );
    }

    group.finish();
}

fn bench_border_detector(c: &mut Criterion) {
    let parser = create_menu_parser(24, 80);
    let detector = BorderDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    c.bench_function("border_detector", |b| {
        b.iter(|| {
            let elements = detector.detect(black_box(parser.grid()), black_box(&context));
            black_box(elements);
        });
    });
}

fn bench_menu_detector(c: &mut Criterion) {
    let parser = create_menu_parser(24, 80);
    let detector = MenuDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    c.bench_function("menu_detector", |b| {
        b.iter(|| {
            let elements = detector.detect(black_box(parser.grid()), black_box(&context));
            black_box(elements);
        });
    });
}

fn bench_table_detector(c: &mut Criterion) {
    let parser = create_table_parser(24, 80);
    let detector = TableDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    c.bench_function("table_detector", |b| {
        b.iter(|| {
            let elements = detector.detect(black_box(parser.grid()), black_box(&context));
            black_box(elements);
        });
    });
}

fn bench_input_detector(c: &mut Criterion) {
    let parser = create_complex_parser(24, 80);
    let detector = InputDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    c.bench_function("input_detector", |b| {
        b.iter(|| {
            let elements = detector.detect(black_box(parser.grid()), black_box(&context));
            black_box(elements);
        });
    });
}

fn bench_button_detector(c: &mut Criterion) {
    let parser = create_complex_parser(24, 80);
    let detector = ButtonDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    c.bench_function("button_detector", |b| {
        b.iter(|| {
            let elements = detector.detect(black_box(parser.grid()), black_box(&context));
            black_box(elements);
        });
    });
}

fn bench_checkbox_detector(c: &mut Criterion) {
    let parser = create_complex_parser(24, 80);
    let detector = CheckboxDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    c.bench_function("checkbox_detector", |b| {
        b.iter(|| {
            let elements = detector.detect(black_box(parser.grid()), black_box(&context));
            black_box(elements);
        });
    });
}

fn bench_progress_detector(c: &mut Criterion) {
    let parser = create_complex_parser(24, 80);
    let detector = ProgressDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    c.bench_function("progress_detector", |b| {
        b.iter(|| {
            let elements = detector.detect(black_box(parser.grid()), black_box(&context));
            black_box(elements);
        });
    });
}

fn bench_status_bar_detector(c: &mut Criterion) {
    let parser = create_complex_parser(24, 80);
    let detector = StatusBarDetector::new();
    let context = DetectionContext::new(Position::new(0, 0));

    c.bench_function("status_bar_detector", |b| {
        b.iter(|| {
            let elements = detector.detect(black_box(parser.grid()), black_box(&context));
            black_box(elements);
        });
    });
}

criterion_group!(
    benches,
    bench_full_pipeline,
    bench_border_detector,
    bench_menu_detector,
    bench_table_detector,
    bench_input_detector,
    bench_button_detector,
    bench_checkbox_detector,
    bench_progress_detector,
    bench_status_bar_detector
);
criterion_main!(benches);
