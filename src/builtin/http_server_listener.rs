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
use std::io::{BufRead, BufReader, Read};
use std::net::TcpListener;
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
            })),
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

        let max_request_size = if let Some(NodeValue::U32(size)) = inputs.get("max_request_size")
        {
            *size as usize
        } else {
            1024 * 1024 // 1MB default
        };

        let timeout_ms =
            if let Some(NodeValue::U32(timeout)) = inputs.get("connection_timeout_ms") {
                *timeout as u64
            } else {
                5000 // 5 seconds default
            };

        // Lock state
        let mut state = self.state.lock().map_err(|e| ComponentError {
            message: format!("Failed to lock server state: {}", e),
            component_name: "http-server-listener".to_string(),
            details: None,
        })?;

        // Initialize listener if needed (or if config changed)
        if state.listener.is_none() || state.host != host || state.port != port {
            let addr = format!("{}:{}", host, port);
            let listener = TcpListener::bind(&addr).map_err(|e| ComponentError {
                message: format!("Failed to bind to {}: {}", addr, e),
                component_name: "http-server-listener".to_string(),
                details: Some(format!(
                    "Check if port {} is available and not in use",
                    port
                )),
            })?;

            // Set non-blocking mode so we can check for stop signals
            listener
                .set_nonblocking(true)
                .map_err(|e| ComponentError {
                    message: format!("Failed to set non-blocking mode: {}", e),
                    component_name: "http-server-listener".to_string(),
                    details: None,
                })?;

            state.listener = Some(listener);
            state.host = host.clone();
            state.port = port;
            state.is_running = true;
            state.connection_count = 0;

            log::info!("HTTP server listening on {}:{}", host, port);
        }

        // Try to accept a connection (non-blocking)
        if let Some(ref listener) = state.listener {
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
                            outputs.insert(
                                "connection_id".to_string(),
                                NodeValue::U32(connection_id),
                            );
                            outputs.insert("status".to_string(), NodeValue::String("ready".to_string()));

                            log::debug!(
                                "Read HTTP request from connection {} ({})",
                                connection_id,
                                addr
                            );
                        }
                        Err(e) => {
                            log::warn!("Failed to read request from {}: {}", addr, e);
                            outputs.insert(
                                "status".to_string(),
                                NodeValue::String(format!("error: {}", e)),
                            );
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection available, return waiting status
                    outputs.insert(
                        "status".to_string(),
                        NodeValue::String("waiting".to_string()),
                    );
                }
                Err(e) => {
                    log::error!("Failed to accept connection: {}", e);
                    outputs.insert(
                        "status".to_string(),
                        NodeValue::String(format!("error: {}", e)),
                    );
                }
            }
        } else {
            outputs.insert(
                "status".to_string(),
                NodeValue::String("not_initialized".to_string()),
            );
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
                        format!("Request with body exceeds maximum size of {} bytes", max_size),
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
            let value = line_lower
                .strip_prefix("content-length:")?
                .trim();
            return value.parse().ok();
        }
    }
    None
}

/// Register the HTTP server listener node in the component registry
pub fn register_http_server_listener(registry: &mut crate::graph::node::ComponentRegistry) {
    let spec = ComponentSpec::new_builtin(
        "builtin:http:server-listener".to_string(),
        "HTTP Server Listener".to_string(),
        "Listens for incoming HTTP connections and outputs raw HTTP requests. \
         Runs continuously, accepting one connection per execution cycle. \
         Use with http-request-parser to process requests."
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
