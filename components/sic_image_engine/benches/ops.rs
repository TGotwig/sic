use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::DynamicImage;
use sic_core::image;
use sic_image_engine::engine::{EnvItem, ImageEngine, Instr};
use sic_image_engine::ImgOp;

const BENCH_IMAGE: &'static [u8] = include_bytes!("../../../resources/bench/horses.jpg");

fn blur(image: DynamicImage) {
    let _ = ImageEngine::new(image).ignite(&[
        Instr::Operation(ImgOp::Crop((0, 0, 1000, 700))),
        Instr::Operation(ImgOp::GrayScale),
        Instr::EnvAdd(EnvItem::PreserveAspectRatio(true)),
        Instr::Operation(ImgOp::Resize((100, 100))),
    ]);
}

fn criterion_benchmark(criterion: &mut Criterion) {
    let image = image::load_from_memory(BENCH_IMAGE).unwrap();

    criterion.bench_function("ops", |bencher| {
        bencher.iter(|| blur(black_box(image.clone())));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
