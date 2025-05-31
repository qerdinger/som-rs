# ValBenchmark

This document follows the structure of the ValBenchmark folder, which contains various benchmarks for testing the performance of different data types and operations on Integer & Double.

---

## DoubleBench

Performs a sequence of basic float arithmetic operations to assess performance of standard operations such as `+ - * // %` on Double types.

- **Operations:** Addition, subtraction, multiplication, division, modulus  
- **Operands:** Mix of literals and local float variables  
- **Expected Result:** `1.5`

---

## DoubleMedBench

Performs a moderate number of float operations involving combinations of arithmetic and intermediate scaling factors.

- **Iterations:** 25  
- **Pattern:** Linear formula with nested operations  
- **Expected Result:** `6703950302750.187`

---

## DoubleHighBench

Executes a computationally intensive float workload simulating real-world float accumulation and value drift over large iteration counts.

- **Operations:** Multiplication, addition, incremental value growth  
- **Iterations:** 100 000  
- **Expected Result:** `3972083870680.3447`

---

## DoubleLExpBench

Stress test for extremely low double values, ensuring precision and correctness near the lower limits of IEEE 754 double representation.

- **Purpose:** Floating-point underflow and small value handling  
- **Iterations:** 100  
- **Expected Result:** `2.536869555601273e-124`

---

## DoubleSqrtBench

Evaluates the performance and numerical behavior of repeated square root operations over a large range of inputs.

- **Iterations:** 25 000  
- **Operation:** Cumulative square root chaining  
- **Expected Result:** `158.61150128198418`

---

## IntegerBench

Tests basic integer arithmetic performance using standard operations across constant and variable operands.

- **Operations:** Integer `+ - * / %`  
- **Operands:** Constants and local variables  
- **Expected Result:** `1`

---

## IntegerMedBench
Performs a moderate integer workload with a mix of arithmetic operations and intermediate scaling, iterating over a small range.

- **Iterations:** 25  
- **Pattern:** `result := result + i * 2 - (result / 2) + 15 * 2 - 8`  
- **Expected Result:** `10167463313254`

---

## IntegerHighBench

Measures integer accumulation and growth under compounded values over multiple dependent variables.

- **Iterations:** 1 000  
- **Pattern:** Accumulation of growing multipliers and loop variable  
- **Expected Result:** `668167500`

---

## IntegerBigBench

Performs large-scale integer accumulation over a high iteration count using exponential scaling to stress test integer range handling.

- **Iterations:** 1 000  
- **Computation:** Summation using `(i * 10^2)`  
- **Expected Result:** `Result of the summation if i * 10^2 for i in range(1000)`

---

## IntegerExpBench

A smaller-scale version of `IntegerBigBench` to test integer arithmetic performance and accuracy over exponential input with fewer iterations.

- **Iterations:** 100  
- **Computation:** `(i * 10^2)` for smaller range  
- **Expected Result:** `Result of the summation if i * 10^2 for i in range(100)`

---
