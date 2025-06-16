use std::env;
use rand::random;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::{hint::black_box, time::{Duration, Instant}};

use som_gc::gcref::Gc;
use som_gc::gc_interface::{GCInterface, SOMAllocator};

use som_interpreter_bc::gc::get_callbacks_for_gc;
use som_interpreter_bc::universe::DEFAULT_HEAP_SIZE;

// Import both implementations
use som_value::value::BaseValue as NanBoxedValue;
use som_value_lbits::value::BaseValue as LbitsValue;

pub fn bench_value_representations(c: &mut Criterion) {
    let args: Vec<String> = env::args().collect();
    let bench_name = if args.len() > 1 { &args[1] } else { "value_representation_comparison" };

    let mut group = c.benchmark_group(bench_name);
    group.warm_up_time(Duration::from_millis(1));
    group.measurement_time(Duration::from_secs(1));

    // GC initialized ONCE
    let gc_interface = GCInterface::init(DEFAULT_HEAP_SIZE, get_callbacks_for_gc());

    // Pre-allocate separate Gc<String> values for each benchmark
    let gc_str_for_nan = gc_interface.alloc("This is a string !".to_string());
    let gc_str_for_lbits = gc_interface.alloc("This is a string !".to_string());

    // Benchmark creation operations
    group.bench_function("create_integer/NaN", |b| {
        b.iter(|| {
            let val: i32 = black_box(random());
            let v = NanBoxedValue::new_integer(val);
            black_box(v);
        })
    });

    group.bench_function("create_integer/LBits", |b| {
        b.iter(|| {
            let val: i32 = black_box(random());
            let v = LbitsValue::new_integer(val);
            black_box(v);
        })
    });

    group.bench_function("create_double/NaN", |b| {
        b.iter(|| {
            let val: f64 = black_box(random());
            let v = NanBoxedValue::new_double(val);
            black_box(v);
        })
    });

    group.bench_function("create_double/LBits", |b| {
        b.iter(|| {
            let val: f64 = black_box(random());
            let v = LbitsValue::new_double(val);
            black_box(v);
        })
    });

    group.bench_function("create_boolean/NaN", |b| {
        b.iter(|| {
            let val: bool = black_box(random());
            let v = NanBoxedValue::new_boolean(val);
            black_box(v);
        })
    });

    group.bench_function("create_boolean/LBits", |b| {
        b.iter(|| {
            let val: bool = black_box(random());
            let v = LbitsValue::new_boolean(val);
            black_box(v);
        })
    });

    group.bench_function("create_nil/NaN", |b| {
        b.iter(|| {
            let v = NanBoxedValue::NIL;
            black_box(v);
        })
    });

    group.bench_function("create_nil/LBits", |b| {
        b.iter(|| {
            let v = LbitsValue::NIL;
            black_box(v);
        })
    });

    group.bench_function("create_string/NaN", |b| {
        let gc_val = gc_str_for_nan.clone();
        b.iter(|| {
            let fstr = NanBoxedValue::new_string(gc_val.clone());
            black_box(fstr);
        });
    });

    group.bench_function("create_string/LBits", |b| {
        let gc_val = gc_str_for_lbits.clone();
        b.iter(|| {
            let fstr = LbitsValue::new_string(gc_val.clone());
            black_box(fstr);
        });
    });

    // Create test values
    let nan_int = NanBoxedValue::new_integer(5002);
    let lbits_int = LbitsValue::new_integer(5002);
    let nan_int_max = NanBoxedValue::new_integer(i32::MAX);
    let lbits_int_max = LbitsValue::new_integer(i32::MAX);
    let nan_int_neg = NanBoxedValue::new_integer(-5002);
    let lbits_int_neg = LbitsValue::new_integer(-5002);
    let nan_int_min = NanBoxedValue::new_integer(i32::MIN);
    let lbits_int_min = LbitsValue::new_integer(i32::MIN);
    let nan_double = NanBoxedValue::new_double(3.14);
    let lbits_double = LbitsValue::new_double(3.14);
    let nan_bool = NanBoxedValue::new_boolean(true);
    let lbits_bool = LbitsValue::new_boolean(true);
    let nan_nil = NanBoxedValue::NIL;
    let lbits_nil = LbitsValue::NIL;
    let gc_string: Gc<String> = gc_interface.alloc("This is a string !".to_string());
    let gc2_string: Gc<String> = gc_interface.alloc("This is a string !".to_string());
    let nan_string = NanBoxedValue::new_string(gc_string);
    let lbits_string = LbitsValue::new_string(gc2_string);

    macro_rules! bench_check {
        ($group:ident, $prefix:literal, $label:ident, $exprs:expr) => {
            $group.bench_function(concat!($prefix, stringify!($label)), |b| {
                b.iter(|| {
                    for expr in $exprs {
                        black_box(expr);
                    }
                });
            });
        };
    }

    bench_check!(group, "NanBoxing/", is_integer_check, [
        nan_int.is_integer(),
        nan_int_neg.is_integer(),
        nan_int_max.is_integer(),
        nan_int_min.is_integer(),
        nan_double.is_integer(),
        nan_bool.is_integer(),
        nan_nil.is_integer(),
        nan_string.is_integer()
    ]);

    bench_check!(group, "Lbits/", is_integer_check, [
        lbits_int.is_integer(),
        lbits_int_neg.is_integer(),
        lbits_int_max.is_integer(),
        lbits_int_min.is_integer(),
        lbits_double.is_integer(),
        lbits_bool.is_integer(),
        lbits_nil.is_integer(),
        lbits_string.is_integer()
    ]);

    bench_check!(group, "NanBoxing/", is_double_check, [
        nan_int.is_double(),
        nan_int_neg.is_double(),
        nan_int_max.is_double(),
        nan_int_min.is_double(),
        nan_double.is_double(),
        nan_bool.is_double(),
        nan_nil.is_double(),
        nan_string.is_double()
    ]);

    bench_check!(group, "Lbits/", is_double_check, [
        lbits_int.is_double(),
        lbits_int_neg.is_double(),
        lbits_int_max.is_double(),
        lbits_int_min.is_double(),
        lbits_double.is_double(),
        lbits_bool.is_double(),
        lbits_nil.is_double(),
        lbits_string.is_double()
    ]);

    bench_check!(group, "NanBoxing/", is_boolean_check, [
        nan_int.is_boolean(),
        nan_int_neg.is_boolean(),
        nan_int_max.is_boolean(),
        nan_int_min.is_boolean(),
        nan_double.is_boolean(),
        nan_bool.is_boolean(),
        nan_nil.is_boolean(),
        nan_string.is_boolean()
    ]);

    bench_check!(group, "Lbits/", is_boolean_check, [
        lbits_int.is_boolean(),
        lbits_int_neg.is_boolean(),
        lbits_int_max.is_boolean(),
        lbits_int_min.is_boolean(),
        lbits_double.is_boolean(),
        lbits_bool.is_boolean(),
        lbits_nil.is_boolean(),
        lbits_string.is_boolean()
    ]);

    bench_check!(group, "NanBoxing/", is_nil_check, [
        nan_int.is_nil(),
        nan_int_neg.is_nil(),
        nan_int_max.is_nil(),
        nan_int_min.is_nil(),
        nan_double.is_nil(),
        nan_bool.is_nil(),
        nan_nil.is_nil(),
        nan_string.is_nil()
    ]);

    bench_check!(group, "Lbits/", is_nil_check, [
        lbits_int.is_nil(),
        lbits_int_neg.is_nil(),
        lbits_int_max.is_nil(),
        lbits_int_min.is_nil(),
        lbits_double.is_nil(),
        lbits_bool.is_nil(),
        lbits_nil.is_nil(),
        lbits_string.is_nil()
    ]);

    bench_check!(group, "NanBoxing/", is_string_check, [
        nan_int.is_string(),
        nan_int_neg.is_string(),
        nan_int_max.is_string(),
        nan_int_min.is_string(),
        nan_double.is_string(),
        nan_bool.is_string(),
        nan_nil.is_string(),
        nan_string.is_string()
    ]);

    bench_check!(group, "Lbits/", is_string_check, [
        lbits_int.is_string(),
        lbits_int_neg.is_string(),
        lbits_int_max.is_string(),
        lbits_int_min.is_string(),
        lbits_double.is_string(),
        lbits_bool.is_string(),
        lbits_nil.is_string(),
        lbits_string.is_string()
    ]);

    bench_check!(group, "NanBoxing/", is_ptr_type_check, [
        nan_int.is_ptr_type(),
        nan_int_neg.is_ptr_type(),
        nan_int_max.is_ptr_type(),
        nan_int_min.is_ptr_type(),
        nan_double.is_ptr_type(),
        nan_bool.is_ptr_type(),
        nan_nil.is_ptr_type(),
        nan_string.is_ptr_type()
    ]);

    bench_check!(group, "Lbits/", is_ptr_type_check, [
        lbits_int.is_ptr_type(),
        lbits_int_neg.is_ptr_type(),
        lbits_int_max.is_ptr_type(),
        lbits_int_min.is_ptr_type(),
        lbits_double.is_ptr_type(),
        lbits_bool.is_ptr_type(),
        lbits_nil.is_ptr_type(),
        lbits_string.is_ptr_type()
    ]);

    group.bench_function("extract_integer/NaN", |b| {
        b.iter(|| black_box(nan_int.as_integer()));
    });

    group.bench_function("extract_integer/LBits", |b| {
        b.iter(|| black_box(lbits_int.as_integer()));
    });

    group.bench_function("extract_double/NaN", |b| {
        b.iter(|| black_box(nan_double.as_double()));
    });

    group.bench_function("extract_double/Lbits", |b| {
        b.iter(|| black_box(lbits_double.as_double()));
    });

    group.bench_function("extract_boolean/NaN", |b| {
        b.iter(|| black_box(nan_bool.as_boolean()));
    });

    group.bench_function("extract_boolean/LBits", |b| {
        b.iter(|| black_box(lbits_bool.as_boolean()));
    });

    group.bench_function("extract_nil_as_boolean/NaN", |b| {
        b.iter(|| black_box(nan_nil.as_boolean()));
    });

    group.bench_function("extract_nil_as_boolean/LBits", |b| {
        b.iter(|| black_box(lbits_nil.as_boolean()));
    });

    group.bench_function("extract_string/NaN", |b| {
        b.iter(|| black_box(nan_string.as_string::<Gc<String>>()));
    });

    group.bench_function("extract_string/LBits", |b| {
        b.iter(|| black_box(lbits_string.as_string::<Gc<String>>()));
    });

    group.bench_function("tag_extraction/NaN", |b| {
        b.iter(|| {
            black_box(nan_int.tag());
            black_box(nan_double.tag());
            black_box(nan_bool.tag());
            black_box(nan_nil.tag());
            black_box(nan_string.tag());
        });
    });

    group.bench_function("tag_extraction/LBits", |b| {
        b.iter(|| {
            black_box(lbits_int.tag());
            black_box(lbits_double.tag());
            black_box(lbits_bool.tag());
            black_box(lbits_nil.tag());
            black_box(lbits_string.tag());
        });
    });

    group.bench_function("payload_extraction/NaN", |b| {
        b.iter(|| {
            black_box(nan_int.payload());
            black_box(nan_double.payload());
            black_box(nan_bool.payload());
            black_box(nan_nil.payload());
            black_box(nan_string.payload());
        });
    });

    group.bench_function("payload_extraction/LBits", |b| {
        b.iter(|| {
            black_box(lbits_int.payload());
            black_box(lbits_double.payload());
            black_box(lbits_bool.payload());
            black_box(lbits_nil.payload());
            black_box(lbits_string.payload());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_value_representations);
criterion_main!(benches);
