use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use async_trait::async_trait;
use rs9p::fcall::*;
use rs9p::srv::{FId, Filesystem}; // Removed 'Srv', fixed 'Fid' -> 'FId'
use rs9p::Result; // Use the crate directly

// 1. The "Kernel" State
#[derive(Clone)]
struct WillowFS {
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl WillowFS {
    fn new() -> Self {
        WillowFS {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

// 2. The 9P Implementation
#[async_trait]
impl Filesystem for WillowFS {
    type FId = String; // Fixed 'Fid' -> 'FId'

    // ATTACH: The client connects
    async fn rattach(
        &self,
        fid: &FId<Self::FId>,
        _afid: Option<&FId<Self::FId>>,
        _uname: &str,
        _aname: &str,
        _n_uname: u32,
    ) -> Result<FCall> {
        // Fixed Fcall -> FCall
        let qid = QId {
            // Fixed Qid -> QId
            typ: QIdType::DIR, // Fixed QidType -> QIdType
            version: 0,
            path: 0,
        };
        *fid.aux.lock().await = Some(String::from("/"));
        Ok(FCall::Rattach { qid })
    }

    // WALK: Navigate directories
    async fn rwalk(
        &self,
        fid: &FId<Self::FId>,
        newfid: &FId<Self::FId>,
        names: &[String],
    ) -> Result<FCall> {
        let mut wqids = Vec::new();
        for name in names {
            let qid = QId {
                typ: QIdType::FILE,
                version: 0,
                path: 12345,
            };
            wqids.push(qid);
            *newfid.aux.lock().await = Some(name.clone());
        }
        Ok(FCall::Rwalk { wqids })
    }

    // OPEN (Linux specific: rlopen)
    async fn rlopen(&self, _fid: &FId<Self::FId>, _flags: u32) -> Result<FCall> {
        let qid = QId {
            typ: QIdType::FILE,
            version: 0,
            path: 12345,
        };
        Ok(FCall::Rlopen { qid, iounit: 8192 })
    }

    // CREATE (Linux specific: rlcreate)
    async fn rlcreate(
        &self,
        fid: &FId<Self::FId>,
        name: &str,
        _flags: u32,
        _mode: u32,
        _gid: u32,
    ) -> Result<FCall> {
        let mut store = self.files.write().unwrap();
        store.insert(name.to_string(), Vec::new());

        *fid.aux.lock().await = Some(name.to_string());

        let qid = QId {
            typ: QIdType::FILE,
            version: 0,
            path: 12345,
        };
        Ok(FCall::Rlcreate { qid, iounit: 8192 })
    }

    // READ
    async fn rread(&self, fid: &FId<Self::FId>, offset: u64, count: u32) -> Result<FCall> {
        let name_guard = fid.aux.lock().await;
        let name = name_guard.as_ref().unwrap();

        // Handle Directory Listing (ls)
        if name == "/" {
            let store = self.files.read().unwrap();
            // DirEntryCursor was moved/renamed, so we build the buffer manually for now
            // or use a simpler iterator approach.
            // For a "Hello World", we will manually serialize a single entry if needed,
            // but let's try the raw byte approach for simplicity in 9P2000.L

            // NOTE: 9P2000.L readdir is complex.
            // For this specific errors fix, I will return an empty read for "/"
            // just to get it compiling. Implementing full `readdir` requires `rs9p::utils`.
            return Ok(FCall::Rread { data: Vec::new() });
        }

        // Handle File Reading (cat)
        let store = self.files.read().unwrap();
        if let Some(content) = store.get(name) {
            let start = offset as usize;
            let end = std::cmp::min(start + count as usize, content.len());
            if start > content.len() {
                Ok(FCall::Rread { data: Vec::new() })
            } else {
                Ok(FCall::Rread {
                    data: content[start..end].to_vec(),
                })
            }
        } else {
            Ok(FCall::Rread { data: Vec::new() })
        }
    }

    // WRITE
    async fn rwrite(&self, fid: &FId<Self::FId>, _offset: u64, data: &[u8]) -> Result<FCall> {
        let name_guard = fid.aux.lock().await;
        let name = name_guard.as_ref().unwrap();

        let mut store = self.files.write().unwrap();
        if let Some(file_content) = store.get_mut(name) {
            file_content.extend_from_slice(data);
            Ok(FCall::Rwrite {
                count: data.len() as u32,
            })
        } else {
            Ok(FCall::Rwrite { count: 0 })
        }
    }

    // STAT (getattr)
    async fn rgetattr(&self, _fid: &FId<Self::FId>, req_mask: u64) -> Result<FCall> {
        Ok(FCall::Rgetattr {
            valid: req_mask,
            qid: QId {
                typ: QIdType::FILE,
                version: 0,
                path: 12345,
            },
            mode: 0o644,
            uid: 0,
            gid: 0,
            nlink: 1,
            rdev: 0,
            size: 0,
            atime: SystemTime::now().into(),
            mtime: SystemTime::now().into(),
            ctime: SystemTime::now().into(),
            btime: SystemTime::now().into(),
            gen: 0,
            data_version: 0,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    println!("[*] Starting Willow Daemon...");

    let fs = WillowFS::new();
    let addr = "0.0.0.0:5640";
    println!("[*] Listening on TCP {}", addr);

    // srv_tcp is usually in the root or srv module, check specific version
    // For 0.13, it is often:
    rs9p::srv::srv_tcp(fs, addr).await
}
