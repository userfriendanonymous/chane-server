use super::auth::MeResponse;
use ts_rs::TS;

#[derive(TS)] #[ts(export)]
struct AuthMeResponse(MeResponse);
