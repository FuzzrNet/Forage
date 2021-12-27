#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    Hash,
    PartialEq,
    rkyv :: Archive,
    rkyv :: Deserialize,
    rkyv :: Serialize,
)]
#[archive_attr(derive(Debug, Eq, Hash, PartialEq), repr(C))]
pub struct File {
    pub bao_hash: String,
    pub blake3_hash: String,
    #[with(cambria::Number)]
    pub bytes_read: i64,
    #[with(cambria::Number)]
    pub bytes_written: i64,
    #[with(cambria::Number)]
    pub date_accessed: i64,
    #[with(cambria::Number)]
    pub date_created: i64,
    #[with(cambria::Number)]
    pub date_modified: i64,
    #[with(cambria::Bool)]
    pub dropped: bool,
    #[with(cambria::Number)]
    pub max_slice: i64,
    pub mime_type: String,
    #[with(cambria::Number)]
    pub min_slice: i64,
    pub parent_rev: String,
    pub path: String,
    #[with(cambria::Bool)]
    pub removed: bool,
}
impl cambria::FromValue for File {
    fn from_value(value: &cambria::Value) -> cambria::anyhow::Result<Self> {
        if let cambria::Value::Object(obj) = value {
            Ok(Self {
                bao_hash: {
                    let value = obj
                        .get("bao_hash")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key bao_hash"))?;
                    cambria::FromValue::from_value(value)?
                },
                blake3_hash: {
                    let value = obj
                        .get("blake3_hash")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key blake3_hash"))?;
                    cambria::FromValue::from_value(value)?
                },
                bytes_read: {
                    let value = obj
                        .get("bytes_read")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key bytes_read"))?;
                    cambria::FromValue::from_value(value)?
                },
                bytes_written: {
                    let value = obj
                        .get("bytes_written")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key bytes_written"))?;
                    cambria::FromValue::from_value(value)?
                },
                date_accessed: {
                    let value = obj
                        .get("date_accessed")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key date_accessed"))?;
                    cambria::FromValue::from_value(value)?
                },
                date_created: {
                    let value = obj
                        .get("date_created")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key date_created"))?;
                    cambria::FromValue::from_value(value)?
                },
                date_modified: {
                    let value = obj
                        .get("date_modified")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key date_modified"))?;
                    cambria::FromValue::from_value(value)?
                },
                dropped: {
                    let value = obj
                        .get("dropped")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key dropped"))?;
                    cambria::FromValue::from_value(value)?
                },
                max_slice: {
                    let value = obj
                        .get("max_slice")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key max_slice"))?;
                    cambria::FromValue::from_value(value)?
                },
                mime_type: {
                    let value = obj
                        .get("mime_type")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key mime_type"))?;
                    cambria::FromValue::from_value(value)?
                },
                min_slice: {
                    let value = obj
                        .get("min_slice")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key min_slice"))?;
                    cambria::FromValue::from_value(value)?
                },
                parent_rev: {
                    let value = obj
                        .get("parent_rev")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key parent_rev"))?;
                    cambria::FromValue::from_value(value)?
                },
                path: {
                    let value = obj
                        .get("path")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key path"))?;
                    cambria::FromValue::from_value(value)?
                },
                removed: {
                    let value = obj
                        .get("removed")
                        .ok_or_else(|| cambria::anyhow::anyhow!("expected key removed"))?;
                    cambria::FromValue::from_value(value)?
                },
            })
        } else {
            Err(cambria::anyhow::anyhow!("expected object"))
        }
    }
}
impl cambria::ArchivedCambria for ArchivedFile {
    fn lenses() -> &'static [u8] {
        use cambria::aligned::{Aligned, A8};
        static LENSES: Aligned<A8, [u8; 1116usize]> = Aligned([
            98u8, 108u8, 97u8, 107u8, 101u8, 51u8, 95u8, 104u8, 97u8, 115u8, 104u8, 98u8, 108u8,
            97u8, 107u8, 101u8, 51u8, 95u8, 104u8, 97u8, 115u8, 104u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 1u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            98u8, 97u8, 111u8, 95u8, 104u8, 97u8, 115u8, 104u8, 98u8, 97u8, 111u8, 95u8, 104u8,
            97u8, 115u8, 104u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 98u8, 121u8, 116u8, 101u8, 115u8, 95u8, 114u8,
            101u8, 97u8, 100u8, 98u8, 121u8, 116u8, 101u8, 115u8, 95u8, 114u8, 101u8, 97u8, 100u8,
            0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 98u8, 121u8, 116u8, 101u8, 115u8, 95u8, 119u8, 114u8, 105u8, 116u8,
            116u8, 101u8, 110u8, 98u8, 121u8, 116u8, 101u8, 115u8, 95u8, 119u8, 114u8, 105u8,
            116u8, 116u8, 101u8, 110u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 109u8, 105u8, 110u8, 95u8, 115u8,
            108u8, 105u8, 99u8, 101u8, 109u8, 105u8, 110u8, 95u8, 115u8, 108u8, 105u8, 99u8, 101u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 109u8, 97u8, 120u8, 95u8, 115u8, 108u8, 105u8, 99u8, 101u8,
            109u8, 97u8, 120u8, 95u8, 115u8, 108u8, 105u8, 99u8, 101u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 112u8, 97u8, 114u8, 101u8, 110u8, 116u8, 95u8, 114u8, 101u8, 118u8,
            112u8, 97u8, 114u8, 101u8, 110u8, 116u8, 95u8, 114u8, 101u8, 118u8, 0u8, 0u8, 0u8, 0u8,
            1u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 109u8,
            105u8, 109u8, 101u8, 95u8, 116u8, 121u8, 112u8, 101u8, 109u8, 105u8, 109u8, 101u8,
            95u8, 116u8, 121u8, 112u8, 101u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8,
            2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 100u8, 97u8, 116u8, 101u8,
            95u8, 99u8, 114u8, 101u8, 97u8, 116u8, 101u8, 100u8, 100u8, 97u8, 116u8, 101u8, 95u8,
            99u8, 114u8, 101u8, 97u8, 116u8, 101u8, 100u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8,
            1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 100u8, 97u8, 116u8, 101u8,
            95u8, 109u8, 111u8, 100u8, 105u8, 102u8, 105u8, 101u8, 100u8, 100u8, 97u8, 116u8,
            101u8, 95u8, 109u8, 111u8, 100u8, 105u8, 102u8, 105u8, 101u8, 100u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 100u8, 97u8, 116u8, 101u8, 95u8, 97u8, 99u8, 99u8, 101u8, 115u8, 115u8,
            101u8, 100u8, 100u8, 97u8, 116u8, 101u8, 95u8, 97u8, 99u8, 99u8, 101u8, 115u8, 115u8,
            101u8, 100u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 3u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            2u8, 0u8, 0u8, 0u8, 11u8, 0u8, 0u8, 0u8, 216u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 11u8, 0u8, 0u8, 0u8, 207u8, 253u8, 255u8,
            255u8, 212u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 8u8, 0u8,
            0u8, 0u8, 220u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8,
            0u8, 0u8, 8u8, 0u8, 0u8, 0u8, 208u8, 253u8, 255u8, 255u8, 208u8, 253u8, 255u8, 255u8,
            0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 10u8, 0u8, 0u8, 0u8, 216u8, 253u8, 255u8,
            255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 10u8, 0u8, 0u8, 0u8,
            206u8, 253u8, 255u8, 255u8, 208u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8,
            0u8, 0u8, 13u8, 0u8, 0u8, 0u8, 216u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 13u8, 0u8, 0u8, 0u8, 209u8, 253u8, 255u8, 255u8,
            216u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8,
            224u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8,
            9u8, 0u8, 0u8, 0u8, 213u8, 253u8, 255u8, 255u8, 216u8, 253u8, 255u8, 255u8, 0u8, 0u8,
            0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 224u8, 253u8, 255u8, 255u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 213u8, 253u8,
            255u8, 255u8, 216u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8,
            112u8, 97u8, 116u8, 104u8, 0u8, 0u8, 0u8, 4u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            9u8, 0u8, 0u8, 0u8, 112u8, 97u8, 116u8, 104u8, 0u8, 0u8, 0u8, 4u8, 196u8, 253u8, 255u8,
            255u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 10u8, 0u8, 0u8, 0u8, 204u8, 253u8,
            255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 10u8, 0u8,
            0u8, 0u8, 194u8, 253u8, 255u8, 255u8, 196u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8,
            2u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 204u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 193u8, 253u8, 255u8, 255u8,
            196u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 12u8, 0u8, 0u8,
            0u8, 204u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8,
            0u8, 12u8, 0u8, 0u8, 0u8, 196u8, 253u8, 255u8, 255u8, 200u8, 253u8, 255u8, 255u8, 0u8,
            0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 13u8, 0u8, 0u8, 0u8, 208u8, 253u8, 255u8, 255u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 13u8, 0u8, 0u8, 0u8, 201u8,
            253u8, 255u8, 255u8, 208u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8,
            0u8, 13u8, 0u8, 0u8, 0u8, 216u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 13u8, 0u8, 0u8, 0u8, 209u8, 253u8, 255u8, 255u8, 216u8,
            253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 100u8, 114u8, 111u8,
            112u8, 112u8, 101u8, 100u8, 7u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8,
            0u8, 100u8, 114u8, 111u8, 112u8, 112u8, 101u8, 100u8, 7u8, 196u8, 253u8, 255u8, 255u8,
            0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 114u8, 101u8, 109u8, 111u8, 118u8, 101u8,
            100u8, 7u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 114u8, 101u8,
            109u8, 111u8, 118u8, 101u8, 100u8, 7u8, 176u8, 253u8, 255u8, 255u8, 0u8, 0u8, 0u8, 0u8,
            188u8, 253u8, 255u8, 255u8, 29u8, 0u8, 0u8, 0u8,
        ]);
        &LENSES[..]
    }
    fn schema() -> &'static cambria::ArchivedSchema {
        use cambria::aligned::{Aligned, A8};
        static SCHEMA: Aligned<A8, [u8; 424usize]> = Aligned([
            112u8, 97u8, 114u8, 101u8, 110u8, 116u8, 95u8, 114u8, 101u8, 118u8, 109u8, 105u8,
            110u8, 95u8, 115u8, 108u8, 105u8, 99u8, 101u8, 109u8, 105u8, 109u8, 101u8, 95u8, 116u8,
            121u8, 112u8, 101u8, 109u8, 97u8, 120u8, 95u8, 115u8, 108u8, 105u8, 99u8, 101u8, 100u8,
            97u8, 116u8, 101u8, 95u8, 109u8, 111u8, 100u8, 105u8, 102u8, 105u8, 101u8, 100u8,
            100u8, 97u8, 116u8, 101u8, 95u8, 99u8, 114u8, 101u8, 97u8, 116u8, 101u8, 100u8, 100u8,
            97u8, 116u8, 101u8, 95u8, 97u8, 99u8, 99u8, 101u8, 115u8, 115u8, 101u8, 100u8, 98u8,
            121u8, 116u8, 101u8, 115u8, 95u8, 119u8, 114u8, 105u8, 116u8, 116u8, 101u8, 110u8,
            98u8, 121u8, 116u8, 101u8, 115u8, 95u8, 114u8, 101u8, 97u8, 100u8, 98u8, 108u8, 97u8,
            107u8, 101u8, 51u8, 95u8, 104u8, 97u8, 115u8, 104u8, 98u8, 97u8, 111u8, 95u8, 104u8,
            97u8, 115u8, 104u8, 0u8, 0u8, 0u8, 120u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 14u8, 0u8,
            0u8, 0u8, 8u8, 0u8, 0u8, 0u8, 233u8, 255u8, 255u8, 255u8, 3u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 11u8, 0u8, 0u8, 0u8, 202u8, 255u8, 255u8, 255u8, 3u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 10u8, 0u8, 0u8, 0u8, 172u8,
            255u8, 255u8, 255u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 13u8,
            0u8, 0u8, 0u8, 139u8, 255u8, 255u8, 255u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 13u8, 0u8, 0u8, 0u8, 106u8, 255u8, 255u8, 255u8, 2u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 12u8, 0u8, 0u8, 0u8, 74u8, 255u8, 255u8, 255u8,
            2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 13u8, 0u8, 0u8, 0u8, 41u8,
            255u8, 255u8, 255u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 100u8,
            114u8, 111u8, 112u8, 112u8, 101u8, 100u8, 7u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 248u8, 254u8, 255u8, 255u8, 2u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 219u8, 254u8, 255u8, 255u8,
            3u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 9u8, 0u8, 0u8, 0u8, 190u8,
            254u8, 255u8, 255u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 10u8,
            0u8, 0u8, 0u8, 160u8, 254u8, 255u8, 255u8, 3u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 112u8, 97u8, 116u8, 104u8, 0u8, 0u8, 0u8, 4u8, 3u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 114u8, 101u8, 109u8, 111u8, 118u8, 101u8, 100u8,
            7u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 5u8, 0u8, 0u8, 0u8,
            14u8, 0u8, 0u8, 0u8, 212u8, 254u8, 255u8, 255u8,
        ]);
        unsafe { cambria::rkyv::archived_root::<cambria::Schema>(&SCHEMA[..]) }
    }
}
impl cambria::Cambria for File {
    fn lenses() -> &'static [u8] {
        use cambria::ArchivedCambria;
        ArchivedFile::lenses()
    }
    fn schema() -> &'static cambria::ArchivedSchema {
        use cambria::ArchivedCambria;
        ArchivedFile::schema()
    }
}
