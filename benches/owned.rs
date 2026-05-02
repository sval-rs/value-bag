#[cfg(feature = "owned")]
mod imp {
    use value_bag::ValueBag;

    use criterion::{criterion_group, criterion_main, Criterion};
    use std::hint::black_box;

    pub fn criterion_benchmark(c: &mut Criterion) {
        c.bench_function("from u8 to owned", |b| {
            let v = ValueBag::from(1u8);

            b.iter(|| black_box(v.to_owned()))
        });

        c.bench_function("from str to owned", |b| {
            let v = ValueBag::from("a string");

            b.iter(|| black_box(v.to_owned()))
        });

        c.bench_function("from str owned clone", |b| {
            let v = ValueBag::from("a string").to_owned();

            b.iter(|| black_box(v.clone()))
        });

        c.bench_function("from str to shared", |b| {
            let v = ValueBag::from("a string");

            b.iter(|| black_box(v.to_shared()))
        });

        c.bench_function("from str shared clone", |b| {
            let v = ValueBag::from("a string").to_shared();

            b.iter(|| black_box(v.clone()))
        });

        c.bench_function("from display to owned", |b| {
            let v = ValueBag::from_display(&42);

            b.iter(|| black_box(v.to_owned()))
        });

        #[cfg(feature = "serde1")]
        {
            c.bench_function("from serde to owned", |b| {
                use value_bag_serde1::lib::ser::{Serialize, SerializeStruct, Serializer};

                pub struct MyData<'a> {
                    a: i32,
                    b: &'a str,
                }

                impl<'a> Serialize for MyData<'a> {
                    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                        let mut serializer = serializer.serialize_struct("MyData", 2)?;

                        serializer.serialize_field("a", &self.a)?;
                        serializer.serialize_field("b", &self.b)?;

                        serializer.end()
                    }
                }

                let v = ValueBag::from_serde1(&MyData {
                    a: 42,
                    b: "a string",
                });

                b.iter(|| black_box(v.to_owned()))
            });
        }

        #[cfg(feature = "sval2")]
        {
            c.bench_function("from sval to owned", |b| {
                use value_bag_sval2::lib::{Label, Result, Stream, Value};

                pub struct MyData<'a> {
                    a: i32,
                    b: &'a str,
                }

                impl<'a> Value for MyData<'a> {
                    fn stream<'sval, S: Stream<'sval> + ?Sized>(
                        &'sval self,
                        stream: &mut S,
                    ) -> Result {
                        stream.record_begin(None, Some(&Label::new("MyData")), None, Some(2))?;

                        stream.record_value_begin(None, &Label::new("a"))?;
                        stream.value(&self.a)?;
                        stream.record_value_end(None, &Label::new("a"))?;

                        stream.record_value_begin(None, &Label::new("b"))?;
                        stream.value(&self.b)?;
                        stream.record_value_end(None, &Label::new("b"))?;

                        stream.record_end(None, Some(&Label::new("MyData")), None)
                    }
                }

                let v = ValueBag::from_sval2(&MyData {
                    a: 42,
                    b: "a string",
                });

                b.iter(|| black_box(v.to_owned()))
            });
        }
    }
}

#[cfg(feature = "owned")]
criterion_group!(benches, imp::criterion_benchmark);
#[cfg(feature = "owned")]
criterion_main!(benches);

#[cfg(not(feature = "owned"))]
fn main() {}
