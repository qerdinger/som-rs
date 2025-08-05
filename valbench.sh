DIRECTORY="core-lib/Examples/Benchmarks/ValBenchmark"
EXEC="./target/release/som-interpreter-bc"

cargo build --release --features=som-gc/marksweep --features=idiomatic

for file in $DIRECTORY/*Bench.som; do
    if [ -f "$file" ]; then
        echo "$file"
        basename="${file##*/}"
        test_name="${basename%Bench.som}"
        echo "--- Launching: $test_name ---"
        #cargo run --bin ${EXE:=som-interpreter-bc} --no-default-features --features=som-gc/marksweep,idiomatic -- -c core-lib/Smalltalk core-lib/TestSuite -- TestHarness ${test_name}Test
        $EXEC -c core-lib/Smalltalk core-lib/Examples/Benchmarks core-lib/Examples/Benchmarks/LanguageFeatures core-lib/Examples/Benchmarks/TestSuite core-lib/Examples/Benchmarks/ValBenchmark . -- BenchmarkHarness ${test_name}Bench 1
        echo "-----------------------------"
        sleep 5
    fi
done