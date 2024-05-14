use chrono::Utc;
use ulid::Ulid;

struct Post {
    id: Ulid,
    created_at: Utc,
    in_reply_to_id: Option<Ulid>,
    in_reply_to_account_id: Option<String>,
    sensitive: bool,
    spoiler_text: String,
    //visibility: String,
}
