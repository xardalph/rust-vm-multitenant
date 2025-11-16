#![allow(dead_code)]

use docker_api::{Docker, Result, conn::TtyChunk};
use std::str;

#[cfg(unix)]
pub fn new_docker() -> Result<Docker> {
    Ok(Docker::unix("/var/run/docker.sock"))
}

#[cfg(not(unix))]
pub fn new_docker() -> Result<Docker> {
    Docker::new("tcp://127.0.0.1:8080")
}

pub fn print_chunk(chunk: TtyChunk) {
    match chunk {
        TtyChunk::StdOut(bytes) => {
            println!("Stdout: {}", str::from_utf8(&bytes).unwrap_or_default())
        }
        TtyChunk::StdErr(bytes) => {
            eprintln!("Stdout: {}", str::from_utf8(&bytes).unwrap_or_default())
        }
        TtyChunk::StdIn(_) => unreachable!(),
    }
}

fn main() {}
