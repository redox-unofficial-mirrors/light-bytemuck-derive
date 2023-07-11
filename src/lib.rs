#![forbid(unsafe_op_in_unsafe_fn)]
#![no_std]

#[macro_export]
macro_rules! pod_struct {
    {
        $struct_vis:vis struct $name:ident {
            $(
                $(#[$outer:meta])*
                $vis:vis $field:ident: $ty:ty,
            )*
        }
    } => {
        #[derive(Clone, Copy, Debug)]
        #[repr(C)]
        $struct_vis struct $name {
            $(
                $(#[$outer])*
                $vis $field: $ty,
            )*
        }

        // SAFETY: bytemuck requires (1) that the type is inhabited, and (2) that an all-zeroes
        // pattern is valid.
        //
        // The first condition is upheld as structs cannot themselves be uninhabited enums, and
        // since all individual fields are checked to be Zeroable, it cannot contain fields that
        // makes the struct effectively uninhabited.
        //
        // The second condition is similarly upheld by the fact that all fields need to be
        // zeroable, and this macro enforces no padding bytes exist.
        unsafe impl ::bytemuck::Zeroable for $name {}

        // SAFETY:
        //
        // - "the type must be inhabited", which is upheld since the struct cannot be an empty enum
        // itself, and all fields are required to be Pod too.
        // - "the type must allow any bit pattern", which similarly is upheld as this macro checks
        // every field allows any bit pattern (i.e. implements Pod), and no padding fields are
        // allowed to exist.
        // - "the type must not contain any uninit (or padding) bytes", which is upheld by checking
        // that the sum of the field sizes equals the size of the struct.
        // - "the type needs to have all fields also be Pod", which is enforced using the
        // `needs_traits` function.
        // - "the type needs to be repr(C) or repr(transparent)", upheld since repr(C) is always
        // set in the struct declaration.
        // - the last requirement forbids any form of interior mutability, i.e. that an &Self
        // cannot be used for anything but reads. This is enforced by checking that all fields are
        // Pod; interior mutability is only possible with UnsafeCell or references thereto, which
        // are not Pod, or with unsafe code, which is required to uphold that additional invariant.
        unsafe impl ::bytemuck::Pod for $name {}

        const _: () = {
            const SIZE: usize = ::core::mem::size_of::<$name>();
            const FIELD_SIZE_SUM: usize = 0 $(+ ::core::mem::size_of::<$ty>())*;

            #[allow(dead_code)]
            fn needs_traits<T: ::bytemuck::Zeroable + ::bytemuck::Pod + ::core::marker::Copy + 'static>() {}

            #[allow(dead_code)]
            unsafe fn _never_called() {
                // SAFETY: Never called!
                let _ = unsafe {
                    core::mem::transmute::<[u8; SIZE], [u8; FIELD_SIZE_SUM]>(core::mem::zeroed())
                };

                $(
                    needs_traits::<$ty>();
                )*

                needs_traits::<$name>();
            }

            // The bytemuck Pod docs do not mention anything about alignment, but as long as there
            // are no padding bytes, which has already been checked for, that is irrelevant.
        };
    }
}
