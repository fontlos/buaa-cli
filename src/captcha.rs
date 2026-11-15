use buaa_api::api::sso::captcha::{CaptchaAction, CaptchaSolver};

use std::io::Write;

use crate::utils;

pub struct StdinSolver;

impl CaptchaSolver for StdinSolver {
    fn solve(&self, captcha: &[u8]) -> Result<CaptchaAction, String> {
        let path = utils::get_path("captcha.png").unwrap();
        std::fs::write(&path, captcha).map_err(|_| "Failed to write captcha image".to_string())?;
        println!("[Info] Captcha saved to {}", path.display());
        print!("[Info] Please input captcha (or type 'refresh'): ");
        std::io::stdout().flush().map_err(|_| "Failed to flush stdout".to_string())?;
        let mut s = String::new();
        std::io::stdin()
            .read_line(&mut s)
            .map_err(|_| "Failed to read captcha input".to_string())?;
        if s.trim() == "refresh" {
            return Ok(CaptchaAction::Refresh);
        }
        Ok(CaptchaAction::Submit(s.trim().to_string()))
    }
}
