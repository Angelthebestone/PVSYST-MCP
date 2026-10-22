//! Shared helpers for the stdio smoke tests: a minimal MCP client driving the
//! compiled server exactly like a real client (newline-delimited JSON-RPC).
//! Each test crate includes it with `mod common;` and uses a subset.
// Test code: failing loudly on unexpected state is the point.
#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    unreachable_pub
)]

use pvsyst_mcp::{Paths, Server};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub const DEMO_WS: &str = r"C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data";

/// In-process server with the same surface a client gets by default.
pub fn protocol_server() -> Server {
    let mut paths = Paths::resolve();
    paths.aproximaciones = false;
    Server::new(paths)
}

/// Sends one line to a fresh in-process server and parses the response.
pub fn protocol_call(line: &str) -> Value {
    let out = protocol_server().handle_line(line).expect("response");
    serde_json::from_str(&out).expect("valid json")
}

pub struct Client {
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    child: Child,
}

impl Client {
    pub fn start(workspace: Option<&str>) -> Client {
        Client::start_env(workspace, &[])
    }

    /// Start the server with extra environment variables (e.g. `PVSYST_CLI`,
    /// `PVSYST_MCP_APROXIMACIONES`, `FAKE_PVSYST_FIXTURES`).
    pub fn start_env(workspace: Option<&str>, env: &[(&str, &str)]) -> Client {
        let exe = env!("CARGO_BIN_EXE_pvsyst-mcp");
        let mut cmd = Command::new(exe);
        // Pin the default surface: a developer shell that exports this must
        // not change what the "default" tests see.
        cmd.env_remove("PVSYST_MCP_APROXIMACIONES");
        if let Some(w) = workspace {
            cmd.env("PVSYST_WORKSPACE", w);
        }
        for (k, v) in env {
            cmd.env(k, v);
        }
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn pvsyst-mcp");
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        Client {
            stdin,
            reader: BufReader::new(stdout),
            child,
        }
    }

    pub fn send(&mut self, v: &Value) {
        writeln!(self.stdin, "{v}").unwrap();
        self.stdin.flush().unwrap();
    }

    pub fn recv(&mut self) -> Value {
        let mut line = String::new();
        self.reader.read_line(&mut line).expect("read line");
        assert!(!line.is_empty(), "server closed stdout unexpectedly");
        serde_json::from_str(&line).unwrap_or_else(|e| panic!("bad json: {e} in {line}"))
    }

    /// Send an arbitrary JSON-RPC request and return the response.
    pub fn request(&mut self, id: u64, method: &str, params: Value) -> Value {
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        self.recv()
    }

    pub fn initialize(&mut self) -> Value {
        let init = self.request(
            1,
            "initialize",
            json!({ "protocolVersion": "2024-11-05", "capabilities": {},
                    "clientInfo": { "name": "t", "version": "0" } }),
        );
        self.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        init
    }

    pub fn call(&mut self, id: u64, name: &str, args: Value) -> Value {
        self.request(id, "tools/call", json!({ "name": name, "arguments": args }))
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // Closing stdin makes the server exit its read loop.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The JSON object a tool returned in its first text content item.
pub fn tool_text(resp: &Value) -> Value {
    let text = resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("no text content in {resp}"));
    serde_json::from_str(text).unwrap_or_else(|e| panic!("tool text not json: {e} in {text}"))
}

/// Path of a file under the repository's `tests/fixtures` folder.
pub fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(rel)
}

/// A private, self-cleaning copy of the synthetic workspace in `tests/fixtures/workspace`.
pub fn synthetic_workspace() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("temp dir");
    copy_dir(&fixture("workspace"), tmp.path());
    tmp
}

/// Copy one PVsyst demo project (`.PRJ` plus `variante`) into `destino/Projects`.
pub fn copiar_proyecto_demo(destino: &Path, nombre: &str, variante: &str) {
    let projects = destino.join("Projects");
    std::fs::create_dir_all(&projects).unwrap();
    for archivo in [format!("{nombre}.PRJ"), format!("{nombre}.{variante}")] {
        std::fs::copy(
            Path::new(DEMO_WS).join("Projects").join(&archivo),
            projects.join(&archivo),
        )
        .unwrap();
    }
}

/// Recursively copy a directory (workspaces for isolated tests).
pub fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &dest);
        } else {
            std::fs::copy(entry.path(), dest).unwrap();
        }
    }
}
