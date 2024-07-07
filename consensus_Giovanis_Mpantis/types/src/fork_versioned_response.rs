use crate::ForkName;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::value::Value;
use std::sync::Arc;

/// A trait for deserializing types based on a specific fork version.
pub trait ForkVersionDeserialize: Sized + DeserializeOwned {
    /// Deserialize the given `value` based on the specified `fork_name`.
    fn deserialize_by_fork<'de, D: Deserializer<'de>>(
        value: Value,
        fork_name: ForkName,
    ) -> Result<Self, D::Error>;
}

/// A versioned response structure that includes version information, metadata, and data.
///
/// The metadata type `M` should generally be set to `EmptyMetadata` if no additional fields
/// are required beyond version information. For custom metadata, any type implementing `Deserialize`
/// can be used.
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct ForkVersionedResponse<T, M = EmptyMetadata> {
    /// Optional version information indicating the fork name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<ForkName>,
    /// Additional metadata associated with the response.
    #[serde(flatten)]
    pub metadata: M,
    /// The main data payload of the response.
    pub data: T,
}

/// Metadata type that deserializes from a JSON map.
///
/// This is essentially a placeholder type that can be expanded for specific metadata needs.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct EmptyMetadata {}

/// Fork versioned response including information about execution optimism and finalization.
pub type ExecutionOptimisticFinalizedForkVersionedResponse<T> =
    ForkVersionedResponse<T, ExecutionOptimisticFinalizedMetadata>;

/// Metadata structure for execution optimism and finalization information.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ExecutionOptimisticFinalizedMetadata {
    /// Indicates if execution is optimistic.
    pub execution_optimistic: Option<bool>,
    /// Indicates if the response is finalized.
    pub finalized: Option<bool>,
}

impl<'de, F, M> serde::Deserialize<'de> for ForkVersionedResponse<F, M>
where
    F: ForkVersionDeserialize,
    M: DeserializeOwned,
{
    /// Deserialize implementation for `ForkVersionedResponse`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            version: Option<ForkName>,
            #[serde(flatten)]
            metadata: serde_json::Value,
            data: serde_json::Value,
        }

        let helper = Helper::deserialize(deserializer)?;
        let data = match helper.version {
            Some(fork_name) => F::deserialize_by_fork::<'de, D>(helper.data, fork_name)?,
            None => serde_json::from_value(helper.data).map_err(serde::de::Error::custom)?,
        };
        let metadata = serde_json::from_value(helper.metadata).map_err(serde::de::Error::custom)?;

        Ok(ForkVersionedResponse {
            version: helper.version,
            metadata,
            data,
        })
    }
}

impl<F: ForkVersionDeserialize> ForkVersionDeserialize for Arc<F> {
    /// Deserialize implementation for `Arc<F>` where `F` implements `ForkVersionDeserialize`.
    fn deserialize_by_fork<'de, D: Deserializer<'de>>(
        value: Value,
        fork_name: ForkName,
    ) -> Result<Self, D::Error> {
        Ok(Arc::new(F::deserialize_by_fork::<'de, D>(
            value, fork_name,
        )?))
    }
}

impl<T, M> ForkVersionedResponse<T, M> {
    /// Applies a function to the inner `data`, potentially transforming its type.
    pub fn map_data<U>(self, f: impl FnOnce(T) -> U) -> ForkVersionedResponse<U, M> {
        let ForkVersionedResponse {
            version,
            metadata,
            data,
        } = self;
        ForkVersionedResponse {
            version,
            metadata,
            data: f(data),
        }
    }
}

#[cfg(test)]
mod fork_version_response_tests {
    use crate::{
        ExecutionPayload, ExecutionPayloadBellatrix, ForkName, ForkVersionedResponse,
        MainnetEthSpec,
    };
    use serde_json::json;

    #[test]
    fn fork_versioned_response_deserialize_correct_fork() {
        type E = MainnetEthSpec;

        let response_json =
            serde_json::to_string(&json!(ForkVersionedResponse::<ExecutionPayload<E>> {
                version: Some(ForkName::Bellatrix),
                metadata: Default::default(),
                data: ExecutionPayload::Bellatrix(ExecutionPayloadBellatrix::default()),
            }))
            .unwrap();

        let result: Result<ForkVersionedResponse<ExecutionPayload<E>>, _> =
            serde_json::from_str(&response_json);

        assert!(result.is_ok());
    }

    #[test]
    fn fork_versioned_response_deserialize_incorrect_fork() {
        type E = MainnetEthSpec;

        let response_json =
            serde_json::to_string(&json!(ForkVersionedResponse::<ExecutionPayload<E>> {
                version: Some(ForkName::Capella),
                metadata: Default::default(),
                data: ExecutionPayload::Bellatrix(ExecutionPayloadBellatrix::default()),
            }))
            .unwrap();

        let result: Result<ForkVersionedResponse<ExecutionPayload<E>>, _> =
            serde_json::from_str(&response_json);

        assert!(result.is_err());
    }
}
