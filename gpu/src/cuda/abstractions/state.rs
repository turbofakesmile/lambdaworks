use crate::gpu::cuda::abstractions::{element::CUDAFieldElement, errors::CudaError};
use cudarc::{
    driver::{
        safe::{CudaSlice, DeviceSlice},
        CudaDevice, CudaFunction, DeviceRepr, LaunchAsync, LaunchConfig,
    },
    nvrtc::safe::Ptx,
};
use std::sync::Arc;

const STARK256_PTX: &str =
    include_str!("../../../../math/src/gpu/cuda/shaders/fields/stark256.ptx");

/// Structure for abstracting basic calls to a Metal device and saving the state. Used for
/// implementing GPU parallel computations in Apple machines.
pub struct CudaState {
    device: Arc<CudaDevice>,
}

impl CudaState {
    /// Creates a new CUDA state with the first GPU.
    pub(crate) fn new() -> Result<Self, CudaError> {
        let device =
            CudaDevice::new(0).map_err(|err| CudaError::DeviceNotFound(err.to_string()))?;
        let state = Self { device };

        // Load PTX libraries
        state.load_library(STARK256_PTX, "stark256")?;

        Ok(state)
    }

    fn load_library(&self, src: &str, mod_name: &str) -> Result<(), CudaError> {
        let functions = ["radix2_dit_butterfly"];
        self.device
            .load_ptx(Ptx::from_src(src), mod_name, &functions)
            .map_err(|err| CudaError::PtxError(err.to_string()))
    }

    fn get_function(&self, mod_name: &str, func_name: &str) -> Result<CudaFunction, CudaError> {
        self.device
            .get_func(mod_name, func_name)
            .ok_or_else(|| CudaError::FunctionError(func_name.to_string()))
    }

    /// Allocates a buffer in the GPU and copies `data` into it. Returns its handle.
    fn alloc_buffer_with_data<T: DeviceRepr>(&self, data: &[T]) -> Result<CudaSlice<T>, CudaError> {
        self.device
            .htod_sync_copy(data)
            .map_err(|err| CudaError::AllocateMemory(err.to_string()))
    }
}

impl<F: IsField> Radix2DitButterflyFunction<F> {
    fn new(
        device: Arc<CudaDevice>,
        function: CudaFunction,
        input: CudaSlice<CUDAFieldElement<F>>,
        twiddles: CudaSlice<CUDAFieldElement<F>>,
    ) -> Self {
        Self {
            device,
            function,
            input,
            twiddles,
        }
    }

    pub(crate) fn launch(
        &mut self,
        group_count: usize,
        group_size: usize,
    ) -> Result<(), CudaError> {
        let grid_dim = (group_count as u32, 1, 1); // in blocks
        let block_dim = ((group_size / 2) as u32, 1, 1);

        if block_dim.0 as usize > DeviceSlice::len(&self.twiddles) {
            return Err(CudaError::IndexOutOfBounds(
                block_dim.0 as usize,
                self.twiddles.len(),
            ));
        } else if (grid_dim.0 * block_dim.0) as usize > DeviceSlice::len(&self.input) {
            return Err(CudaError::IndexOutOfBounds(
                (grid_dim.0 * block_dim.0) as usize,
                self.input.len(),
            ));
        }

        let config = LaunchConfig {
            grid_dim,
            block_dim,
            shared_mem_bytes: 0,
        };
        unsafe {
            self.function
                .clone()
                .launch(config, (&mut self.input, &self.twiddles))
        }
        .map_err(|err| CudaError::Launch(err.to_string()))
    }

    pub(crate) fn retrieve_result(self) -> Result<Vec<FieldElement<F>>, CudaError> {
        let Self { device, input, .. } = self;
        let output = device
            .sync_reclaim(input)
            .map_err(|err| CudaError::RetrieveMemory(err.to_string()))?
            .into_iter()
            .map(FieldElement::from)
            .collect();

        Ok(output)
    }
}
