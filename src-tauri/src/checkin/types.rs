use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CheckinStatus {
    Pending,
    InProgress,
    Success,
    AlreadySigned,
    NotLoggedIn,
    Failed,
}

impl std::fmt::Display for CheckinStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckinStatus::Pending => write!(f, "待签到"),
            CheckinStatus::InProgress => write!(f, "签到中"),
            CheckinStatus::Success => write!(f, "已签到"),
            CheckinStatus::AlreadySigned => write!(f, "今日已签"),
            CheckinStatus::NotLoggedIn => write!(f, "未登录"),
            CheckinStatus::Failed => write!(f, "失败"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinRecord {
    pub id: String,
    pub account_id: String,
    pub instance_id: String,
    pub status: CheckinStatus,
    pub detail: String,
    pub points: Option<i64>,
    pub checkin_time: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CheckinStore {
    pub records: Vec<CheckinRecord>,
}

impl CheckinStore {
    pub fn has_checked_today(&self, account_id: &str) -> bool {
        let today = chrono::Local::now().date_naive();
        self.records.iter().any(|r| {
            r.account_id == account_id
                && r.checkin_time.is_some()
                && {
                    let ts = r.checkin_time.unwrap();
                    let dt = chrono::DateTime::from_timestamp(ts, 0)
                        .unwrap_or_default()
                        .with_timezone(&chrono::Local);
                    dt.date_naive() == today
                }
                && (r.status == CheckinStatus::Success || r.status == CheckinStatus::AlreadySigned)
        })
    }

    pub fn today_records(&self) -> Vec<&CheckinRecord> {
        let today = chrono::Local::now().date_naive();
        self.records
            .iter()
            .filter(|r| {
                r.checkin_time.is_some()
                    && {
                        let ts = r.checkin_time.unwrap();
                        let dt = chrono::DateTime::from_timestamp(ts, 0)
                            .unwrap_or_default()
                            .with_timezone(&chrono::Local);
                        dt.date_naive() == today
                    }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckinEvent {
    pub account_id: String,
    pub account_name: String,
    pub status: CheckinStatus,
    pub detail: String,
    pub points: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchSummary {
    pub total: usize,
    pub success: usize,
    pub already_signed: usize,
    pub failed: usize,
    pub skipped: usize,
}
