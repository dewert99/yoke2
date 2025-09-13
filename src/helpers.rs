use crate::core::{Mutable, Output, Ref, Yoke, YokeGen, Yokeable};
use stable_deref_trait::StableDeref;
use std::convert::Infallible;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub trait CovariantYokeable: Yokeable {
    fn cast_ref<'a, 'b>(x: &'a Output<'b, Self>) -> &'a Output<'a, Self>;
}

impl Yokeable for Infallible {
    type Output<'a> = Infallible;
}

impl<'u, Y: Yokeable, C, K> Debug for YokeGen<'u, Y, C, K>
where
    for<'a> Output<'a, Y>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.with_ref(|x| x.fmt(f))
    }
}

impl<'u, Y: Yokeable, C, K> YokeGen<'u, Y, C, K> {
    pub fn get<'a>(&'a self) -> &'a Output<'a, Y>
    where
        Y: CovariantYokeable,
    {
        self.with_ref::<_, &'a Output<'a, Y>>(Y::cast_ref)
    }

    pub fn try_map<'u2, F, Y2: Yokeable, E>(self, f: F) -> Result<YokeGen<'u2, Y2, C, K>, E>
    where
        F: for<'b> FnOnce(Output<'b, Y>, PhantomData<&'b ()>) -> Result<Output<'b, Y2>, E>,
    {
        self.try_map_or_cart(f).map_err(|(err, _)| err)
    }

    pub fn map<'u2, F, Y2: Yokeable>(self, f: F) -> YokeGen<'u2, Y2, C, K>
    where
        F: for<'b> FnOnce(Output<'b, Y>, PhantomData<&'b ()>) -> Output<'b, Y2>,
    {
        match self.try_map_or_cart(|x, p| Ok::<_, Infallible>(f(x, p))) {
            Ok(res) => res,
            Err((err, _)) => match err {},
        }
    }

    pub fn into_inner_cart<F, Out>(self, f: F) -> (Out, C)
    where
        F: for<'b> FnOnce(Output<'b, Y>) -> Out,
    {
        match self.try_map_or_cart::<_, Infallible, _>(|x, _| Err::<Infallible, _>(f(x))) {
            #[allow(unreachable_code)]
            Ok(err) => match err.with_ref(|x| *x) {},
            Err(res) => res,
        }
    }

    pub fn into_inner<F, Out>(self, f: F) -> Out
    where
        F: for<'b> FnOnce(Output<'b, Y>) -> Out,
    {
        self.into_inner_cart(f).0
    }

    pub fn into_cart(self) -> C {
        self.into_inner_cart(|_| {}).1
    }
}

impl<Y: Yokeable, C: StableDeref> Yoke<Y, C> {
    pub fn attach_to_cart<'b, F>(cart: C, f: F) -> Self
    where
        F: for<'a> FnOnce(&'a <C as Deref>::Target) -> Output<'a, Y>,
        C::Target: 'static,
    {
        Yoke::new(cart).map(|x, _| f(x))
    }

    pub fn try_attach_to_cart<F, E>(cart: C, f: F) -> Result<Self, E>
    where
        F: for<'a> FnOnce(&'a <C as Deref>::Target) -> Result<Output<'a, Y>, E>,
        C::Target: 'static,
    {
        YokeGen::new(cart).try_map(|x, _| f(x))
    }
}

impl<Y: Yokeable, C: StableDeref + DerefMut> Yoke<Y, C, Mutable> {
    pub fn attach_to_cart_mut<F>(cart: C, f: F) -> Self
    where
        F: for<'a> FnOnce(&'a mut <C as Deref>::Target) -> Output<'a, Y>,
        C::Target: 'static,
    {
        Yoke::new_mut(cart).map(|x, _| f(x))
    }
    pub fn try_attach_to_cart_mut<F, E>(cart: C, f: F) -> Result<Self, E>
    where
        F: for<'a> FnOnce(&'a mut <C as Deref>::Target) -> Result<Output<'a, Y>, E>,
        C::Target: 'static,
    {
        Yoke::new_mut(cart).try_map(|x, _| f(x))
    }
}

impl<T: 'static + ?Sized> CovariantYokeable for Ref<T> {
    fn cast_ref<'a, 'b>(x: &'a &'b T) -> &'a &'a T {
        x
    }
}
