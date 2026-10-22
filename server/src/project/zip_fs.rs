use async_zip::{Compression, ZipEntryBuilder};
use futures::FutureExt;
use tokio::sync::Mutex;
use tracing::trace;

pub struct ZipFS {
    writer: Mutex<async_zip::base::write::ZipFileWriter<tokio_util::compat::Compat<tokio::io::DuplexStream>>>,
}

impl ZipFS {
    pub fn new(writer_stream: tokio::io::DuplexStream) -> Self {
        let writer = async_zip::base::write::ZipFileWriter::with_tokio(writer_stream);
        Self {
            writer: Mutex::new(writer),
        }
    }

    pub fn end_zip(self) -> impl Future<Output = ()> {
        self.writer.into_inner().close().map(|_| ())
    }
}

impl crate::project::filesystem::Filesystem for ZipFS {
    async fn create_file(&self, path: String, content: &Vec<u8>) -> Result<(), String> {
        let builder = ZipEntryBuilder::new(path.into(), Compression::Deflate);
        self.writer.lock().await.write_entry_whole(builder, content).await.map_err(|e| format!("{e:?}"))
    }

    async fn create_dir(&self, _path: String) -> Result<(), String> {
        trace!("Not creating directory in zip");
        Ok(())
    }
}
