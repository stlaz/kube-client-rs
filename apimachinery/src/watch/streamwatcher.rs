use std::io::BufReader;
use std::{io::BufRead, result::Result};

use reqwest;
use reqwest::blocking::Response;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::WatchEvent;
use k8s_openapi::{serde, serde_json};

use crate::watch::ResourceWatcher;

pub struct StreamWatcher<T>
where
    T: serde::de::DeserializeOwned,
{
    _type: std::marker::PhantomData<T>,

    reader: std::io::BufReader<reqwest::blocking::Response>,
    stop: bool,
}

impl<T> StreamWatcher<T>
where
    T: serde::de::DeserializeOwned,
{
    pub fn new(reader: BufReader<Response>) -> Self {
        StreamWatcher {
            _type: std::marker::PhantomData,
            reader,
            stop: false,
        }
    }
}

impl<T> ResourceWatcher<T> for StreamWatcher<T>
where
    T: serde::de::DeserializeOwned,
{
    fn stop(&self) {}
    // TODO: instead of String, add a retryable/non-retryable error enum
    fn next(&mut self) -> Result<WatchEvent<T>, String> {
        if self.stop {
            return Err(format!("Watcher has been stopped"));
        }

        let mut buffer = String::new();
        let read_bytes = match self.reader.read_line(&mut buffer) {
            Ok(read_bytes) => read_bytes,
            Err(ref e) if e.kind() == std::io::ErrorKind::Other => {
                let Some(wrapped) = e.get_ref() else {
                    return Err(format!("Failed to read line: {e}"));
                };
                let Some(reqwest_err) = wrapped.downcast_ref::<reqwest::Error>() else {
                    return Err(format!("Failed to read line: {e}"));
                };
                if reqwest_err.is_timeout() {
                    return Err(format!("Request timed out")); // TODO: custom error here
                }
                return Err(format!("Failed to read line: {e}"));
            }
            Err(e) => return Err(format!("Failed to read line: {e}")),
        };

        if read_bytes == 0 {
            return Err(format!("Stream closed"));
        }

        let event: WatchEvent<T> = serde_json::from_str(&buffer)
            .map_err(|e| format!("Failed to parse event: {e}\n{buffer}"))?;
        Ok(event)
    }
}
