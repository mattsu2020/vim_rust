use wayland_client::{Connection, ConnectError};

/// Connect to the Wayland compositor using `wayland-client`.
pub fn connect() -> Result<Connection, ConnectError> {
    Connection::connect_to_env()
}
