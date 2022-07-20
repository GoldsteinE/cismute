pub trait Branches<R, T, RefT, Args> {
    fn dispatch(self, val: RefT) -> Result<R, RefT>;
}

impl<R, T, RefT> Branches<R, T, RefT, ()> for ()
where
    T: 'static,
{
    #[inline(always)]
    fn dispatch(self, val: RefT) -> Result<R, RefT> {
        Err(val)
    }
}

macro_rules! impl_branches {
    (;;;) => {};
    ($u:ident $($us:ident)*; $refU:ident $($refUs:ident)*; $f:ident $($fs:ident)*;) => {
        impl_branches!($($us)*; $($refUs)*; $($fs)*;);

        impl<'a, R, T, RefT, $u, $($us,)* $refU, $($refUs,)* $f, $($fs,)*>
            Branches<R, T, RefT, (($u, $refU), $(($us, $refUs),)*)>
            for ($f, $($fs,)*)
        where
            T: 'static,
            $u: 'static,
            RefT: $crate::Cismutable<'a, T, $u, $refU>,
            $f: FnOnce($refU) -> R,
        $(
            $us: 'static,
            RefT: $crate::Cismutable<'a, T, $us, $refUs>,
            $fs: FnOnce($refUs) -> R,
        )*
        {
            #[inline(always)]
            #[allow(non_snake_case)]
            fn dispatch(self, val: RefT) -> Result<R, RefT> {
                let ($f, $($fs,)*) = self;
                let val = match $crate::value::<'a, T, $u, RefT, $refU>(val) {
                    Ok(val) => return Ok($f(val)),
                    Err(val) => val,
                };
                $(
                    let val = match $crate::value::<'a, T, $us, RefT, $refUs>(val) {
                        Ok(val) => return Ok($fs(val)),
                        Err(val) => val,
                    };
                )*
                Err(val)
            }
        }
    };
}

impl_branches!(
    U00 U01 U02 U03 U04 U05 U06 U07
    U08 U09 U10 U11 U12 U13 U14 U15
    U16 U17 U18 U19 U20 U21 U22 U23
    U24 U25 U26 U27 U28 U29 U30 U31;
    R00 R01 R02 R03 R04 R05 R06 R07
    R08 R09 R10 R11 R12 R13 R14 R15
    R16 R17 R18 R19 R20 R21 R22 R23
    R24 R25 R26 R27 R28 R29 R30 R31;
    F00 F01 F02 F03 F04 F05 F06 F07
    F08 F09 F10 F11 F12 F13 F14 F15
    F16 F17 F18 F19 F20 F21 F22 F23
    F24 F25 F26 F27 F28 F29 F30 F31;
);
