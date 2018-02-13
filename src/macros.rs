// TODO: Consider checking for existence of these upon startup, not on each call.
macro_rules! raw_fn {
    ($env:ident, $name:ident) => {
        (*$env.raw).$name.ok_or($crate::error::Error {
            kind: $crate::error::ErrorKind::CoreFnMissing(format!("{}", stringify!($name)))
        })
    };
}

macro_rules! raw_call {
    ($env:ident, $name:ident $(, $args:expr)*) => {
        {
            let result = unsafe {
                let $name = raw_fn!($env, $name)?;
                $name($env.raw $(, $args)*)
            };
            $crate::error::HandleExit::handle_exit($env, result)
        }
    };
}

/// Note: Some functions in emacs-module.h are critically important, like those that support error
/// reporting to Emacs. If they are missing, the only sensible thing to do is crashing. Use this
/// macro to call them instead of [`raw_call!`].
macro_rules! critical {
    ($env:ident, $name:ident $(, $args:expr)*) => {
        unsafe {
            let $name = raw_fn!($env, $name)
                .expect(&format!("Required function {} cannot be found", stringify!($name)));
            $name($env.raw $(, $args)*)
        }
    };
}

// TODO: Export this.
macro_rules! call_lisp {
    ($env:ident, $name:expr $(, $arg:expr)*) => {
        {
            let symbol: $crate::Value = $env.intern($name)?;
            let args = &mut [$($arg.raw,)*];
            raw_call!($env, funcall, symbol.raw, args.len() as ::libc::ptrdiff_t, args.as_mut_ptr())
        }
    };
}

#[macro_export]
macro_rules! emacs_plugin_is_GPL_compatible {
    () => {
        /// This states that the module is GPL-compliant.
        /// Emacs won't load the module if this symbol is undefined.
        #[no_mangle]
        #[allow(non_upper_case_globals)]
        pub static plugin_is_GPL_compatible: libc::c_int = 0;
    }
}

/// Declares `emacs_module_init` and `emacs_rs_module_init`, by wrapping the given function, whose
/// signature must be `fn(&Env) -> Result<Value>`.
#[macro_export]
macro_rules! emacs_module_init {
    ($init:ident) => {
        /// Entry point for Emacs's module loader.
        #[no_mangle]
        pub extern "C" fn emacs_module_init(raw: *mut $crate::raw::emacs_runtime) -> ::libc::c_int {
            match $init(&$crate::Env::from(raw)) {
                Ok(_) => 0,
                // TODO: Try to signal error to Emacs as well
                Err(_) => 1,
            }
        }

        // TODO: Exclude this in release build.
        /// Entry point for live-reloading (by `rs-module`) during development.
        #[no_mangle]
        pub extern "C" fn emacs_rs_module_init(raw: *mut $crate::raw::emacs_env) -> ::libc::c_int {
            match $init(&$crate::Env::from(raw)) {
                Ok(_) => 0,
                // TODO: Try to signal error to Emacs as well
                Err(_) => 1,
            }
        }
    };
}

#[macro_export]
macro_rules! emacs_subrs {
    ($($name:ident -> $extern_name:ident;)*) => {
        $(
            #[allow(non_snake_case, unused_variables)]
            unsafe extern "C" fn $extern_name(env: *mut $crate::raw::emacs_env,
                                              nargs: ::libc::ptrdiff_t,
                                              args: *mut $crate::raw::emacs_value,
                                              data: *mut ::libc::c_void) -> $crate::raw::emacs_value {
                let env = $crate::Env::from(env);
                // TODO: Mark Value as repr(transparent) once it's available, and use this.
                // let args: *mut $crate::Value = ::std::mem::transmute(args);
                // let args: &mut [$crate::Value] = ::std::slice::from_raw_parts_mut(args, nargs as usize);
                // let result = $name(&env, args, data);
                let args: &[$crate::raw::emacs_value] = ::std::slice::from_raw_parts(args, nargs as usize);
                let mut args: Vec<$crate::Value> = args.iter().map(|v| (*v).into()).collect();
                let result = $name(&env, &mut args, data);
                $crate::error::TriggerExit::maybe_exit(&env, result)
            }
        )*
    };
}
