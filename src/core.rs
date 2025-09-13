use crate::kinda_sorta_dangling::KindaSortaDangling;
use stable_deref_trait::StableDeref;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};

pub trait Yokeable {
    type Output<'a>
    where
        Self: 'a;
}

pub type Output<'a, Y> = <Y as Yokeable>::Output<'a>;

pub struct Shared;
pub struct Mutable;

pub struct YokeGen<'a, Y: Yokeable + 'a, C, K = Shared> {
    // yokeable: exists<'a> Output<'a, Y>
    yokeable: KindaSortaDangling<Output<'a, Y>>, //'static dangling,
    cart: KindaSortaDangling<C>,
    kind: PhantomData<K>,
}

pub type Yoke<Y, C, K = Shared> = YokeGen<'static, Y, C, K>;

pub struct Ref<T: ?Sized>(PhantomData<T>);
pub struct RefMut<T: ?Sized>(PhantomData<T>);
impl<T: ?Sized> Yokeable for Ref<T> {
    type Output<'a>
        = &'a T
    where
        Self: 'a;
}

impl<T: ?Sized> Yokeable for RefMut<T> {
    type Output<'a>
        = &'a mut T
    where
        Self: 'a;
}

impl<'a, C: StableDeref> YokeGen<'a, Ref<C::Target>, C> {
    pub fn new(cart: C) -> Self {
        let cart = KindaSortaDangling::new(cart);
        Self {
            yokeable: unsafe { mem::transmute(KindaSortaDangling::new(cart.deref().deref())) },
            cart,
            kind: PhantomData,
        }
    }
}

impl<'a, Y: Yokeable, C: StableDeref> YokeGen<'a, Y, C> {
    pub fn backing_cart(&self) -> &C {
        &self.cart
    }
}

impl<'a, C: StableDeref + DerefMut> YokeGen<'a, RefMut<C::Target>, C, Mutable> {
    pub fn new_mut(cart: C) -> Self {
        let mut cart = KindaSortaDangling::new(cart);
        Self {
            yokeable: unsafe {
                mem::transmute(KindaSortaDangling::new(cart.deref_mut().deref_mut()))
            },
            cart,
            kind: PhantomData,
        }
    }
}

pub trait WithMutFn<'a, Y: Yokeable> {
    type Output;
    fn call<'b>(self, y: &'a mut Output<'b, Y>) -> Self::Output;
}

impl<'a, Y: Yokeable, F: for<'b> FnOnce(&'a mut Output<'b, Y>) -> O, O> WithMutFn<'a, Y> for F {
    type Output = O;

    fn call<'b>(self, y: &'a mut Output<'b, Y>) -> Self::Output {
        self(y)
    }
}

impl<'u, Y: Yokeable, C, K> YokeGen<'u, Y, C, K> {
    pub fn with_mut<'a, F, O>(&'a mut self, f: F) -> O
    where
        F: for<'b> FnOnce(&'a mut Output<'b, Y>) -> O,
    {
        f(self.yokeable.deref_mut())
    }

    pub fn with_ref<'a, F, O>(&'a self, f: F) -> O
    where
        F: for<'b> FnOnce(&'a Output<'b, Y>) -> O,
    {
        f(self.yokeable.deref())
    }

    pub fn try_map_or_cart<'u2, F, Y2: Yokeable, E>(
        self,
        f: F,
    ) -> Result<YokeGen<'u2, Y2, C, K>, (E, C)>
    where
        F: for<'b> FnOnce(Output<'b, Y>, PhantomData<&'b ()>) -> Result<Output<'b, Y2>, E>,
    {
        let t: KindaSortaDangling<Output<'_, Y>> = unsafe { mem::transmute(self.yokeable) };
        match f(t.into_inner(), PhantomData) {
            Ok(yokeable) => Ok(YokeGen {
                yokeable: unsafe { mem::transmute(KindaSortaDangling::new(yokeable)) },
                cart: self.cart,
                kind: self.kind,
            }),
            Err(err) => Err((err, self.cart.into_inner())),
        }
    }
}
