use ndarray::Array3;
use numpy::{IntoPyArray, PyArray3};
use pyo3::exceptions;
use pyo3::prelude::*;
use v4l::{Device, FourCC};
use v4l::buffer::Type;
use v4l::io::traits::{OutputStream, Stream};
use v4l::prelude::MmapStream;
use v4l::video::Capture;

#[pyclass]
struct Camera {
    stream: v4l::io::mmap::Stream<'static>,
    resolution: (usize, usize),
}

#[pymethods]
impl Camera {
    #[new]
    fn new(camera_id: usize, resolution: Option<(u32, u32)>) -> PyResult<Self> {
        let device = Device::new(camera_id)?;

        // Enable RGB conversion and set resolution
        let mut fmt = device.format()?;
        fmt.fourcc = FourCC::new(b"RGB3");
        if let Some(resolution) = resolution {
            fmt.width = resolution.0;
            fmt.height = resolution.1;
        }
        device.set_format(&fmt)?;

        // Read and store the actual resolution
        fmt = device.format()?;

        Ok(Camera {
            stream: MmapStream::with_buffers(&device, Type::VideoCapture, 4)?,
            resolution: (fmt.width as usize, fmt.height as usize),
        })
    }

    fn read<'py>(&mut self, py: Python<'py>) -> PyResult<&'py PyArray3<u8>> {
        let (buffer, _metadata) = self.stream.next()?;
        let frame = Array3::<u8>::from_shape_vec(
            (self.resolution.1, self.resolution.0, 3), buffer.to_vec()
        ).or(Err(exceptions::PyRuntimeError::new_err("Received incomplete frame")))?;
        Ok(frame.into_pyarray(py))
    }

    fn release(&mut self) -> PyResult<()> {
        self.stream.stop()?;
        Ok(())
    }
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn pyv4l2camera(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Camera>()?;
    Ok(())
}
