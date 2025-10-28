//! HTTP Server Listener Built-in Node
//!
//! Provides a TCP-based HTTP server listener that accepts incoming connections
//! and outputs raw HTTP requests for processing by HTTP component pipeline.
//!
//! This is a temporary solution until WASI HTTP incoming-handler becomes available.

use crate::graph::node::{ComponentSpec, DataType, NodeValue};
use crate::runtime::engine::NodeExecutor;
use crate::ComponentError;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// HTTP Server Listener Executor
///
/// This node listens for incoming TCP connections and reads HTTP requests.
/// It's designed to run as a continuous node, accepting connections in a loop.
///
/// **Note**: This is a synchronous blocking implementation. Each connection is
/// handled sequentially. For production use, consider implementing async with
/// connection pooling.
pub struct HttpServerListenerExecutor {
    /// Shared state for the listener (allows stopping and connection tracking)
    state: Arc<Mutex<ServerState>>,
}

/// Internal state for the HTTP server listener
struct ServerState {
    /// TCP listener (created on first execution)
    listener: Option<TcpListener>,
    /// Total connections accepted
    connection_count: u32,
    /// Whether the server is running
    is_running: bool,
    /// Configured host
    host: String,
    /// Configured port
    port: u16,
    /// Current active connection waiting for response (only one at a time)
    active_connection: Option<(u32, TcpStream)>,
}

impl HttpServerListenerExecutor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ServerState {
                listener: None,
                connection_count: 0,
                is_running: false,
                host: "127.0.0.1".to_string(),
                port: 8080,
                active_connection: None,
            })),
        }
    }

    /// Explicitly shutdown the HTTP server listener and release the port
    pub fn shutdown(&self) {
        if let Ok(mut state) = self.state.lock() {
            // Close any active connection
            if let Some((conn_id, _stream)) = state.active_connection.take() {
                log::debug!("Closing active connection {} during shutdown", conn_id);
            }

            // Drop the TcpListener to release the port
            if let Some(listener) = state.listener.take() {
                log::info!(
                    "Closing TcpListener on {}:{}",
                    state.host,
                    state.port
                );
                drop(listener); // Explicitly drop to close socket
                log::info!(
                    "TcpListener dropped, port {}:{} should now be released",
                    state.host,
                    state.port
                );
            } else {
                log::debug!("No listener to close (was already None)");
            }

            state.is_running = false;
        } else {
            log::error!("Failed to lock state during shutdown");
        }
    }
}

impl NodeExecutor for HttpServerListenerExecutor {
    fn execute(
        &self,
        inputs: &HashMap<String, NodeValue>,
    ) -> Result<HashMap<String, NodeValue>, ComponentError> {
        let mut outputs = HashMap::new();

        // Get configuration from inputs
        let host = if let Some(NodeValue::String(h)) = inputs.get("host") {
            h.clone()
        } else {
            "127.0.0.1".to_string()
        };

        let port = if let Some(NodeValue::U32(p)) = inputs.get("port") {
            *p as u16
        } else {
            8080
        };

        let max_request_size = if let Some(NodeValue::U32(size)) = inputs.get("max_request_size") {
            *size as usize
        } else {
            1024 * 1024 // 1MB default
        };

        let timeout_ms = if let Some(NodeValue::U32(timeout)) = inputs.get("connection_timeout_ms")
        {
            *timeout as u64
        } else {
            5000 // 5 seconds default
        };

        // Lock state
        let mut state = self.state.lock().map_err(|e| {
            ComponentError::ExecutionError(format!("Failed to lock server state: {}", e))
        })?;

        // Initialize listener if needed (or if config changed)
        if state.listener.is_none() {
            // First time initialization with SO_REUSEADDR
            let addr = format!("{}:{}", host, port);
            log::debug!("Attempting to bind TcpListener to {} with SO_REUSEADDR", addr);

            // Parse address
            let socket_addr: std::net::SocketAddr = addr.parse().map_err(|e| {
                ComponentError::ExecutionError(format!("Invalid address {}: {}", addr, e))
            })?;

            // Create socket with SO_REUSEADDR to allow immediate port reuse
            let socket = socket2::Socket::new(
                if socket_addr.is_ipv4() {
                    socket2::Domain::IPV4
                } else {
                    socket2::Domain::IPV6
                },
                socket2::Type::STREAM,
                None,
            )
            .map_err(|e| {
                ComponentError::ExecutionError(format!("Failed to create socket: {}", e))
            })?;

            // Set SO_REUSEADDR to allow binding even if port is in TIME_WAIT state
            socket.set_reuse_address(true).map_err(|e| {
                ComponentError::ExecutionError(format!("Failed to set SO_REUSEADDR: {}", e))
            })?;

            // On macOS, try to set SO_REUSEPORT for better port reuse behavior
            // This is a best-effort attempt - if it fails, we continue anyway
            #[cfg(target_os = "macos")]
            {
                use std::os::fd::AsRawFd;
                let fd = socket.as_raw_fd();
                let optval: libc::c_int = 1;
                let ret = unsafe {
                    libc::setsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        libc::SO_REUSEPORT,
                        &optval as *const _ as *const libc::c_void,
                        std::mem::size_of_val(&optval) as libc::socklen_t,
                    )
                };
                if ret != 0 {
                    log::warn!("Failed to set SO_REUSEPORT: {}", std::io::Error::last_os_error());
                } else {
                    log::debug!("Successfully set SO_REUSEPORT on socket");
                }
            }

            // Set non-blocking mode so we can check for stop signals
            socket.set_nonblocking(true).map_err(|e| {
                ComponentError::ExecutionError(format!("Failed to set non-blocking mode: {}", e))
            })?;

            // Bind the socket
            socket.bind(&socket_addr.into()).map_err(|e| {
                log::error!("Failed to bind to {}: {}", addr, e);
                ComponentError::ExecutionError(format!(
                    "Failed to bind to {}: {}. Check if port {} is available and not in use",
                    addr, e, port
                ))
            })?;

            // Start listening
            socket.listen(128).map_err(|e| {
                ComponentError::ExecutionError(format!("Failed to listen: {}", e))
            })?;

            // Convert to std::net::TcpListener
            let listener: TcpListener = socket.into();

            state.listener = Some(listener);
            state.host = host.clone();
            state.port = port;
            state.is_running = true;
            state.connection_count = 0;

            log::info!("HTTP server listening on {}:{}", host, port);
        } else if state.host != host || state.port != port {
            // Config changed - warn but don't try to rebind (would fail with "address in use")
            log::warn!(
                "HTTP server config changed ({}:{} -> {}:{}), but cannot rebind while running. \
                 Stop the node and restart to apply new settings.",
                state.host, state.port, host, port
            );
            // Continue with existing listener
        }

        // Check if there's a response to send back
        if let Some(NodeValue::String(response)) = inputs.get("response") {
            // Send response to the active connection (if any)
            if let Some((conn_id, mut stream)) = state.active_connection.take() {
                // Write the response to the TcpStream
                match stream.write_all(response.as_bytes()) {
                    Ok(_) => {
                        match stream.flush() {
                            Ok(_) => {
                                log::info!("Sent HTTP response to connection {}", conn_id);
                                outputs.insert(
                                    "response_status".to_string(),
                                    NodeValue::String(format!("sent to connection {}", conn_id)),
                                );
                                // Clear raw_request so we don't keep propagating old requests
                                // This signals that the request has been fully processed
                                outputs.insert(
                                    "raw_request".to_string(),
                                    NodeValue::String(String::new()),
                                );
                            }
                            Err(e) => {
                                log::warn!("Failed to flush response to connection {}: {}", conn_id, e);
                                outputs.insert(
                                    "response_status".to_string(),
                                    NodeValue::String(format!("flush error: {}", e)),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to write response to connection {}: {}", conn_id, e);
                        outputs.insert(
                            "response_status".to_string(),
                            NodeValue::String(format!("write error: {}", e)),
                        );
                    }
                }
                // TcpStream is automatically closed when dropped
                // Connection is cleared, ready for next one
            } else {
                log::debug!("Response received but no active connection to send it to");
                // This is normal when there's no request yet
            }
        }

        // Try to accept a connection (non-blocking) - only if no active connection
        if let Some(ref listener) = state.listener {
            // Only accept new connection if we don't have an active one waiting for response
            if state.active_connection.is_none() {
                match listener.accept() {
                    Ok((mut stream, addr)) => {
                        state.connection_count += 1;
                        let connection_id = state.connection_count;

                        // Drop the state lock before reading from stream
                        drop(state);

                        log::info!("Accepted connection {} from {}", connection_id, addr);

                        // Set read timeout
                        stream
                            .set_read_timeout(Some(Duration::from_millis(timeout_ms)))
                            .ok();

                        // Read HTTP request
                        match read_http_request(&mut stream, max_request_size) {
                            Ok(request) => {
                                outputs.insert("raw_request".to_string(), NodeValue::String(request));
                                outputs.insert(
                                    "client_addr".to_string(),
                                    NodeValue::String(addr.to_string()),
                                );
                                outputs
                                    .insert("connection_id".to_string(), NodeValue::U32(connection_id));
                                outputs.insert(
                                    "status".to_string(),
                                    NodeValue::String("ready".to_string()),
                                );

                                log::debug!(
                                    "Read HTTP request from connection {} ({})",
                                    connection_id,
                                    addr
                                );

                                // Store the TcpStream as the active connection for response writing
                                if let Ok(mut state) = self.state.lock() {
                                    state.active_connection = Some((connection_id, stream));
                                    log::debug!("Stored connection {} as active connection", connection_id);
                                } else {
                                    log::warn!("Failed to lock state to store connection {}", connection_id);
                                    // Stream will be dropped (connection closed)
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to read request from {}: {}", addr, e);
                                outputs.insert(
                                    "status".to_string(),
                                    NodeValue::String(format!("error: {}", e)),
                                );
                                // Stream will be dropped (connection closed)
                            }
                        }
                    }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection available, return waiting status
                    outputs.insert(
                        "status".to_string(),
                        NodeValue::String("waiting".to_string()),
                    );
                    // Don't output raw_request - keep the previous value on the port
                    // This prevents overwriting the actual request with empty data
                }
                Err(e) => {
                    log::error!("Failed to accept connection: {}", e);
                    outputs.insert(
                        "status".to_string(),
                        NodeValue::String(format!("error: {}", e)),
                    );
                    // Don't output raw_request - keep the previous value on the port
                }
                }
            } else {
                // Already have an active connection, waiting for response
                outputs.insert(
                    "status".to_string(),
                    NodeValue::String("connection_active".to_string()),
                );
                // Don't output raw_request - keep the previous value on the port
                // This keeps the request data available for the parser
            }
        } else {
            outputs.insert(
                "status".to_string(),
                NodeValue::String("not_initialized".to_string()),
            );
            // Don't output raw_request - port will have no value initially
            // First request will populate it, and it will persist
        }

        Ok(outputs)
    }
}

/// Read HTTP request from TCP stream
///
/// Reads until double CRLF (end of headers), then reads body if Content-Length is present.
fn read_http_request(
    stream: &mut std::net::TcpStream,
    max_size: usize,
) -> Result<String, std::io::Error> {
    let mut reader = BufReader::new(stream);
    let mut request = String::new();
    let mut headers_complete = false;
    let mut content_length: Option<usize> = None;

    // Read headers line by line
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            // Connection closed
            break;
        }

        request.push_str(&line);

        if request.len() > max_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Request exceeds maximum size of {} bytes", max_size),
            ));
        }

        // Check for end of headers (empty line: \r\n)
        if line == "\r\n" {
            headers_complete = true;

            // Check if there's a body (Content-Length header)
            content_length = extract_content_length(&request);

            if content_length.is_none() || content_length == Some(0) {
                // No body or empty body, we're done
                break;
            } else {
                // Need to read body
                break;
            }
        }
    }

    // Read body if Content-Length is present
    if headers_complete {
        if let Some(length) = content_length {
            if length > 0 {
                if request.len() + length > max_size {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Request with body exceeds maximum size of {} bytes",
                            max_size
                        ),
                    ));
                }

                let mut body = vec![0u8; length];
                reader.read_exact(&mut body)?;
                request.push_str(&String::from_utf8_lossy(&body));
            }
        }
    }

    Ok(request)
}

/// Extract Content-Length from HTTP request headers
fn extract_content_length(request: &str) -> Option<usize> {
    for line in request.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.starts_with("content-length:") {
            let value = line_lower.strip_prefix("content-length:")?.trim();
            return value.parse().ok();
        }
    }
    None
}

/// Register the HTTP server listener node in the component registry
pub fn register_http_server_listener(registry: &mut crate::graph::node::ComponentRegistry) {
    let spec = ComponentSpec::new_builtin(
        "builtin:continuous:http-server-listener".to_string(),
        "HTTP Server Listener".to_string(),
        "Listens for incoming HTTP connections, outputs raw HTTP requests, and sends responses back. \
         Runs continuously, accepting connections and handling responses. \
         Stores connections until a response is sent via the response input ports."
            .to_string(),
        Some("HTTP".to_string()),
    )
    .with_input(
        "host".to_string(),
        DataType::String,
        "Host to bind to (default: 127.0.0.1)".to_string(),
    )
    .with_input(
        "port".to_string(),
        DataType::U32,
        "Port to listen on (default: 8080)".to_string(),
    )
    .with_input(
        "max_request_size".to_string(),
        DataType::U32,
        "Maximum request size in bytes (default: 1048576 = 1MB)".to_string(),
    )
    .with_input(
        "connection_timeout_ms".to_string(),
        DataType::U32,
        "Connection read timeout in milliseconds (default: 5000 = 5s)".to_string(),
    )
    .with_input(
        "response".to_string(),
        DataType::String,
        "HTTP response to send back (complete response including headers and body). \
         Automatically sent to the current active connection.".to_string(),
    )
    .with_output(
        "raw_request".to_string(),
        DataType::String,
        "Complete raw HTTP request (headers + body)".to_string(),
    )
    .with_output(
        "client_addr".to_string(),
        DataType::String,
        "Client IP address and port (e.g., '192.168.1.1:54321')".to_string(),
    )
    .with_output(
        "connection_id".to_string(),
        DataType::U32,
        "Sequential connection ID (increments with each connection)".to_string(),
    )
    .with_output(
        "status".to_string(),
        DataType::String,
        "Server status: 'waiting', 'ready', 'error: ...', or 'not_initialized'".to_string(),
    )
    .with_output(
        "response_status".to_string(),
        DataType::String,
        "Response sending status: 'sent to connection X', 'write error: ...', or 'connection X not found'".to_string(),
    );

    registry.register_builtin(spec);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_content_length_present() {
        let request = "POST / HTTP/1.1\r\nHost: example.com\r\nContent-Length: 42\r\n\r\n";
        assert_eq!(extract_content_length(request), Some(42));
    }

    #[test]
    fn test_extract_content_length_missing() {
        let request = "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        assert_eq!(extract_content_length(request), None);
    }

    #[test]
    fn test_extract_content_length_zero() {
        let request = "POST / HTTP/1.1\r\nHost: example.com\r\nContent-Length: 0\r\n\r\n";
        assert_eq!(extract_content_length(request), Some(0));
    }

    #[test]
    fn test_extract_content_length_case_insensitive() {
        let request = "POST / HTTP/1.1\r\nHost: example.com\r\ncontent-length: 100\r\n\r\n";
        assert_eq!(extract_content_length(request), Some(100));
    }

    #[test]
    fn test_extract_content_length_with_spaces() {
        let request = "POST / HTTP/1.1\r\nHost: example.com\r\nContent-Length:   123   \r\n\r\n";
        assert_eq!(extract_content_length(request), Some(123));
    }
}
