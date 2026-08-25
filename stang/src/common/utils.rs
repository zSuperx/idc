#![allow(unused)]

pub fn align_n(x: i128, n: usize) -> i128 {
    (x + n as i128) & !n as i128
}

/// Given `a`, `b`, and a function that evaluates to a `bool`, returns:
/// - `Some((t, f))`: where `t` represents the choice that evaluated to `true`
/// - `None`: if `cond(a) == cond(b)`
pub fn select<T>(a: T, b: T, cond: impl Fn(&T) -> bool) -> Option<(T, T)> {
    let ca = cond(&a);
    let cb = cond(&b);
    if ca ^ cb {
        if ca { Some((a, b)) } else { Some((b, a)) }
    } else {
        None
    }
}

#[macro_export]
macro_rules! die {
    ($($fmtargs:tt)*) => {{
        #[cfg(debug_assertions)]
        eprintln!("=== Invoked from {}:{}:{} ===", file!(), line!(), column!());
        eprintln!($($fmtargs)*);
        ::std::process::exit(1);
    }};
}
