#[cfg(feature = "seq")]
mod imp {
    use value_bag::ValueBag;

    use criterion::{criterion_group, criterion_main, Criterion};
    use std::hint::black_box;

    pub fn criterion_benchmark(c: &mut Criterion) {
        #[cfg(feature = "serde1")]
        {
            c.bench_function("from serde to f64 seq 5", |b| {
                let v = ValueBag::from_serde1(&[1.0, 2.0, 3.0, 4.0, 5.0]);

                b.iter(|| black_box(v.to_f64_seq::<Vec<Option<f64>>>()))
            });
        }

        #[cfg(feature = "sval2")]
        {
            c.bench_function("from sval to f64 seq 5", |b| {
                let v = ValueBag::from_sval2(&[1.0, 2.0, 3.0, 4.0, 5.0]);

                b.iter(|| black_box(v.to_f64_seq::<Vec<Option<f64>>>()))
            });
        }
    }
}

#[cfg(feature = "seq")]
criterion_group!(benches, imp::criterion_benchmark);
#[cfg(feature = "seq")]
criterion_main!(benches);

#[cfg(not(feature = "seq"))]
fn main() {}
