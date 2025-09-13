Inspired by [yoke](https://docs.rs/yoke/latest/yoke/index.html), but the [`Yokeable`] trait is
safe to implement and doesn't require covariance. The [`Yoke::get`] method does require the
[`CovariantYokeable`] trait, but that trait is also safe, if the implementation panics
[`Yoke::get`] also panics. This crate also allows getting mutable  access to the cart during
construction in exchange for losing access to the cart later.

### Examples

#### Mutable access to cart
```rust
use yoke2::{CovariantYokeable, Yoke, YokeMut, Yokeable, Output};

struct MutRefPair<'a>(&'a mut u32, &'a mut u32);

impl Yokeable for MutRefPair<'static> {
  type Output<'a> = MutRefPair<'a> where Self: 'a;
}

impl CovariantYokeable for MutRefPair<'static> {
  fn cast_ref<'a, 'b>(x: &'a Output<'b, Self>) -> &'a Output<'a, Self> {
      x
   }
}

let b = Box::new([4, 2]);

let mut y = YokeMut::<MutRefPair<'static>, _>::attach_to_cart_mut(b, |[a, b]| MutRefPair(a, b));
let x = y.get();
assert_eq!(*x.0, 4);
assert_eq!(*x.1, 2);
y.with_mut(|x| {
  *x.0 = 0;
  *x.1 = 5;
});
let c = y.into_cart();
assert_eq!(*c, [0, 5]);
```

#### Invalid implementation of `CovariantYokeable` fails to compile
```compile_fail
use yoke2::{CovariantYokeable, Yoke, YokeMut, Yokeable, Output};
use core::cell::Cell;

struct CellRef<'a>(Cell<&'a u32>);

impl Yokeable for CellRef<'static> {
  type Output<'a> = CellRef<'a> where Self: 'a;
}

impl CovariantYokeable for CellRef<'static> {
  fn cast_ref<'a, 'b>(x: &'a Output<'b, Self>) -> &'a Output<'a, Self> {
      x
   }
}
```

#### Invalid usage of `with_ref` fails to compile
```compile_fail
use yoke2::{CovariantYokeable, Yoke, YokeMut, Yokeable, Output};
use core::cell::Cell;

struct CellRef<'a>(Cell<&'a u32>);

impl Yokeable for CellRef<'static> {
  type Output<'a> = CellRef<'a> where Self: 'a;
}

let y = Yoke::<CellRef, _>::attach_to_cart(Box::new(5), |x| CellRef(Cell::new(x)));
let n = 6;
y.with_ref(|x| x.0.set(&n));
```

#### Panic-ing implementation of `CovariantYokeable` causes `get` to panic

```should_panic
use yoke2::{CovariantYokeable, Yoke, YokeMut, Yokeable, Output};
use core::cell::Cell;

struct CellRef<'a>(Cell<&'a u32>);

impl Yokeable for CellRef<'static> {
  type Output<'a> = CellRef<'a> where Self: 'a;
}

impl CovariantYokeable for CellRef<'static> {
  fn cast_ref<'a, 'b>(x: &'a Output<'b, Self>) -> &'a Output<'a, Self> {
      panic!()
   }
}

let y = Yoke::<CellRef, _>::attach_to_cart(Box::new(5), |x| CellRef(Cell::new(x)));
let n = 6;
y.get().0.set(&n);
```

#### Valid usage of non covariant `Yokeable`
```rust
use yoke2::{CovariantYokeable, Yoke, YokeMut, Yokeable, Output};
use core::cell::Cell;

struct CellRefPair<'a>(Cell<&'a u32>, Cell<&'a u32>);

impl Yokeable for CellRefPair<'static> {
  type Output<'a> = CellRefPair<'a> where Self: 'a;
}

let y = Yoke::<CellRefPair, _>::attach_to_cart(Box::new([4, 2]), |[x, y]| CellRefPair(Cell::new(x), Cell::new(y)));
y.with_ref(|x| x.0.swap(&x.1));
assert_eq!(y.with_ref(|x| [x.0.get(), x.1.get()]), [&2, &4]);
```