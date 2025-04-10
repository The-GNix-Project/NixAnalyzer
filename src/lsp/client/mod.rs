// SPDX-License-Identifier: GPL-3.0-or-later
//
// This file is part of GNix.
// GNix - The Graphical Nix Project
// -----------------------------------------------------------------------------------------|
// GNix is free software: you can redistribute it and/or modify                             |
// it under the terms of the GNU General Public License as published by                     |
// the Free Software Foundation, either version 3 of the License, or any later version.     |
//                                                                                          |
// GNix is distributed in the hope that it will be useful,                                  |
// but WITHOUT ANY WARRANTY; without even the implied warranty of                           |
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the                            |
// GNU General Public License for more details.                                             |
//                                                                                          |
// You should have received a copy of the GNU General Public License                        |
// along with GNix.  If not, see <https://www.gnu.org/licenses/>.                           |
// -----------------------------------------------------------------------------------------|
pub mod builder;

use tokio::process::{ChildStdin, ChildStdout};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use super::error::LspError;

pub struct LspTransport {
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
}

impl LspTransport {
    pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
        Self {
            stdin,
            reader: BufReader::new(stdout),
        }
    }

    pub async fn send_raw(&mut self, message: String) -> Result<(), LspError> {
        self.stdin.write_all(message.as_bytes()).await?;
        Ok(())
    }

    pub async fn receive_raw(&mut self) -> Result<serde_json::Value, LspError> {
        let mut content_length = 0;
        let mut line = String::new();

        // Read headers
        loop {
            line.clear();
            let bytes_read = self.reader.read_line(&mut line).await?;
            
            // End of headers
            if bytes_read == 2 && line == "\r\n" {
                break;
            }
            
            // Parse Content-Length
            if line.to_ascii_lowercase().starts_with("content-length:") {
                let parts: Vec<&str> = line.split(':').collect();
                content_length = parts[1]
                    .trim()
                    .parse()
                    .map_err(|e| LspError::Protocol(format!("Invalid Content-Length: {}", e)))?;
            }
        }

        // Read message body
        let mut body = vec![0u8; content_length];
        self.reader.read_exact(&mut body).await?;
        
        // Parse JSON response
        Ok(serde_json::from_slice(&body)?)
    }
}