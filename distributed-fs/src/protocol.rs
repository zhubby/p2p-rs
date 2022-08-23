use async_trait::async_trait;
use futures::{io, AsyncRead, AsyncWrite, AsyncWriteExt};
use libp2p::{
    core::{
        upgrade::{read_length_prefixed, write_length_prefixed},
        ProtocolName,
    },
    request_response::RequestResponseCodec,
};

#[derive(Debug, Clone)]
pub struct FileSwapProtocol();

#[derive(Clone)]
pub struct FileSwapCodec();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRequest(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResponse(pub String);

impl ProtocolName for FileSwapProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/dfs/1".as_bytes()
    }
}

#[async_trait]
impl RequestResponseCodec for FileSwapCodec {
    type Protocol = FileSwapProtocol;
    type Request = FileRequest;
    type Response = FileResponse;

    // 读请求
    async fn read_request<T>(
        &mut self,
        _: &FileSwapProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        // 读取固定长度的字节
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(FileRequest(String::from_utf8(vec).unwrap()))
    }

    // 读取响应
    async fn read_response<T>(
        &mut self,
        _: &FileSwapProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        // 读取固定长度的字节
        let vec = read_length_prefixed(io, 1_000_000).await?;

        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        Ok(FileResponse(String::from_utf8(vec).unwrap()))
    }

    // 写请求
    async fn write_request<T>(
        &mut self,
        _: &FileSwapProtocol,
        io: &mut T,
        FileRequest(data): FileRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &FileSwapProtocol,
        io: &mut T,
        FileResponse(data): FileResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;

        Ok(())
    }
}
