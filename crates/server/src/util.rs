use std::sync::Arc;

use anyhttp::{HttpError, HttpExecutor, RequestBody};

#[derive(Clone)]
pub struct WasixHttpExecutor {
    client: wasix_http_client::HttpClient,
}

pub struct Body(wasix_http_client::Body);

struct DynWrapper<E>(E);

type DynChunks = Box<dyn Iterator<Item = Result<Vec<u8>, HttpError>>>;
type DynReader = Box<dyn std::io::Read>;
pub type DynResponseBody = Box<
    dyn anyhttp::Respond<
        Chunks = DynChunks,
        BytesOutput = Result<Vec<u8>, HttpError>,
        Reader = DynReader,
    >,
>;

impl<E> HttpExecutor for DynWrapper<E>
where
    E: HttpExecutor,
    E::Output: Into<Result<anyhttp::Response<E::ResponseBody>, HttpError>>,
    E::ResponseBody: anyhttp::Respond<BytesOutput = Result<Vec<u8>, HttpError>>,
    <E::ResponseBody as anyhttp::Respond>::Chunks:
        Iterator<Item = Result<Vec<u8>, HttpError>> + 'static,
    <E::ResponseBody as anyhttp::Respond>::Reader: std::io::Read + 'static,
{
    type RequestBody = RequestBody;
    type ResponseBody = DynResponseBody;
    type Output = Result<anyhttp::Response<Self::ResponseBody>, HttpError>;

    fn request_body_from_generic(&self, body: RequestBody) -> Self::RequestBody {
        body
    }

    fn new_output_error(&self, error: HttpError) -> Self::Output {
        Err(error)
    }

    fn execute(&self, request: anyhttp::RequestPre<Self::RequestBody>) -> Self::Output {
        let res = self.0.execute_generic(request).into()?;
        let res = res.map_body(move |b| -> DynResponseBody { Box::new(DynRespondWrapper(b)) });
        Ok(res)
    }

    fn execute_generic(&self, pre: anyhttp::RequestPre<RequestBody>) -> Self::Output {
        self.execute(pre)
    }
}

struct DynRespondWrapper<R>(R);

impl<R> anyhttp::Respond for DynRespondWrapper<R>
where
    R: anyhttp::Respond<BytesOutput = Result<Vec<u8>, HttpError>>,
    R::Chunks: Iterator<Item = Result<Vec<u8>, HttpError>> + 'static,
    R::Reader: std::io::Read + 'static,
{
    type Chunks = DynChunks;
    type BytesOutput = Result<Vec<u8>, HttpError>;
    type Reader = DynReader;

    fn into_chunks(self) -> Self::Chunks {
        Box::new(self.0.into_chunks())
    }

    fn into_chunks_boxed(self: Box<Self>) -> Self::Chunks {
        (*self).into_chunks()
    }

    fn bytes(self) -> Self::BytesOutput {
        self.0.bytes()
    }

    fn bytes_boxed(self: Box<Self>) -> Self::BytesOutput {
        (*self).0.bytes()
    }

    fn reader(self) -> Self::Reader {
        Box::new(self.0.reader())
    }

    fn reader_boxed(self: Box<Self>) -> Self::Reader {
        (*self).reader()
    }
}

impl WasixHttpExecutor {
    pub fn new() -> Result<Self, anyhow::Error> {
        let client = wasix_http_client::HttpClient::new()?;
        Ok(Self { client })
    }

    pub fn new_dyn_client() -> Result<anyhttp::sync::DynClient, anyhow::Error> {
        let exec = Self::new()?;
        let w = DynWrapper(exec);
        let e2: anyhttp::sync::DynExecutor = Arc::new(w);
        let c = anyhttp::sync::DynClient::new(e2);
        Ok(c)
    }
}

impl anyhttp::HttpExecutor for WasixHttpExecutor {
    type RequestBody = Body;
    type ResponseBody = Body;
    type Output = Result<anyhttp::Response<Self::ResponseBody>, HttpError>;

    fn request_body_from_generic(&self, body: anyhttp::RequestBody) -> Self::RequestBody {
        match body {
            RequestBody::Empty => Body(wasix_http_client::Body::empty()),
            RequestBody::Bytes(bytes) => Body(wasix_http_client::Body::new_data(bytes)),
            RequestBody::Read(_) => todo!("std::io::Read body not supported yet"),
        }
    }

    fn new_output_error(&self, error: anyhttp::HttpError) -> Self::Output {
        Err(error)
    }

    fn execute(&self, pre: anyhttp::RequestPre<Self::RequestBody>) -> Self::Output {
        let mut req = http::Request::builder()
            .method(pre.request.method)
            .uri(pre.request.uri)
            .body(pre.request.body.0)
            .unwrap();
        *req.headers_mut() = pre.request.headers;
        let res = self
            .client
            .send(req)
            .map_err(|err| HttpError::new_custom(err.to_string()))?;
        let (parts, body) = res.into_parts();

        Ok(anyhttp::Response {
            uri: None,
            status: parts.status,
            version: http::Version::HTTP_11,
            headers: parts.headers,
            extensions: Default::default(),
            body: Body(body),
        })
    }
}

impl anyhttp::Respond for Body {
    type Chunks = BodyIter;
    type BytesOutput = Result<Vec<u8>, HttpError>;
    type Reader = std::io::Cursor<Vec<u8>>;

    fn into_chunks(self) -> Self::Chunks {
        BodyIter { body: Some(self.0) }
    }

    fn into_chunks_boxed(self: Box<Self>) -> Self::Chunks {
        (*self).into_chunks()
    }

    fn bytes(self) -> Self::BytesOutput {
        self.0
            .read_all()
            .map_err(|err| HttpError::new_custom(err.to_string()))
    }

    fn bytes_boxed(self: Box<Self>) -> Self::BytesOutput {
        (*self).bytes()
    }

    fn reader(self) -> Self::Reader {
        unimplemented!()
    }

    fn reader_boxed(self: Box<Self>) -> Self::Reader {
        unimplemented!()
    }
}

pub struct BodyIter {
    body: Option<wasix_http_client::Body>,
}

impl Iterator for BodyIter {
    type Item = Result<Vec<u8>, HttpError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(b) = self.body.take() {
            let res = b
                .read_all()
                .map_err(|err| anyhttp::HttpError::new_custom(err.to_string()));
            Some(res)
        } else {
            None
        }
    }
}
