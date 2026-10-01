use serde::{Deserialize, Serialize};
use sqlx::decode::Decode;
use sqlx::encode::Encode;
use sqlx::postgres::PgTypeInfo;
use sqlx::types::Type;
use std::fmt;
use uuid::Uuid;

/// Macro to generate a newtype ID wrapper around Uuid.
macro_rules! define_id {
    ($name:ident) => {
        /// Newtype wrapper around Uuid for type-safe IDs.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Generates a new random ID (UUID v7).
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            /// Creates an ID from an existing Uuid.
            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Returns the inner Uuid.
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }

        impl Type<sqlx::Postgres> for $name {
            fn type_info() -> PgTypeInfo {
                Uuid::type_info()
            }
        }

        impl<'q> Encode<'q, sqlx::Postgres> for $name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> sqlx::encode::IsNull {
                <Uuid as Encode<'q, sqlx::Postgres>>::encode_by_ref(&self.0, buf)
            }
        }

        impl<'r> Decode<'r, sqlx::Postgres> for $name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let uuid = <Uuid as Decode<'r, sqlx::Postgres>>::decode(value)?;
                Ok(Self(uuid))
            }
        }
    };
}

define_id!(UserId);
define_id!(OrgId);
define_id!(ChannelId);
define_id!(MessageId);
define_id!(BoardId);
define_id!(TaskId);
define_id!(WhiteboardId);
define_id!(NotificationId);
define_id!(InvitationId);
define_id!(SessionId);
define_id!(AttachmentId);
define_id!(ReactionId);
define_id!(PinId);
define_id!(TaskCommentId);
define_id!(LabelId);
define_id!(WebhookId);
define_id!(BotId);
define_id!(AuditLogId);
define_id!(DmConversationId);
define_id!(MessageHistoryId);
