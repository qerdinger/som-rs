use std::env;
use rand::random;
use criterion::{criterion_group, criterion_main, Criterion};
use std::{hint::black_box, time::Duration};

use som_gc::gcref::Gc;
use som_gc::gc_interface::{GCInterface, SOMAllocator};

use som_interpreter_bc::gc::get_callbacks_for_gc;
use som_interpreter_bc::universe::DEFAULT_HEAP_SIZE;

use som_value::value::BaseValue;

pub fn bench_nan_boxing(c: &mut Criterion) {
    let args: Vec<String>  = env::args().collect();
    let bench_name: &str = if args.len() > 2 { &args[1] } else { "bench_tagging_scheme" };
    println!("Saving benchmarks as : [{}]", bench_name);
    let mut bench_group = c.benchmark_group(bench_name);
    bench_group.warm_up_time(Duration::from_millis(1));
    bench_group.measurement_time(Duration::from_secs(1));

    bench_group.bench_function("create_integer", |b| {
        b.iter(|| {
            let val: i32 = black_box(random());
            let v = BaseValue::new_integer(val);
            black_box(v);
        })
    });

    bench_group.bench_function("create_double", |b| {
        b.iter(|| {
            let val: f64 = black_box(random());
            let v = BaseValue::new_double(val);
            black_box(v);
        })
    });

    bench_group.bench_function("create_boolean", |b| {
        b.iter(|| {
            let val: bool = black_box(random());
            let v = BaseValue::new_boolean(val);
            black_box(v);
        })
    });

    bench_group.bench_function("create_nil", |b| {
        b.iter(|| {
            let v = BaseValue::NIL;
            black_box(v);
        })
    });

    bench_group.bench_function("create_string", |b| {
        b.iter_with_setup(
            || GCInterface::init(DEFAULT_HEAP_SIZE, get_callbacks_for_gc()),
            |gc_interface| {
                let s = black_box("This is a string !".to_string());
                let gc_string: Gc<String> = gc_interface.alloc(s);
                let fstr = BaseValue::new_string(gc_string);
                black_box(fstr);
            },
        )
    });

    let int_val = BaseValue::new_integer(5002);
    let int_max_val = BaseValue::new_integer(i32::MAX);
    let int_neg_val = BaseValue::new_integer(-5002);
    let int_min_val = BaseValue::new_integer(i32::MIN);
    let double_val = BaseValue::new_double(3.14);
    let bool_t_val = BaseValue::new_boolean(true);
    let bool_f_val = BaseValue::new_boolean(false);
    let nil_val = BaseValue::NIL;
    let gc_interface = GCInterface::init(DEFAULT_HEAP_SIZE, get_callbacks_for_gc());
    let gc_string: Gc<String> = gc_interface.alloc("This is a string !".to_string());
    let string_val = BaseValue::new_string(gc_string);

    println!("{:?}", int_val.as_integer().unwrap());
    println!("{:?}", int_neg_val.as_integer().unwrap());
    println!("{:?}", int_max_val.as_integer().unwrap());
    println!("{:?}", int_min_val.as_integer().unwrap());
    println!("{:?}", double_val.as_double().unwrap());
    println!("{:?}", bool_t_val.as_boolean().unwrap());
    println!("{:?}", bool_f_val.as_boolean().unwrap());
    println!("{:?}", nil_val.is_nil());
    println!("{:?}", string_val.as_string::<Gc<String>>().unwrap());

    bench_group.bench_function("is_integer_check", |b| {
        b.iter(|| {
            black_box(int_val.is_integer());
            black_box(int_neg_val.is_integer());
            black_box(int_max_val.is_integer());
            black_box(int_min_val.is_integer());
            black_box(double_val.is_integer());
            black_box(bool_t_val.is_integer());
            black_box(bool_f_val.is_integer());
            black_box(nil_val.is_integer());
            black_box(string_val.is_integer());
        })
    });

    bench_group.bench_function("is_double_check", |b| {
        b.iter(|| {
            black_box(int_val.is_double());
            black_box(int_neg_val.is_double());
            black_box(int_max_val.is_double());
            black_box(int_min_val.is_double());
            black_box(double_val.is_double());
            black_box(bool_t_val.is_double());
            black_box(bool_f_val.is_double());
            black_box(nil_val.is_double());
            black_box(string_val.is_double());
        })
    });

    bench_group.bench_function("is_boolean_check", |b| {
        b.iter(|| {
            black_box(int_val.is_boolean());
            black_box(int_neg_val.is_boolean());
            black_box(int_max_val.is_boolean());
            black_box(int_min_val.is_boolean());
            black_box(double_val.is_boolean());
            black_box(bool_t_val.is_boolean());
            black_box(bool_f_val.is_boolean());
            black_box(nil_val.is_boolean());
            black_box(string_val.is_boolean());
        })
    });

    bench_group.bench_function("is_nil_check", |b| {
        b.iter(|| {
            black_box(int_val.is_nil());
            black_box(int_neg_val.is_nil());
            black_box(int_max_val.is_nil());
            black_box(int_min_val.is_nil());
            black_box(double_val.is_nil());
            black_box(bool_t_val.is_nil());
            black_box(bool_f_val.is_nil());
            black_box(nil_val.is_nil());
            black_box(string_val.is_nil());
        })
    });

    bench_group.bench_function("is_string_check", |b| {
        b.iter(|| {
            black_box(int_val.is_string());
            black_box(int_neg_val.is_string());
            black_box(int_max_val.is_string());
            black_box(int_min_val.is_string());
            black_box(double_val.is_string());
            black_box(bool_t_val.is_string());
            black_box(bool_f_val.is_string());
            black_box(nil_val.is_string());
            black_box(string_val.is_string());
        })
    });

    bench_group.bench_function("is_ptr_type_check", |b| {
        b.iter(|| {
            black_box(int_val.is_ptr_type());
            black_box(int_neg_val.is_ptr_type());
            black_box(int_max_val.is_ptr_type());
            black_box(int_min_val.is_ptr_type());
            black_box(double_val.is_ptr_type());
            black_box(bool_t_val.is_ptr_type());
            black_box(bool_f_val.is_ptr_type());
            black_box(nil_val.is_ptr_type());
            black_box(string_val.is_ptr_type());
        })
    });

    bench_group.bench_function("extract_integer", |b| {
        b.iter(|| {
            black_box(int_val.as_integer());
        })
    });

    bench_group.bench_function("extract_double", |b| {
        b.iter(|| {
            black_box(double_val.as_double());
        })
    });

    bench_group.bench_function("extract_boolean", |b| {
        b.iter(|| {
            black_box(bool_t_val.as_boolean());
        })
    });

    bench_group.bench_function("extract_nil_as_boolean", |b| {
        b.iter(|| {
            black_box(nil_val.as_boolean());
        })
    });

    bench_group.bench_function("extract_string", |b| {
        b.iter(|| {
            black_box(string_val.as_string::<Gc<String>>());
        })
    });

    bench_group.bench_function("tag_extraction", |b| {
        b.iter(|| {
            black_box(int_val.tag());
            black_box(double_val.tag());
            black_box(bool_t_val.tag());
            black_box(nil_val.tag());
            black_box(string_val.tag());
        })
    });

    bench_group.bench_function("payload_extraction", |b| {
        b.iter(|| {
            black_box(int_val.payload());
            black_box(double_val.payload());
            black_box(bool_t_val.payload());
            black_box(nil_val.payload());
            black_box(string_val.payload());
        })
    });

    bench_group.finish();
}

criterion_group!(benches, bench_nan_boxing);
criterion_main!(benches);
