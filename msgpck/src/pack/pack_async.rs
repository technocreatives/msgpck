use crate::PackError;

/// Trait for asynchronously serializing a type using [msgpack][https://msgpack.org/].
///
/// # Usage
///
/// The recommended usage is to use the derive macro `#[derive(AsyncMsgPck)]` on your
/// type which will generate an implementation for you.
#[cfg(feature = "async")]
pub trait AsyncMsgPck {
    /// Pack yourself into a writer, asynchronously.
    async fn pack_async(&self, writer: impl embedded_io_async::Write) -> Result<(), PackError>;
    // {
    //     use embedded_io_async::Error;

    //     writer
    //         .write_all(&crate::pack_vec(self)?)
    //         .await
    //         .map_err(|e| PackError::AsyncWriteError {
    //             kind: e.kind(),
    //             #[cfg(feature = "std")]
    //             error: format!("{e:?}"),
    //         })?;
    //     Ok(())
    // }
}

#[cfg(feature = "async")]
#[cfg(test)]
#[smol_potat::test]
async fn test_async() {
    let data = vec![String::from("hello")];
    let mut writer = Vec::new();
    data.pack_async(&mut writer).await.unwrap();
}
