use std::io::{Read, Write};
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::ready;
use nix::libc;
use nix::sys::termios;
use serialport::posix::TTYPort;
use serialport::{SerialPort, SerialPortSettings};
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct Serial {
    io: AsyncFd<TTYPort>,
}

impl Serial {
    /// Open a nonblocking serial port from the provided path
    pub fn from_path<T: AsRef<Path>>(
        path: T,
        settings: &SerialPortSettings,
    ) -> crate::Result<Self> {
        let port = TTYPort::open(path.as_ref(), settings)?;
        Self::from_serial(port)
    }

    pub fn from_serial(port: TTYPort) -> crate::Result<Self> {
        // Get the termios structure
        let mut t = termios::tcgetattr(port.as_raw_fd()).map_err(map_nix_error)?;

        // Set VMIN = 1 to block until at least one character is received.
        t.control_chars[termios::SpecialCharacterIndices::VMIN as usize] = 1;
        termios::tcsetattr(port.as_raw_fd(), termios::SetArg::TCSANOW, &t)
            .map_err(map_nix_error)?;

        // Set the O_NONBLOCK flag.
        let flags = unsafe { libc::fcntl(port.as_raw_fd(), libc::F_GETFL) };
        if flags < 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        match unsafe { libc::fcntl(port.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } {
            0 => Ok(Self {
                io: AsyncFd::new(port)?,
            }),
            _ => Err(std::io::Error::last_os_error().into()),
        }
    }
}

impl SerialPort for Serial {
    fn name(&self) -> Option<String> {
        self.io.get_ref().name()
    }

    fn settings(&self) -> SerialPortSettings {
        self.io.get_ref().settings()
    }

    fn baud_rate(&self) -> serialport::Result<u32> {
        self.io.get_ref().baud_rate()
    }

    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        self.io.get_ref().data_bits()
    }

    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        self.io.get_ref().flow_control()
    }

    fn parity(&self) -> serialport::Result<serialport::Parity> {
        self.io.get_ref().parity()
    }

    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        self.io.get_ref().stop_bits()
    }

    fn timeout(&self) -> std::time::Duration {
        self.io.get_ref().timeout()
    }

    fn set_all(&mut self, settings: &SerialPortSettings) -> serialport::Result<()> {
        self.io.get_mut().set_all(settings)
    }

    fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
        self.io.get_mut().set_baud_rate(baud_rate)
    }

    fn set_data_bits(&mut self, data_bits: serialport::DataBits) -> serialport::Result<()> {
        self.io.get_mut().set_data_bits(data_bits)
    }

    fn set_flow_control(
        &mut self,
        flow_control: serialport::FlowControl,
    ) -> serialport::Result<()> {
        self.io.get_mut().set_flow_control(flow_control)
    }

    fn set_parity(&mut self, parity: serialport::Parity) -> serialport::Result<()> {
        self.io.get_mut().set_parity(parity)
    }

    fn set_stop_bits(&mut self, stop_bits: serialport::StopBits) -> serialport::Result<()> {
        self.io.get_mut().set_stop_bits(stop_bits)
    }

    fn set_timeout(&mut self, timeout: std::time::Duration) -> serialport::Result<()> {
        self.io.get_mut().set_timeout(timeout)
    }

    fn write_request_to_send(&mut self, level: bool) -> serialport::Result<()> {
        self.io.get_mut().write_request_to_send(level)
    }

    fn write_data_terminal_ready(&mut self, level: bool) -> serialport::Result<()> {
        self.io.get_mut().write_data_terminal_ready(level)
    }

    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        self.io.get_mut().read_clear_to_send()
    }

    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        self.io.get_mut().read_data_set_ready()
    }

    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        self.io.get_mut().read_ring_indicator()
    }

    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        self.io.get_mut().read_carrier_detect()
    }

    fn bytes_to_read(&self) -> serialport::Result<u32> {
        self.io.get_ref().bytes_to_read()
    }

    fn bytes_to_write(&self) -> serialport::Result<u32> {
        self.io.get_ref().bytes_to_write()
    }

    fn clear(&self, buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
        self.io.get_ref().clear(buffer_to_clear)
    }

    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        self.io.get_ref().try_clone()
    }
}

macro_rules! uninterruptibly {
    ($e:expr) => {{
        loop {
            match $e {
                Err(ref error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                res => break res,
            }
        }
    }};
}

impl Read for Serial {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        uninterruptibly!(match unsafe {
            libc::read(
                self.as_raw_fd(),
                bytes.as_ptr() as *mut libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(std::io::Error::last_os_error()),
        })
    }
}

impl Write for Serial {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        uninterruptibly!(match unsafe {
            libc::write(
                self.as_raw_fd(),
                bytes.as_ptr() as *const libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(std::io::Error::last_os_error()),
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        uninterruptibly!(
            termios::tcdrain(self.io.as_raw_fd()).map_err(|error| match error {
                nix::Error::Sys(errno) => std::io::Error::from(errno),
                error => std::io::Error::new(std::io::ErrorKind::Other, error.to_string()),
            })
        )
    }
}

impl<'a> Read for &'a Serial {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        uninterruptibly!(match unsafe {
            libc::read(
                self.as_raw_fd(),
                bytes.as_ptr() as *mut libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(std::io::Error::last_os_error()),
        })
    }
}

impl<'a> Write for &'a Serial {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        uninterruptibly!(match unsafe {
            libc::write(
                self.as_raw_fd(),
                bytes.as_ptr() as *const libc::c_void,
                bytes.len() as libc::size_t,
            )
        } {
            x if x >= 0 => Ok(x as usize),
            _ => Err(std::io::Error::last_os_error()),
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        uninterruptibly!(
            termios::tcdrain(self.io.as_raw_fd()).map_err(|error| match error {
                nix::Error::Sys(errno) => std::io::Error::from(errno),
                error => std::io::Error::new(std::io::ErrorKind::Other, error.to_string()),
            })
        )
    }
}

impl AsRawFd for Serial {
    fn as_raw_fd(&self) -> RawFd {
        self.io.as_raw_fd()
    }
}

impl IntoRawFd for Serial {
    fn into_raw_fd(self) -> RawFd {
        self.io.as_raw_fd()
    }
}

impl AsyncRead for Serial {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        loop {
            let mut guard = ready!(self.io.poll_read_ready(cx))?;

            match guard.try_io(|inner| {
                let read = inner.get_ref().read(buf.initialize_unfilled())?;
                buf.advance(read);
                Ok(())
            }) {
                Ok(result) => return Poll::Ready(result),
                Err(_would_block) => continue,
            }
        }
    }
}

impl AsyncWrite for Serial {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        loop {
            let mut guard = ready!(self.io.poll_write_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().write(buf)) {
                Ok(x) => return Poll::Ready(x),
                Err(_) => continue,
            };
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        loop {
            let mut guard = ready!(self.io.poll_write_ready(cx))?;

            match guard.try_io(|inner| inner.get_ref().flush()) {
                Ok(x) => return Poll::Ready(x),
                Err(_) => continue,
            };
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

fn map_nix_error(e: nix::Error) -> serialport::Error {
    serialport::Error {
        kind: serialport::ErrorKind::Io(std::io::ErrorKind::Other),
        description: e.to_string(),
    }
}
