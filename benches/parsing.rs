use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use so::stackexchange::scraper::{DuckDuckGo, Google, Scraper};
use std::collections::HashMap;
use std::time::Duration;

fn bench_parsers(c: &mut Criterion) {
    let limit: u16 = 10;
    let mut sites = HashMap::new();
    sites.insert(
        String::from("stackoverflow"),
        String::from("stackoverflow.com"),
    );
    sites.insert(String::from("askubuntu"), String::from("askubuntu.com"));

    let mut group = c.benchmark_group("Scraping html");

    group.sample_size(80);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(limit as u64));

    group.bench_with_input(
        BenchmarkId::new("Google.parse", "exit-vim"),
        include_str!("../test/google/exit-vim.html"),
        |b, html| b.iter(|| Google.parse(html, &sites, limit)),
    );

    group.bench_with_input(
        BenchmarkId::new("DuckDuckGo.parse", "exit-vim"),
        include_str!("../test/duckduckgo/exit-vim.html"),
        |b, html| b.iter(|| DuckDuckGo.parse(html, &sites, limit)),
    );

    let mut sites = HashMap::new();
    sites.insert(
        String::from("stackoverflow"),
        String::from("stackoverflow.com"),
    );

    group.bench_with_input(
        BenchmarkId::new("Google.parse", "/q/"),
        include_str!("../test/google/parsing-q.html"),
        |b, html| b.iter(|| Google.parse(html, &sites, limit)),
    );

    let mut sites = HashMap::new();
    sites.insert(String::from("meta"), String::from("meta.stackexchange.com"));

    group.bench_with_input(
        BenchmarkId::new("DuckDuckGo.parse", "tagged"),
        include_str!("../test/duckduckgo/tagged.html"),
        |b, html| b.iter(|| DuckDuckGo.parse(html, &sites, limit)),
    );

    group.finish();
}

criterion_group!(benches, bench_parsers);
criterion_main!(benches);
