// use compiler::compiler::Compiler;
// use criterion::{criterion_group, criterion_main, Criterion};

// fn generic_bench(c: &mut Criterion) {
//     let code = "class Foo<T> {
//         inner: T,
//     }

//     def inner<T>(self) for Foo -> T {
//         return self.inner;
//     }

//     def main() -> int {
//         let a = Foo<Foo<int>> {
//             inner: Foo<int> {
//                 inner: 10,
//             }
//         };

//         let foo_arr = [[[[[[[[[[a, a, a]]]]]]]]]];

//         let inner = foo_arr[0][0][0][0][0][0][0][0][0][0].inner<Foo<int>>().inner<int>();
//         return inner;
//     }";

//     c.bench_function("generic datatypes", |b| {
//         b.iter(|| Compiler::compile(code, None))
//     });
// }

// criterion_group!(benches, generic_bench);
// criterion_main!(benches);
