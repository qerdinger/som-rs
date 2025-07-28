DIRECTORY="core-lib/TestSuite"
 
for file in $DIRECTORY/*Test.som; do
    if [ -f "$file" ]; then
        echo "$file"
        basename="${file##*/}"
        test_name="${basename%Test.som}"
        echo "$test_name"
        cargo run --bin ${EXE:=som-interpreter-bc} --no-default-features --features=som-gc/marksweep,idiomatic -- -c core-lib/Smalltalk core-lib/TestSuite -- TestHarness ${test_name}Test
        sleep 2
    fi
done