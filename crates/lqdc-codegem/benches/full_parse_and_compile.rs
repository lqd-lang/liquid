use std::fs::read_to_string;

use codegem::ir::ModuleBuilder;
use criterion::*;
use lqdc_codegem::codegen::CodegenPass;
use lqdc_common::{
    codepass::PassRunner, make_signatures::MakeSignaturesPass, parsepass::ParsePass,
};

fn criterion_benchmark(c: &mut Criterion) {
    let mut module_builder = ModuleBuilder::default();
    let input = read_to_string("benches/benchmark.lqd").unwrap();
    c.bench_function("full_parse_and_compile", |b| {
        b.iter(|| {
            PassRunner::<(), ()>::new(&input)
                .run::<ParsePass>()
                .unwrap()
                .run::<MakeSignaturesPass>()
                .unwrap()
                .set_arg(&mut module_builder)
                .run::<CodegenPass>()
                .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
