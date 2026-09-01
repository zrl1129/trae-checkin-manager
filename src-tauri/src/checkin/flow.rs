use std::time::Duration;

use anyhow::Result;
use serde_json::Value;
use tokio::time::sleep;

use crate::checkin::cdp::CdpClient;
use crate::checkin::types::CheckinStatus;

pub struct CheckinResult {
    pub status: CheckinStatus,
    pub detail: String,
    pub points: Option<i64>,
}

pub async fn perform_checkin(cdp: &mut CdpClient) -> Result<CheckinResult> {
    let menu_open = cdp
        .evaluate(
            r#"(() => { return !!document.querySelector('[class*="accountPopover"]'); })()"#,
        )
        .await?;

    if !menu_open.as_bool().unwrap_or(false) {
        log::info!("点击左下角头像...");
        cdp.evaluate(
            r#"(() => {
                const el = document.querySelector('[class*="accountTrigger"]');
                if (el) { el.click(); return true; }
                return false;
            })()"#,
        )
        .await?;

        for _ in 0..10 {
            sleep(Duration::from_millis(500)).await;
            let open = cdp
                .evaluate(
                    r#"(() => { return !!document.querySelector('[class*="accountPopover"]'); })()"#,
                )
                .await?;
            if open.as_bool().unwrap_or(false) {
                break;
            }
        }
    }

    let state: Value = cdp
        .evaluate(
            r#"(() => {
                const btn = document.querySelector('[class*="accountCheckinButton"]');
                const label = document.querySelector('[class*="accountCheckinButtonLabel"]');
                if (!btn) return { error: 'checkin_button_not_found' };
                return {
                    buttonText: label ? (label.textContent || '').trim() : (btn.textContent || '').trim(),
                    title: (document.querySelector('[class*="accountCheckinTitle"]')?.textContent || '').trim()
                };
            })()"#,
        )
        .await?;

    if state.get("error").and_then(|v| v.as_str()) == Some("checkin_button_not_found") {
        let hint = cdp
            .evaluate(
                r#"(() => {
                    const menu = document.querySelector('[class*="accountPopover"]');
                    if (!menu) return { menuOpen: false };
                    const t = (menu.textContent || '');
                    return {
                        menuOpen: true,
                        hasLogin: /登录|扫码|立即登录|手机号/.test(t),
                        sample: t.replace(/\s+/g, ' ').slice(0, 80)
                    };
                })()"#,
            )
            .await?;

        if hint.get("hasLogin").and_then(|v| v.as_bool()).unwrap_or(false) {
            let sample = hint
                .get("sample")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            return Ok(CheckinResult {
                status: CheckinStatus::NotLoggedIn,
                detail: format!("未登录: {}", sample),
                points: None,
            });
        }

        return Ok(CheckinResult {
            status: CheckinStatus::Failed,
            detail: "签到按钮未找到".to_string(),
            points: None,
        });
    }

    let button_text = state
        .get("buttonText")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if button_text.contains("已签") {
        return Ok(CheckinResult {
            status: CheckinStatus::AlreadySigned,
            detail: button_text.to_string(),
            points: None,
        });
    }

    log::info!("点击签到按钮...");
    cdp.evaluate(
        r#"(() => {
            const btn = document.querySelector('[class*="accountCheckinButton"]');
            if (!btn) return false;
            btn.click();
            return true;
        })()"#,
    )
    .await?;

    sleep(Duration::from_secs(2)).await;

    let verify = cdp
        .evaluate(
            r#"(() => {
                const label = document.querySelector('[class*="accountCheckinButtonLabel"]');
                const btn = document.querySelector('[class*="accountCheckinButton"]');
                return label ? (label.textContent || '').trim() : (btn ? (btn.textContent || '').trim() : 'button_gone');
            })()"#,
        )
        .await?;

    let after_text = verify.as_str().unwrap_or("");

    if after_text.contains("已签") {
        let title = cdp
            .evaluate(
                r#"(() => {
                    const el = document.querySelector('[class*="accountCheckinTitle"]');
                    return el ? (el.textContent || '').trim() : '';
                })()"#,
            )
            .await?;

        let title_text = title.as_str().unwrap_or("");
        let points = extract_points(title_text);

        return Ok(CheckinResult {
            status: CheckinStatus::Success,
            detail: after_text.to_string(),
            points,
        });
    }

    Ok(CheckinResult {
        status: CheckinStatus::Failed,
        detail: format!("签到后验证失败: {}", after_text),
        points: None,
    })
}

fn extract_points(text: &str) -> Option<i64> {
    let mut num = String::new();
    let mut found = false;

    for c in text.chars() {
        if c.is_ascii_digit() {
            num.push(c);
            found = true;
        } else if found {
            if !num.is_empty() {
                if let Ok(n) = num.parse::<i64>() {
                    if n >= 50 {
                        return Some(n);
                    }
                }
            }
            num.clear();
            found = false;
        }
    }

    if !num.is_empty() {
        if let Ok(n) = num.parse::<i64>() {
            if n >= 50 {
                return Some(n);
            }
        }
    }

    None
}
