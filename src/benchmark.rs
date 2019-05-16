#[macro_use]
extern crate criterion;

use criterion::*;

mod linecollector;
use linecollector::LineCollector;

const NUM_TEST_LINES: usize = 1000;
const NUM_DISP_LINES: usize = 50;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
        "lines-throughput",
        Benchmark::new("empty_lines", |b| {
            let empty_lines_input = vec![""; NUM_TEST_LINES];
            let mut lc = LineCollector::new();

            b.iter(|| {
                for i in empty_lines_input.iter() {
                    lc.insert(black_box(i));
                    black_box(lc.iter().take(NUM_DISP_LINES).count());
                }
            })
        })
        .throughput(Throughput::Elements(NUM_TEST_LINES as u32)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
