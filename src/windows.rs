use std::ffi::OsStr;
use std::mem::ManuallyDrop;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{FromRawHandle, RawHandle};
use std::path::Path;
use std::pin::Pin;

use serialport::windows::COMPort;
use serialport::{SerialPort, SerialPortSettings};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::windows::named_pipe::NamedPipeClient;
use winapi::um::commapi::SetCommTimeouts;
use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::winbase::{COMMTIMEOUTS, FILE_FLAG_OVERLAPPED};
use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, GENERIC_READ, GENERIC_WRITE, HANDLE};

pub struct Serial {
    inner: ManuallyDrop<COMPort>,
    pipe: NamedPipeClient,
}

impl Serial {
    /// Opens a COM port at the specified path
    pub fn from_path<T: AsRef<Path>>(
        path: T,
        settings: &SerialPortSettings,
    ) -> crate::Result<Self> {
        let mut name = Vec::<u16>::new();

        // See https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew#communications-resources
        // for more details about the path the `CreateFileW` call.
        name.extend(OsStr::new("\\\\.\\").encode_wide());
        name.extend(path.as_ref().as_os_str().encode_wide());
        name.push(0);

        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
                0 as HANDLE,
            )
        };

        if handle != INVALID_HANDLE_VALUE {
            // The COMPort is used to interact with the serial port by
            // leveraging the `serialport` crate.
            let mut serial = unsafe { COMPort::from_raw_handle(handle) };
            serial.set_all(settings)?;
            override_comm_timeouts(handle)?;

            // We abuse tokio's named pipe implementation to get the async
            // part working. The `AsyncFd` of the unix implementation is much
            // cleaner, but we don't have that on Windows so...
            let pipe = unsafe { NamedPipeClient::from_raw_handle(handle)? };

            // We use `ManuallyDrop` because both `COMPort` and `NamedPipeClient` take
            // ownership of the raw handle. To avoid closing the handle twice when
            // the `Serial` is dropped, we explicitly do **not** drop one of the
            // owned instances. Yes, it is a hack.

            Ok(Serial {
                inner: ManuallyDrop::new(serial),
                pipe,
            })
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

impl SerialPort for Serial {
    fn name(&self) -> Option<String> {
        self.inner.name()
    }

    fn settings(&self) -> SerialPortSettings {
        self.inner.settings()
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        self.inner.baud_rate()
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        self.inner.data_bits()
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        self.inner.flow_control()
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        self.inner.parity()
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        self.inner.stop_bits()
    }

    fn timeout(&self) -> std::time::Duration {
        self.inner.timeout()
    }

    fn set_all(&mut self, settings: &SerialPortSettings) -> serialport::Result<()> {
        self.inner.set_all(settings)
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
        self.inner.set_baud_rate(baud_rate)
    }

    fn set_data_bits(&mut self, data_bits: serialport::DataBits) -> serialport::Result<()> {
        self.inner.set_data_bits(data_bits)
    }

    fn set_flow_control(
        &mut self,
        flow_control: serialport::FlowControl,
    ) -> serialport::Result<()> {
        self.inner.set_flow_control(flow_control)
    }

    fn set_parity(&mut self, parity: serialport::Parity) -> serialport::Result<()> {
        self.inner.set_parity(parity)
    }

    fn set_stop_bits(&mut self, stop_bits: serialport::StopBits) -> serialport::Result<()> {
        self.inner.set_stop_bits(stop_bits)
    }

    fn set_timeout(&mut self, timeout: std::time::Duration) -> serialport::Result<()> {
        self.inner.set_timeout(timeout)
    }

    fn write_request_to_send(&mut self, level: bool) -> serialport::Result<()> {
        self.inner.write_request_to_send(level)
    }

    fn write_data_terminal_ready(&mut self, level: bool) -> serialport::Result<()> {
        self.inner.write_data_terminal_ready(level)
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        self.inner.read_clear_to_send()
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        self.inner.read_data_set_ready()
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        self.inner.read_ring_indicator()
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        self.inner.read_carrier_detect()
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        self.inner.bytes_to_read()
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        self.inner.bytes_to_write()
    }

    fn clear(&self, buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
        self.inner.clear(buffer_to_clear)
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        self.inner.try_clone()
    }
}

impl std::io::Read for Serial {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // TODO: check if this actually works
        self.inner.read(buf)
    }
}

impl std::io::Write for Serial {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // TODO: check if this actually works
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // TODO: check if this actually works
        self.inner.flush()
    }
}

impl AsyncRead for Serial {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().pipe).poll_read(cx, buf)
    }
}

impl AsyncWrite for Serial {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.get_mut().pipe).poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().pipe).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().pipe).poll_shutdown(cx)
    }
}

/// Overrides timeout value set by serialport-rs so that the read end will
/// never wake up with 0-byte payload.
fn override_comm_timeouts(handle: RawHandle) -> crate::Result<()> {
    let mut timeouts = COMMTIMEOUTS {
        // wait at most 1ms between two bytes (0 means no timeout)
        ReadIntervalTimeout: 1,
        // disable "total" timeout to wait at least 1 byte forever
        ReadTotalTimeoutMultiplier: 0,
        ReadTotalTimeoutConstant: 0,
        // write timeouts are just copied from serialport-rs
        WriteTotalTimeoutMultiplier: 0,
        WriteTotalTimeoutConstant: 0,
    };

    let r = unsafe { SetCommTimeouts(handle, &mut timeouts) };
    if r == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}
