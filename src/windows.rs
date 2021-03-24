use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use std::io::{self, Read, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

/// Serial port I/O struct.
pub struct TTYPort;

impl TTYPort {
    /// Open serial port from a provided path, using the default reactor.
    pub fn open(_builder: &crate::SerialPortBuilder) -> crate::Result<Self> {
       unimplemented!()
    }

    /// Sets the exclusivity of the port
    ///
    /// If a port is exclusive, then trying to open the same device path again
    /// will fail.
    ///
    /// See the man pages for the tiocexcl and tiocnxcl ioctl's for more details.
    ///
    /// ## Errors
    ///
    /// * `Io` for any error while setting exclusivity for the port.
    pub fn set_exclusive(&mut self, _exclusive: bool) -> crate::Result<()> {
        unimplemented!()
    }
}

impl crate::SerialPort for TTYPort {
    #[inline(always)]
    fn name(&self) -> Option<String> {
        unimplemented!()
    }

    #[inline(always)]
    fn baud_rate(&self) -> crate::Result<u32> {
        unimplemented!()
    }

    #[inline(always)]
    fn data_bits(&self) -> crate::Result<crate::DataBits> {
        unimplemented!()
    }

    #[inline(always)]
    fn flow_control(&self) -> crate::Result<crate::FlowControl> {
        unimplemented!()
    }

    #[inline(always)]
    fn parity(&self) -> crate::Result<crate::Parity> {
        unimplemented!()
    }

    #[inline(always)]
    fn stop_bits(&self) -> crate::Result<crate::StopBits> {
        unimplemented!()
    }

    #[inline(always)]
    fn timeout(&self) -> Duration {
        Duration::from_secs(0)
    }

    #[inline(always)]
    fn set_baud_rate(&mut self, _baud_rate: u32) -> crate::Result<()> {
        unimplemented!()
    }

    #[inline(always)]
    fn set_data_bits(&mut self, _data_bits: crate::DataBits) -> crate::Result<()> {
        unimplemented!()
    }

    #[inline(always)]
    fn set_flow_control(&mut self, _flow_control: crate::FlowControl) -> crate::Result<()> {
        unimplemented!()
    }

    #[inline(always)]
    fn set_parity(&mut self, _parity: crate::Parity) -> crate::Result<()> {
        unimplemented!()
    }

    #[inline(always)]
    fn set_stop_bits(&mut self, _stop_bits: crate::StopBits) -> crate::Result<()> {
        unimplemented!()
    }

    #[inline(always)]
    fn set_timeout(&mut self, _: Duration) -> crate::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn write_request_to_send(&mut self, _level: bool) -> crate::Result<()> {
        unimplemented!()
    }

    #[inline(always)]
    fn write_data_terminal_ready(&mut self, _level: bool) -> crate::Result<()> {
        unimplemented!()
    }

    #[inline(always)]
    fn read_clear_to_send(&mut self) -> crate::Result<bool> {
        unimplemented!()
    }

    #[inline(always)]
    fn read_data_set_ready(&mut self) -> crate::Result<bool> {
        unimplemented!()
    }

    #[inline(always)]
    fn read_ring_indicator(&mut self) -> crate::Result<bool> {
        unimplemented!()
    }

    #[inline(always)]
    fn read_carrier_detect(&mut self) -> crate::Result<bool> {
        unimplemented!()
    }

    #[inline(always)]
    fn bytes_to_read(&self) -> crate::Result<u32> {
        unimplemented!()
    }

    #[inline(always)]
    fn bytes_to_write(&self) -> crate::Result<u32> {
        unimplemented!()
    }

    #[inline(always)]
    fn clear(&self, _buffer_to_clear: crate::ClearBuffer) -> crate::Result<()> {
        unimplemented!()
    }

    #[inline(always)]
    fn try_clone(&self) -> crate::Result<Box<dyn crate::SerialPort>> {
        Err(crate::Error::new(
            crate::ErrorKind::Io(std::io::ErrorKind::Other),
            "Cannot clone Tokio handles",
        ))
    }

    #[inline(always)]
    fn set_break(&self) -> crate::Result<()> {
        unimplemented!()
    }

    #[inline(always)]
    fn clear_break(&self) -> crate::Result<()> {
        unimplemented!()
    }
}

impl Read for TTYPort {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }
}

impl Write for TTYPort {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}

impl AsyncRead for TTYPort {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        unimplemented!()
    }
}

impl AsyncWrite for TTYPort {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        unimplemented!()
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        unimplemented!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        unimplemented!()
    }
}
