/*
This module generates code to try efficiently convert some arbitrary `T: 'static` into
a `Primitive`.

In the future when `min_specialization` is stabilized we could use it instead and avoid needing
the `'static` bound altogether.
*/

use crate::internal::Primitive;

pub(super) fn from_any<'v, T: ?Sized + 'static>(value: &'v T) -> Option<Primitive<'v>> {
    // When we're on `nightly`, we can use const type ids
    #[cfg(value_bag_capture_const_type_id)]
    {
        use crate::std::any::TypeId;

        macro_rules! to_primitive {
            ($($ty:ty : ($const_ident:ident, $option_ident:ident),)*) => {
                trait ToPrimitive<'a>
                where
                    Self: 'static,
                {
                    const CALL: fn(&'_ &'a Self) -> Option<Primitive<'a>> = {
                        $(
                            const $const_ident: TypeId = TypeId::of::<$ty>();
                            const $option_ident: TypeId = TypeId::of::<Option<$ty>>();
                        )*

                        const STR: TypeId = TypeId::of::<str>();

                        match TypeId::of::<Self>() {
                            $(
                                $const_ident => |v| Some(Primitive::from(unsafe { *(*v as *const Self as *const $ty) })),
                                $option_ident => |v| Some({
                                    let v = unsafe { *(*v as *const Self as *const Option<$ty>) };
                                    match v {
                                        Some(v) => Primitive::from(v),
                                        None => Primitive::None,
                                    }
                                }),
                            )*

                            STR => |v| Some(Primitive::from(unsafe { &**(v as *const &'a Self as *const &'a str) })),

                            _ => |_| None,
                        }
                    };

                    fn to_primitive(&'a self) -> Option<Primitive<'a>> {
                        (Self::CALL)(&self)
                    }
                }

                impl<'a, T: ?Sized + 'static> ToPrimitive<'a> for T {}
            }
        }

        // NOTE: The types here *must* match the ones used below when `const_type_id` is not available
        to_primitive![
            usize: (USIZE, OPTION_USIZE),
            u8: (U8, OPTION_U8),
            u16: (U16, OPTION_U16),
            u32: (U32, OPTION_U32),
            u64: (U64, OPTION_U64),
            u128: (U128, OPTION_U128),

            isize: (ISIZE, OPTION_ISIZE),
            i8: (I8, OPTION_I8),
            i16: (I16, OPTION_I16),
            i32: (I32, OPTION_I32),
            i64: (I64, OPTION_I64),
            i128: (I128, OPTION_I128),

            f32: (F32, OPTION_F32),
            f64: (F64, OPTION_F64),

            char: (CHAR, OPTION_CHAR),
            bool: (BOOL, OPTION_BOOL),

            &'static str: (STATIC_STR, OPTION_STATIC_STR),
            // We deal with `str` separately because it's unsized
            // str: (STR),
        ];

        value.to_primitive()
    }

    // When we're not on `nightly`, use the ctor crate
    #[cfg(value_bag_capture_ctor)]
    {
        #![allow(unused_unsafe)]

        use ctor::ctor;

        use crate::std::{
            any::{Any, TypeId},
            cmp::Ordering,
            marker::PhantomData,
        };

        // From: https://github.com/servo/rust-quicksort
        // We use this algorithm instead of the standard library's `sort_by` because it
        // works in no-std environments
        fn quicksort_helper<T, F>(arr: &mut [T], left: isize, right: isize, compare: &F)
        where
            F: Fn(&T, &T) -> Ordering,
        {
            if right <= left {
                return;
            }

            let mut i: isize = left - 1;
            let mut j: isize = right;
            let mut p: isize = i;
            let mut q: isize = j;
            unsafe {
                let v: *mut T = &mut arr[right as usize];
                loop {
                    i += 1;
                    while compare(&arr[i as usize], &*v) == Ordering::Less {
                        i += 1
                    }
                    j -= 1;
                    while compare(&*v, &arr[j as usize]) == Ordering::Less {
                        if j == left {
                            break;
                        }
                        j -= 1;
                    }
                    if i >= j {
                        break;
                    }
                    arr.swap(i as usize, j as usize);
                    if compare(&arr[i as usize], &*v) == Ordering::Equal {
                        p += 1;
                        arr.swap(p as usize, i as usize)
                    }
                    if compare(&*v, &arr[j as usize]) == Ordering::Equal {
                        q -= 1;
                        arr.swap(j as usize, q as usize)
                    }
                }
            }

            arr.swap(i as usize, right as usize);
            j = i - 1;
            i += 1;
            let mut k: isize = left;
            while k < p {
                arr.swap(k as usize, j as usize);
                k += 1;
                j -= 1;
                assert!(k < arr.len() as isize);
            }
            k = right - 1;
            while k > q {
                arr.swap(i as usize, k as usize);
                k -= 1;
                i += 1;
                assert!(k != 0);
            }

            quicksort_helper(arr, left, j, compare);
            quicksort_helper(arr, i, right, compare);
        }

        fn quicksort_by<T, F>(arr: &mut [T], compare: F)
        where
            F: Fn(&T, &T) -> Ordering,
        {
            if arr.len() <= 1 {
                return;
            }

            let len = arr.len();
            quicksort_helper(arr, 0, (len - 1) as isize, &compare);
        }

        enum Void {}

        #[repr(transparent)]
        struct VoidRef<'a>(*const *const Void, PhantomData<&'a Void>);

        macro_rules! type_ids {
            ($($ty:ty,)*) => {
                [
                    $(
                        (
                            std::any::TypeId::of::<$ty>(),
                            (|v| unsafe {
                                // SAFETY: We verify the value is $ty before casting
                                let v = *(*v.0 as *const $ty);
                                Primitive::from(v)
                            }) as for<'a> fn(VoidRef<'a>) -> Primitive<'a>
                        ),
                    )*
                    $(
                        (
                            std::any::TypeId::of::<Option<$ty>>(),
                            (|v| unsafe {
                                // SAFETY: We verify the value is Option<$ty> before casting
                                let v = *(*v.0 as *const Option<$ty>);
                                if let Some(v) = v {
                                    Primitive::from(v)
                                } else {
                                    Primitive::None
                                }
                            }) as for<'a> fn(VoidRef<'a>) -> Primitive<'a>
                        ),
                    )*
                    (
                        std::any::TypeId::of::<str>(),
                        (|v| unsafe {
                            // SAFETY: We verify the value is str before casting
                            let v = &**(v.0 as *const *const str);
                            Primitive::from(v)
                        }) as for<'a> fn(VoidRef<'a>) -> Primitive<'a>
                    ),
                ]
            };
        }

        #[ctor]
        static TYPE_IDS: [(
            TypeId,
            for<'a> fn(VoidRef<'a>) -> Primitive<'a>,
        ); 35] = {
            // NOTE: The types here *must* match the ones used above when `const_type_id` is available
            let mut type_ids = type_ids![
                usize,
                u8,
                u16,
                u32,
                u64,
                u128,
                isize,
                i8,
                i16,
                i32,
                i64,
                i128,
                f32,
                f64,
                char,
                bool,
                &'static str,
                // We deal with `str` separately because it's unsized
            ];

            quicksort_by(&mut type_ids, |&(ref a, _), &(ref b, _)| a.cmp(b));

            type_ids
        };

        if let Ok(i) = TYPE_IDS.binary_search_by_key(&value.type_id(), |&(k, _)| k) {
            Some((TYPE_IDS[i].1)(VoidRef(&(value) as *const &'v T as *const *const Void, PhantomData)))
        } else {
            None
        }
    }

    // When we're not on `nightly` and aren't on a supported arch, we can't do capturing
    #[cfg(value_bag_capture_fallback)]
    {
        macro_rules! type_ids {
            ($($ty:ty,)*) => {
                |value| {
                    $(
                        if let Some(value) = (*value as &dyn std::any::Any).downcast_ref::<$ty>() {
                            return Some(Primitive::from(*value));
                        }
                    )*
                    $(
                        if let Some(value) = (*value as &dyn std::any::Any).downcast_ref::<Option<$ty>>() {
                            if let Some(value) = value {
                                return Some(Primitive::from(*value));
                            } else {
                                return Some(Primitive::None);
                            }
                        }
                    )*

                    None
                }
            };
        }

        let type_ids = type_ids![
            usize,
            u8,
            u16,
            u32,
            u64,
            u128,
            isize,
            i8,
            i16,
            i32,
            i64,
            i128,
            f32,
            f64,
            char,
            bool,
            &'static str,
        ];

        (type_ids)(&value)
    }
}
