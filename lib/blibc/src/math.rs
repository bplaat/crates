macro_rules! unary {
    ($name:ident, $kind:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(value: $kind) -> $kind {
            libm::$name(value)
        }
    };
}

macro_rules! binary {
    ($name:ident, $kind:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(first: $kind, second: $kind) -> $kind {
            libm::$name(first, second)
        }
    };
}

macro_rules! ternary {
    ($name:ident, $kind:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn $name(first: $kind, second: $kind, third: $kind) -> $kind {
            libm::$name(first, second, third)
        }
    };
}

unary!(acos, f64);
unary!(acosf, f32);
unary!(acosh, f64);
unary!(acoshf, f32);
unary!(asin, f64);
unary!(asinf, f32);
unary!(asinh, f64);
unary!(asinhf, f32);
unary!(atan, f64);
binary!(atan2, f64);
binary!(atan2f, f32);
unary!(atanf, f32);
unary!(atanh, f64);
unary!(atanhf, f32);
unary!(cbrt, f64);
unary!(cbrtf, f32);
unary!(ceil, f64);
unary!(ceilf, f32);
binary!(copysign, f64);
binary!(copysignf, f32);
unary!(cos, f64);
unary!(cosf, f32);
unary!(cosh, f64);
unary!(coshf, f32);
unary!(erf, f64);
unary!(erfc, f64);
unary!(erfcf, f32);
unary!(erff, f32);
unary!(exp, f64);
unary!(exp10, f64);
unary!(exp10f, f32);
unary!(exp2, f64);
unary!(exp2f, f32);
unary!(expf, f32);
unary!(expm1, f64);
unary!(expm1f, f32);
unary!(fabs, f64);
unary!(fabsf, f32);
binary!(fdim, f64);
binary!(fdimf, f32);
unary!(floor, f64);
unary!(floorf, f32);
ternary!(fma, f64);
ternary!(fmaf, f32);
binary!(fmax, f64);
binary!(fmaxf, f32);
binary!(fmaximum, f64);
binary!(fmaximum_num, f64);
binary!(fmaximum_numf, f32);
binary!(fmaximumf, f32);
binary!(fmin, f64);
binary!(fminf, f32);
binary!(fminimum, f64);
binary!(fminimum_num, f64);
binary!(fminimum_numf, f32);
binary!(fminimumf, f32);
binary!(fmod, f64);
binary!(fmodf, f32);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn frexp(value: f64, exponent: *mut i32) -> f64 {
    let (fraction, exponent_value) = libm::frexp(value);
    unsafe { exponent.write(exponent_value) };
    fraction
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn frexpf(value: f32, exponent: *mut i32) -> f32 {
    let (fraction, exponent_value) = libm::frexpf(value);
    unsafe { exponent.write(exponent_value) };
    fraction
}

binary!(hypot, f64);
binary!(hypotf, f32);

#[unsafe(no_mangle)]
pub extern "C" fn ilogb(value: f64) -> i32 {
    libm::ilogb(value)
}

#[unsafe(no_mangle)]
pub extern "C" fn ilogbf(value: f32) -> i32 {
    libm::ilogbf(value)
}

unary!(j0, f64);
unary!(j0f, f32);
unary!(j1, f64);
unary!(j1f, f32);

#[unsafe(no_mangle)]
pub extern "C" fn jn(order: i32, value: f64) -> f64 {
    libm::jn(order, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn jnf(order: i32, value: f32) -> f32 {
    libm::jnf(order, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn ldexp(value: f64, exponent: i32) -> f64 {
    libm::ldexp(value, exponent)
}

#[unsafe(no_mangle)]
pub extern "C" fn ldexpf(value: f32, exponent: i32) -> f32 {
    libm::ldexpf(value, exponent)
}

unary!(lgamma, f64);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lgamma_r(value: f64, sign: *mut i32) -> f64 {
    let (result, sign_value) = libm::lgamma_r(value);
    unsafe { sign.write(sign_value) };
    result
}

unary!(lgammaf, f32);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn lgammaf_r(value: f32, sign: *mut i32) -> f32 {
    let (result, sign_value) = libm::lgammaf_r(value);
    unsafe { sign.write(sign_value) };
    result
}

unary!(log, f64);
unary!(log10, f64);
unary!(log10f, f32);
unary!(log1p, f64);
unary!(log1pf, f32);
unary!(log2, f64);
unary!(log2f, f32);
unary!(logf, f32);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn modf(value: f64, integer: *mut f64) -> f64 {
    let (fraction, integer_value) = libm::modf(value);
    unsafe { integer.write(integer_value) };
    fraction
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn modff(value: f32, integer: *mut f32) -> f32 {
    let (fraction, integer_value) = libm::modff(value);
    unsafe { integer.write(integer_value) };
    fraction
}

binary!(nextafter, f64);
binary!(nextafterf, f32);
binary!(pow, f64);
binary!(powf, f32);
binary!(remainder, f64);
binary!(remainderf, f32);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn remquo(first: f64, second: f64, quotient: *mut i32) -> f64 {
    let (remainder, quotient_value) = libm::remquo(first, second);
    unsafe { quotient.write(quotient_value) };
    remainder
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn remquof(first: f32, second: f32, quotient: *mut i32) -> f32 {
    let (remainder, quotient_value) = libm::remquof(first, second);
    unsafe { quotient.write(quotient_value) };
    remainder
}

unary!(rint, f64);
unary!(rintf, f32);
unary!(round, f64);
unary!(roundeven, f64);
unary!(roundevenf, f32);
unary!(roundf, f32);

#[unsafe(no_mangle)]
pub extern "C" fn scalbn(value: f64, exponent: i32) -> f64 {
    libm::scalbn(value, exponent)
}

#[unsafe(no_mangle)]
pub extern "C" fn scalbnf(value: f32, exponent: i32) -> f32 {
    libm::scalbnf(value, exponent)
}

unary!(sin, f64);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sincos(value: f64, sine: *mut f64, cosine: *mut f64) {
    let (sine_value, cosine_value) = libm::sincos(value);
    unsafe {
        sine.write(sine_value);
        cosine.write(cosine_value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sincosf(value: f32, sine: *mut f32, cosine: *mut f32) {
    let (sine_value, cosine_value) = libm::sincosf(value);
    unsafe {
        sine.write(sine_value);
        cosine.write(cosine_value);
    }
}

unary!(sinf, f32);
unary!(sinh, f64);
unary!(sinhf, f32);
unary!(sqrt, f64);
unary!(sqrtf, f32);
unary!(tan, f64);
unary!(tanf, f32);
unary!(tanh, f64);
unary!(tanhf, f32);
unary!(tgamma, f64);
unary!(tgammaf, f32);
unary!(trunc, f64);
unary!(truncf, f32);
unary!(y0, f64);
unary!(y0f, f32);
unary!(y1, f64);
unary!(y1f, f32);

#[unsafe(no_mangle)]
pub extern "C" fn yn(order: i32, value: f64) -> f64 {
    libm::yn(order, value)
}

#[unsafe(no_mangle)]
pub extern "C" fn ynf(order: i32, value: f32) -> f32 {
    libm::ynf(order, value)
}
