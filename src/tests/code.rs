#![cfg(test)]
use crate::Compiler;
use rstest::rstest;

#[rstest]
#[case("def main() -> int64 { return 0; }", true)]
#[case(
    "def main() -> int64 {
    let a = 5;
    return a;
}",
    true
)]
#[case(
    "def main() -> int64 {
    let a = 1 - 2;
    return a;
}",
    true
)]
#[case(
    "def main() -> int64 {
    let a = - 2;
    return a;
}",
    true
)]
#[case(
    "def main() -> int64 {
        let a = 5;
        a = 6;
        return a;
    }",
    false
)]
#[case(
    "def main() -> int64 {
        let a = 5;
        let a = 49;
        return a * 3;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let a = [1, 2, 3];
        let b = [2, 3, 4, 5];
        return a[0] * b[3];
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let mut a = 5;
        a = 1.0;
        return 0;
    }",
    false
)]
#[case(
    "def main() -> int64 {
        let a = 5;
        if a == 5 {
            return 0;
        }
        return 1;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let a = 5;
        if a < 5 {
            return 0;
        } 
        return 1;
    }",
    true
)]
#[case(
    "class Foo {
        inner: int64,
    }

    def main() -> int64 {
        let a = Foo { inner: 10 };
        return a;
    }",
    false
)]
#[case(
    "class Foo {
        inner: int64,
    }

    def inner_greater_then_10(self) for Foo -> bool {
        return self.inner > 10;
    }

    def new() for Foo -> Foo {
        return Foo { inner: 0 };
    }

    def main() -> int64 {
        let mut a = Foo::new();
        a.inner = 11;
        if a.inner_greater_then_10() {
            return 1;
        }
        return 0;
    }",
    true
)]
#[case(
    "class Foo<T> {
        inner: T,
    }

    def main() -> int64 {
        let a = Foo<int64> { inner: 10 };
        return a.inner;
    }",
    true
)]
#[case(
    "class Foo<T> {
        inner: T,
    }

    def new<T>(inner: T) for Foo -> Foo<T> {
        return Foo<T> { inner: inner };
    }

    def main() -> int64 {
        let a = Foo::new<int64>(10);
        return a.inner;
    }",
    true
)]
#[case(
    "class Foo<T> {
        inner: T,
    }

    def new<T>(inner: T) for Foo -> Foo<T> {
        return Foo<T> { inner: inner };
    }

    def echo<T>(self) for Foo<T> -> T {
        return self.inner;
    }

    def main() -> int64 {
        let a = Foo::new<int64>(10);
        let b = Foo::new<bool>(true);
        let ret = a.echo<int64>();
        return ret;
    }",
    true
)]
#[case(
    "class HashMap<K, V> {
        key: K,
        value: V,
    }
    
    def new<K, V>(key: K, value: V) for HashMap -> HashMap<K, V> {
        return HashMap<K, V> {
            key: key,
            value: value,
        };
    }

    def set<K, V>(self, key: K, value: V) for HashMap<K, V> -> int64 {
        self.key = key;
        self.value = value;
        return 0;
    }

    def main() -> int64 {
        let mut map = HashMap::new<int64, int64>(10, 10);
        map.set<int64, int64>(11, 11);
        return map.key;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let p = 5 as *int64 as int64 as *int64;
        return 0;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let p = 0 as *int64 as *float as *bool as int64;
        return p;
    }",
    true
)]
#[case(
    "class Foo {
        inner: int64,
    }

    def new() for Foo -> Foo {
        return Foo { inner: 0 };
    }

    def main() -> int64 {
        if Foo { inner: 10 }.inner == 10 {
            return 0;
        }

        return Foo::new().inner;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let num = {
            if false {
                return 10;
            }
            return 20;
        };

        return num;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let num = {
            if true {
                return 10;
            }
        };

        return num;
    }",
    false
)]
#[case(
    "def main() -> int64 {
        let num = {
            let foo = 10;
            return 0;
        };

        return foo;
    }",
    false
)]
#[case(
    "def main() -> int64 {
        let mut foo = 10;
        let num = {
            foo = foo + 1;
            return 0;
        };

        return foo;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let foo = 10;
        let num = {
            let bar = 21;
            return foo + bar;
        };

        return num;

    }",
    true
)]
#[case(
    "def main() -> int64 {
        let foo = 10;
        let num = {
            let foo = 49;
            return 0;
        };

        return foo + num;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let bar = 10;
        let foo = {
            return { return bar + { return bar + { return 10;} }; } + bar;
        };
        return foo;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        if true {
            return 0;
        } else {
            return 1;
        }
    }",
    true
)]
#[case(
    "def main() -> int64 {
        if true {
            return 0;
        } else {
            return 1;
        }
        return 2;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        if true {
            return 0;
        } else {
            
        }
        return 2;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let num = {
            if true {
                return 0;
            } else {
                return 1;
            }
        };

        return num;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let num = {
            if true {
                return 0;
            }
        }

        return num;
    }",
    false
)]
#[case(
    "def main() -> int64 {
        if true {
            return 0;
        }
    }",
    false
)]
#[case(
    "def main() -> int64 {
        if true {
            return 0;
        } 
        return 1;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1;
        } else {
            return 0.5;
        }
    }",
    false
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1.0;
        } else {
            return 0.0;
        }
        return 0;
    }",
    false
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1;
        } else {
            return 0.0;
        }
        return 0;
    }",
    false
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1;
        } else {
            return 2;
        }
        return 3;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1;
        }
    }",
    false
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1;
        } else if true {
            return 2;
        } else if true {
            return 3;
        } else {
            return 4;
        }
    }",
    true
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1;
        } else if true {

        } else if true {
            return 3;
        } else {
            return 4;
        }
    }",
    false
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1;
        } else if true {

        } else if true {
            return 3;
        } else {
            return 4;
        }

        return 5;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1;
        } else if true {

        } else if true {
            return 3.0;
        } else {
            return 4;
        }

        return 5;
    }",
    false
)]
#[case(
    "def main() -> int64 {
        if true {
            return 1;
        } else if true {
            return 2;
        } else if true {
            return 3;
        }
    }",
    false
)]
#[case(
    "def foo() {
        if true {

        }
    }
    def main() -> int64 {
        foo();
        return 0;
    }",
    true
)]
#[case(
    "def foo() {
        if true {

        } else if true {

        } else {

        }
    }
    def main() -> int64 {
        foo();
        return 0;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let s = \"test\";
        return 0;
    }",
    false
)]
#[case(
    "class String {
        dummy: int64,
    }

    def main() -> int64 {
        let s = \"test\";
        return 0;
    }",
    false
)]
#[case(
    "class String {
        dummy: int64,
    }

    def new() for String -> String {
        return String { dummy: 0 };
    }

    def with_capacity(cap: int64) for String -> String {
        return String { dummy: 0 };
    }

    def push_char(self, c: int8) for String -> int64 {
        return 0;
    }

    def main() -> int64 {
        let s = \"test\";
        return 0;
    }",
    true
)]
#[case(
    "class Foo {
        inner: int64,
    }

    def new() for Foo -> Foo {
        return Foo { inner: 0 };
    }

    def data(self) for Foo -> int64 {
        return self.inner;
    }

    def times_2(n: int64) -> int64 {
        return 2 * n;
    }

    def foo_add(self, other: Foo) for Foo -> Foo {
        let mut i = 0;
        while i < other.inner {
            self.inner = times_2(self.inner);
            i = i + 1;
        }

        if other.inner == self.inner {
            return Foo { inner: 0 };
        }

        return Foo { inner: self.inner + other.inner };
    }

    def main() -> int64 {
        let a = Foo::new();
        let b = Foo::new();
        if a.inner == b.inner {
            return 0;
        }
        return 1;
    }",
    true
)]
#[case(
    "class Foo<T> {
        inner: T,
    }

    def inner<T>(self) for Foo -> T {
        return self.inner;
    }

    def main() -> int64 {
        let a = Foo<int64> {
            inner: 10
        };

        let b = a.inner<int64>();

        return b;
    }",
    true
)]
#[case(
    "class Foo<T> {
        inner: T,
    }

    def inner<T>(self) for Foo -> T {
        return self.inner;
    }

    def main() -> int64 {
        let a = Foo<Foo<int64>> {
            inner: Foo<int64> {
                inner: 10,
            }
        };

        let inner = a.inner<Foo<int64>>().inner<int64>();

        return inner;

    }",
    true
)]
#[case(
    "class Foo<T> {
        inner: T,
    }

    def inner<T>(self) for Foo -> T {
        return self.inner;
    }

    def main() -> int64 {
        let a = Foo<Foo<int64>> {
            inner: Foo<int64> {
                inner: 10,
            }
        };

        
        let foo_arr = [a, a, a, a, a, a, a, a, a];
        
        let inner = foo_arr[0].inner<Foo<int64>>().inner<int64>();
        return inner;

    }",
    true
)]
#[case(
    "class Foo<T> {
        inner: T,
    }

    def inner<T>(self) for Foo -> T {
        return self.inner;
    }

    def main() -> int64 {
        let a = Foo<Foo<int64>> {
            inner: Foo<int64> {
                inner: 10,
            }
        };

        
        let foo_arr = [[[[[[[[[[a, a, a]]]]]]]]]];
        
        let inner = foo_arr[0][0][0][0][0][0][0][0][0][0].inner<Foo<int64>>().inner<int64>();
        return inner;

    }",
    true
)]
#[case(
    "def main() -> int64 {
        while true {
            return 0;
        }

        return 1;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        if true {
            if true {
                return 0;
            } 
        }

        return 1;
    }",
    true
)]
#[case(
    "def main() -> int64 {
        if true {
            if true {
                return 0;
            } 
        }
    }",
    false
)]
#[case(
    "def main() -> int64 {
        if true {
            if true {
                return 0;
            } 
        } else {
            return 1;
        }
    }",
    false
)]
#[case(
    "def main() -> int64 {
        let mut count = 0;
        let max = 20;

        while count < max {
            if count == 10 {
                return 10;
            }
        }

        return 0;
    }",
    true
)]
#[case(
    "def eq(self, other: int64) for int64 -> bool {
        return self == other;
    }
    
    def eq(self, other: bool) for bool -> bool {
        return self == other;
    }

    def main() -> int64 {
        return 0;
    }",
    true
)]
#[case(
    "def eq(self, other: int64) for int64 -> bool {
        return self == other;
    }
    
    def eq(self, other: int64) for int64 -> bool {
        return self == other;
    }

    def main() -> int64 {
        return 0;
    }",
    false
)]
#[case(
    "class Foo {
        inner: int64,
    }
    
    def main() -> int64 {
        let a = Foo { inner: 10 };
        let b = Foo { inner: 20 };

        let c = a + b;

        return c.inner;
    }",
    false
)]
#[case(
    "def is_three(self) for int64 -> bool {
        return self == 3;
    }
    
    def main() -> int64 {
        let a = 3;
        if a.is_three() {
            return 0;
        }
        return 1;
    }",
    true
)]
// idx muss pointer zurückgeben
#[case(
    "class Foo {
        inner: int64,
    }
    
    def idx(self, idx: int64) for Foo -> int64 {
        return self.inner;
    }

    def main() -> int64 {
        let mut a = Foo { inner: 10 };
        a[0] = 20;
        return a[0];
    }",
    false
)]
#[case(
    "class Foo {
        inner: int64,
    }
    
    def idx(self, idx: int64) for Foo -> *int64 {
        return &self.inner;
    }

    def main() -> int64 {
        let mut a = Foo { inner: 10 };
        a[0] = 20;
        return a[0];
    }",
    true
)]
#[case(
    "def main() -> int64 {
        let list = list[1, 2, 3];
        return list[0];
    };",
    false
)]
#[case(
    "class List<T> {
        inner: *T,
    }

    def new<T>() for List -> List<T> {
        return List { inner: 0 as *T };
    }

    def push<T>(self, elem: T) for List<T> -> int64 {
        return 0;
    }

    def main() -> int64 {
        let mut list = list![1, 2, 3];
        return 0;
    }",
    true
)]
#[case(
    "class List<T> {
        inner: *T,
    }
    
    def new<T>() for List -> List<T> {
        return List { inner: 0 as *T };
    }

    def push<T>(self, elem: T) for List<T> -> int64 {
        return 0;
    }

    def main() -> int64 {
        let my_list = list![1, 2, 3];

        return 0;
    }",
    true
)]
// nested list
#[case(
    "class List<T> {
        inner: *T,
    }
    
    def new<T>() for List -> List<T> {
        return List { inner: 0 as *T };
    }

    def push<T>(self, elem: T) for List<T> -> int64 {
        return 0;
    }

    def main() -> int64 {
        let my_list = list![list![1, 2, 3], list![4, 5, 6]];

        return 0;
    }",
    true
)]
#[case(
    "
    def main() -> int64 {
        let a = 10;
        let b = 20;

        if a == 10 && b == 20 {
            return 0;
        }

        return 1;
    }
    ",
    true
)]
#[case(
    "class Foo<T> {
        inner: T,
    }
    
    def test(self) for Foo<int64> -> int64 {
        return 0;
    }

    def test(self) for Foo<bool> -> int64 {
        return 1;
    }
    
    def inner<T>(self) for Foo<T> -> T {
        return self.inner;
    }
    
    def main() -> int64 {
        let f = Foo<int64>{ inner: 42 };
    
        f.test();
    
        let ret_val = f.inner<int64>();
        return ret_val;
    }",
    true
)]
#[case(
    "
    #asdfasf
    class Foo<T> { #asdfasdf
        inner: int64, #adsfasdf
    } #asdfasdf
    
    def main() -> int64 {
        return 0; #adsfasdf
    }
    #ads
    ",
    true
)]
#[case(
    "def main() -> int64 {
        # return 10;
        return 0; # ok
    }",
    true
)]
#[case(
    "# adsfasfasdf
    def main() -> int64 {
        # return 10;
    }",
    false
)]
#[case(
    "def main() -> # int64 {
        return 0;
    }",
    false
)]
#[case(
    "def main#() -> int64 {
        return 0;
    }",
    false
)]
#[case(
    "class Bar {
        inner: int64,
    }

    def get_inner(self) for Bar -> int64 {
        return self.inner;
    }

    class Foo {
        inner: Bar,
    }
    
    
    def main() -> int64 {
        let f = Foo { inner: Bar { inner: 42 } };
    
        let ret_val = f.inner.get_inner();
        return ret_val;   
    }",
    true
)]

fn test_compile(#[case] code: &'static str, #[case] should_compile: bool) {
    let res = Compiler::compile(code, None);
    if should_compile {
        res.as_ref().unwrap();
    }

    assert!(res.is_ok() == should_compile, "code: \"{}\n\"", code);
}
