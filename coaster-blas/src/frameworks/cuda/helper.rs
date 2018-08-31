// Those macros should be removed when read()/read_only()/write() are refactored
// to return typed memory. For now they remove a lot of visual clutter and
// lessen probability of stupid mistakes.
macro_rules! read {
    ($x:ident, $slf:ident) => (
        try!($x.read($slf.device()))
    )
}

macro_rules! read_write {
    ($x:ident, $slf:ident) => (
        try!($x.read_write($slf.device()))
    )
}

macro_rules! write_only {
    ($x:ident, $slf:ident) => (
        try!($x.write_only($slf.device()))
    )
}

// trans! cannot be inlined into macros above, because `$mem` would become
// intermidiate variable and `*mut $t` will outlive it.
macro_rules! trans {
    ($mem:ident, $t:ident) => (
        unsafe { ::std::mem::transmute::<u64, *mut $t>(*$mem.id_c()) }
    )
}

macro_rules! exec {
    ($name:ident, $f:expr) => ({
        let res = $f;
        res.map_err(|_| PluginError::Operation(
            stringify!(Unable to execute operation $name)).into())
    })
}


#[macro_export]
macro_rules! iblas_asum_for_cuda {
    ($t:ident) => (
        fn asum(&self, x: &SharedTensor<$t>, result: &mut SharedTensor<$t>)
                -> Result<(), ::coaster::error::Error> {
            let n = x.desc().size() as i32;
            let x_mem = read!(x, self);
            let r_mem = write_only!(result, self);
            exec!(asum, CONTEXT.asum(
                trans!(x_mem, $t),
                trans!(r_mem, $t),
                n, None))
        }
    );
}

#[macro_export]
macro_rules! iblas_axpy_for_cuda {
    ($t:ident) => (
        fn axpy(&self, a: &SharedTensor<$t>, x: &SharedTensor<$t>,
                y: &mut SharedTensor<$t>)
                -> Result<(), ::coaster::error::Error> {
            let n = x.desc().size() as i32;
            let a_mem = read!(a, self);
            let x_mem = read!(x, self);
            let y_mem = read_write!(y, self);
            exec!(axpy, CONTEXT.axpy(
                trans!(a_mem, $t),
                trans!(x_mem, $t),
                trans!(y_mem, $t),
                n, None, None))
        }
    );
}

#[macro_export]
macro_rules! iblas_copy_for_cuda {
    ($t:ident) => (
        fn copy(&self, x: &SharedTensor<$t>, y: &mut SharedTensor<$t>)
                -> Result<(), ::coaster::error::Error> {
            let n = x.desc().size() as i32;
            let x_mem = read!(x, self);
            let y_mem = write_only!(y, self);
            exec!(copy, CONTEXT.copy(
                trans!(x_mem, $t),
                trans!(y_mem, $t),
                n, None, None))
        }
    );
}

#[macro_export]
macro_rules! iblas_dot_for_cuda {
    ($t:ident) => (
        fn dot(&self, x: &SharedTensor<$t>, y: &SharedTensor<$t>,
               result: &mut SharedTensor<$t>)
               -> Result<(), ::coaster::error::Error> {
            let n = x.desc().size() as i32;
            let x_mem = read!(x, self);
            let y_mem = read!(y, self);
            let r_mem = write_only!(result, self);
            exec!(dot, CONTEXT.dot(
                trans!(x_mem, $t),
                trans!(y_mem, $t),
                trans!(r_mem, $t),
                n, None, None))
        }
    );
}

#[macro_export]
macro_rules! iblas_nrm2_for_cuda {
    ($t:ident) => (
        fn nrm2(&self, x: &SharedTensor<$t>, result: &mut SharedTensor<$t>)
                -> Result<(), ::coaster::error::Error> {
            let n = x.desc().size() as i32;
            let x_mem = read!(x, self);
            let r_mem = write_only!(result, self);
            exec!(nrm2, CONTEXT.nrm2(
                trans!(x_mem, $t),
                trans!(r_mem, $t),
                n, None))
        }
    );
}

#[macro_export]
macro_rules! iblas_scal_for_cuda {
    ($t:ident) => (
        fn scal(&self, a: &SharedTensor<$t>, x: &mut SharedTensor<$t>)
                -> Result<(), ::coaster::error::Error> {
            let n = x.desc().size() as i32;
            let a_mem = read!(a, self);
            let x_mem = read_write!(x, self);
            exec!(scal, CONTEXT.scal(
                trans!(a_mem, $t),
                trans!(x_mem, $t),
                n, None))
        }
    );
}

#[macro_export]
macro_rules! iblas_swap_for_cuda {
    ($t:ident) => (
        fn swap(&self, x: &mut SharedTensor<$t>, y: &mut SharedTensor<$t>)
                -> Result<(), ::coaster::error::Error> {
            let n = x.desc().size() as i32;
            let x_mem = read_write!(x, self);
            let y_mem = read_write!(y, self);
            exec!(swap, CONTEXT.swap(
                trans!(x_mem, $t),
                trans!(y_mem, $t),
                n, None, None))
        }
    );
}

#[macro_export]
macro_rules! iblas_gemm_for_cuda {
    ($t:ident) => (
        fn gemm(&self,
                alpha: &SharedTensor<$t>,
                at: Transpose,
                a: &SharedTensor<$t>,
                bt: Transpose,
                b: &SharedTensor<$t>,
                beta: &SharedTensor<$t>,
                c: &mut SharedTensor<$t>
        ) -> Result<(), ::coaster::error::Error> {
            let c_desc = c.desc().clone();
            let alpha_mem = read!(alpha, self);
            let beta_mem = read!(beta, self);
            let a_mem = read!(a, self);
            let b_mem = read!(b, self);
            let c_mem = write_only!(c, self);

            let a_0 = a.desc()[0] as i32;
            let a_1 = a.desc().iter().skip(1).fold(1, |prod, i| prod * i) as i32;
            let b_0 = b.desc()[0] as i32;
            let b_1 = b.desc().iter().skip(1).fold(1, |prod, i| prod * i) as i32;
            let c_1 = c_desc.iter().skip(1).fold(1, |prod, i| prod * i) as i32;
            let n = match bt {
                Transpose::NoTrans => b_1,
                _ => b_0
            };
            let (m, k) = match at {
                Transpose::NoTrans => (a_0, a_1),
                _ => (a_1, a_0)
            };
            let lda = a_1;
            let ldb = b_1;
            let ldc = c_1;
            exec!(gemm, CONTEXT.gemm(
                ::cublas::api::Operation::from(bt),
                ::cublas::api::Operation::from(at),
                n, m, k,
                trans!(alpha_mem, $t),
                trans!(b_mem, $t), // matrix a and b are switched to make it work with row-major memory layout.
                ldb,
                trans!(a_mem, $t),
                lda,
                trans!(beta_mem, $t),
                trans!(c_mem, $t),
                ldc))
        }
    );
}
