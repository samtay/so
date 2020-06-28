use criterion::{black_box, criterion_group, criterion_main, Criterion};
use so::tui::markdown::parse;

const MD: &str = r####"
## project

Here's some `inline code`. It should escape `*asterisks*`.
It should also respect

    indented code blocks

and
```python
code fences
```
Obviously.

### but also
I'm on a Mac running OS&nbsp;X&nbsp;v10.6 (Snow&nbsp;Leopard). I have Mercurial 1.1 installed.

After I hit **[Esc]** to exit insert mode I can't figure out how to save and quit. Hitting **[Ctrl]** + **[C]** shows me instructions that say typing "quit<enter>" will write and quit, but it doesn't seem to work.


"####;

// TODO bench preprocess as well, separately
pub fn md_benchmark(c: &mut Criterion) {
    c.bench_function("markdown::parse", |b| b.iter(|| parse(black_box(MD))));
}

criterion_group!(benches, md_benchmark);
criterion_main!(benches);
