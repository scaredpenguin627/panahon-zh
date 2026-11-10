//! 统计模板翻译对当前 PAGASA 预警的实际覆盖率。
//!
//! ```powershell
//! cargo run -p panahon-core --example advisory_coverage --features network
//! ```

fn main() {
    let url = format!("{}/alerts", panahon_core::API_ROOT);
    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(20))
        .set("User-Agent", "PanahonZh/0.3 coverage probe")
        .call();

    let value: serde_json::Value = match response {
        Ok(response) => match response.into_json() {
            Ok(value) => value,
            Err(error) => {
                println!("解析失败：{error}");
                return;
            }
        },
        Err(error) => {
            println!("请求失败：{error}");
            return;
        }
    };

    let alerts = value
        .pointer("/data")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    println!("预警条数：{}", alerts.len());
    let mut total_sentences = 0usize;
    let mut total_translated = 0usize;
    let mut useful = 0usize;

    for (index, alert) in alerts.iter().enumerate() {
        let kind = alert
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("?");
        let message = alert
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let rendered = panahon_core::advisory::render(message);
        total_sentences += rendered.total;
        total_translated += rendered.translated;
        if rendered.is_useful() {
            useful += 1;
        }
        if std::env::args().any(|arg| arg == "print") && index == 0 {
            println!("---- 译文 ----");
            println!("{}", rendered.text);
            println!("--------------");
        }
        let ratio = if rendered.total == 0 {
            0.0
        } else {
            rendered.translated as f32 / rendered.total as f32 * 100.0
        };
        for gap in &rendered.untranslated {
            println!("     未命中: {}", &gap[..gap.len().min(160)]);
        }
        println!(
            "  {:>2}. {:<24} 句子 {:>2}/{:<2} ({ratio:>5.1}%) 可用={}",
            index + 1,
            kind,
            rendered.translated,
            rendered.total,
            rendered.is_useful()
        );
    }

    let ratio = if total_sentences == 0 {
        0.0
    } else {
        total_translated as f32 / total_sentences as f32 * 100.0
    };
    println!(
        "\n合计：{total_translated}/{total_sentences} 句命中模板（{ratio:.1}%），{useful}/{} 条可用",
        alerts.len()
    );
}
