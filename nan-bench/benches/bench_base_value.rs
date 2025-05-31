use criterion::{criterion_group, criterion_main, Criterion};
use std::{hint::black_box};
use rand::random;
use som_value::value::BaseValue;
use som_gc::gcref::Gc;
use som_gc::gc_interface::GCInterface;
use som_interpreter_bc::gc::get_callbacks_for_gc;
use som_interpreter_bc::universe::DEFAULT_HEAP_SIZE;

pub fn bench_nan_boxing(c: &mut Criterion) {

    c.bench_function("create_string", |b| {
        b.iter_with_setup(
            || GCInterface::init(DEFAULT_HEAP_SIZE, get_callbacks_for_gc()),
            |gc_interface| {
                let s = black_box("Hello, World!".to_string());
                let gc_string: Gc<String> = gc_interface.alloc(s);
                let v = BaseValue::new_string(gc_string);
                black_box(v);
            },
        )
    });

    // Benchmark Integer creation
    c.bench_function("create_integer", |b| {
        b.iter(|| {
            let val: i32 = black_box(random());
            let v = BaseValue::new_integer(val);
            black_box(v);
        })
    });

    // Benchmark Double creation
    c.bench_function("create_double", |b| {
        b.iter(|| {
            let val: f64 = black_box(random());
            let v = BaseValue::new_double(val);
            black_box(v);
        })
    });

    // Benchmark Boolean creation
    c.bench_function("create_boolean", |b| {
        b.iter(|| {
            let val: bool = black_box(random());
            let v = BaseValue::new_boolean(val);
            black_box(v);
        })
    });

    // Benchmark Nil creation
    c.bench_function("create_nil", |b| {
        b.iter(|| {
            let v = BaseValue::NIL;
            black_box(v);
        })
    });

    // Benchmarking Integer, Double, Boolean, and Nil checks
    let int_val = BaseValue::new_integer(42);
    let double_val = BaseValue::new_double(3.14);
    let bool_val = BaseValue::new_boolean(true);
    let nil_val = BaseValue::NIL;

    // Benchmarking type checks
    c.bench_function("is_integer_check", |b| {
        b.iter(|| {
            black_box(int_val.is_integer());
            black_box(double_val.is_integer());
            black_box(bool_val.is_integer());
            black_box(nil_val.is_integer());
        })
    });

    c.bench_function("is_double_check", |b| {
        b.iter(|| {
            black_box(int_val.is_double());
            black_box(double_val.is_double());
            black_box(bool_val.is_double());
            black_box(nil_val.is_double());
        })
    });

    c.bench_function("is_boolean_check", |b| {
        b.iter(|| {
            black_box(int_val.is_boolean());
            black_box(double_val.is_boolean());
            black_box(bool_val.is_boolean());
            black_box(nil_val.is_boolean());
        })
    });

    c.bench_function("is_nil_check", |b| {
        b.iter(|| {
            black_box(int_val.is_nil());
            black_box(double_val.is_nil());
            black_box(bool_val.is_nil());
            black_box(nil_val.is_nil());
        })
    });

    c.bench_function("is_ptr_type_check", |b| {
        b.iter(|| {
            black_box(int_val.is_ptr_type());
            black_box(double_val.is_ptr_type());
            black_box(bool_val.is_ptr_type());
            black_box(nil_val.is_ptr_type());
        })
    });

    // Benchmarking extraction methods
    c.bench_function("extract_integer", |b| {
        b.iter(|| {
            black_box(int_val.as_integer());
        })
    });

    c.bench_function("extract_double", |b| {
        b.iter(|| {
            black_box(double_val.as_double());
        })
    });

    c.bench_function("extract_boolean", |b| {
        b.iter(|| {
            black_box(bool_val.as_boolean());
        })
    });

    c.bench_function("extract_nil_as_boolean", |b| {
        b.iter(|| {
            black_box(nil_val.as_boolean());
        })
    });

    // Benchmarking tag extraction
    c.bench_function("tag_extraction", |b| {
        b.iter(|| {
            black_box(int_val.tag());
            black_box(double_val.tag());
            black_box(bool_val.tag());
            black_box(nil_val.tag());
        })
    });

    // Benchmarking payload extraction
    c.bench_function("payload_extraction", |b| {
        b.iter(|| {
            black_box(int_val.payload());
            black_box(double_val.payload());
            black_box(bool_val.payload());
            black_box(nil_val.payload());
        })
    });
}

criterion_group!(benches, bench_nan_boxing);
criterion_main!(benches);
