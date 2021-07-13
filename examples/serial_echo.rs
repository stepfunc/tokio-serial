use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/ttyUSB0";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

#[tokio::main]
async fn main() {
    // Parse TTY path from the command line
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());

    // Create the serial port
    let settings = tokio_serial::SerialPortSettings::default();
    let mut port = tokio_serial::Serial::from_path(tty_path, &settings).unwrap();

    // Setup stdout to print the bytes
    let mut stdout = tokio::io::stdout();

    let mut buf = vec![0; 1024];
    loop {
        // Listen for anything written
        match port.read(&mut buf).await {
            // Return value of `Ok(0)` signifies that the remote has closed
            Ok(0) => return,
            Ok(n) => {
                // Write the data to stdout
                stdout
                    .write_all(&buf[..n])
                    .await
                    .expect("unable to write to stdout");
                stdout.flush().await.expect("unable to flush stdout");

                // Copy the data back to socket
                port.write_all(&buf[..n])
                    .await
                    .expect("unable to write to the serial port");
            }
            Err(_) => return,
        }
    }
}
