//! PAGASA 预警正文的中文渲染。
//!
//! PAGASA 的通报由少量固定句式组成。本模块只翻译能确定匹配的句子，其余保留英文
//! 原文，因此中文版本不会引入原文没有的信息。译文始终标注为自动翻译。

/// 一条通报的翻译结果。
#[derive(Clone, Debug, PartialEq)]
pub struct Rendered {
    /// 中文正文；无法识别的句子保留英文。
    pub text: String,
    /// 成功识别的句子数。
    pub translated: usize,
    /// 正文句子总数。
    pub total: usize,
    /// 没有命中模板、保留英文的句子。
    pub untranslated: Vec<String>,
}

impl Rendered {
    /// 至少一半句子识别成功才认为译文可用。
    pub fn is_useful(&self) -> bool {
        self.translated > 0 && self.translated * 2 >= self.total
    }
}

/// 把整条通报翻译成中文。无法识别的内容原样保留。
pub fn render(message: &str) -> Rendered {
    let mut translated = 0;
    let mut total = 0;
    let mut gaps: Vec<String> = Vec::new();
    let mut out: Vec<String> = Vec::new();

    for line in message.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(header) = translate_header(trimmed) {
            out.push(header);
            continue;
        }
        if let Some(issued) = translate_issued(trimmed) {
            out.push(issued);
            continue;
        }
        if let Some(system) = translate_weather_system(trimmed) {
            out.push(system);
            continue;
        }

        let mut paragraph = Vec::new();
        for sentence in split_sentences(trimmed) {
            total += 1;
            match translate_sentence(&sentence) {
                Some(chinese) => {
                    translated += 1;
                    paragraph.push(chinese);
                }
                None => {
                    gaps.push(sentence.clone());
                    paragraph.push(clean_place_marks(&sentence));
                }
            }
        }
        if !paragraph.is_empty() {
            out.push(paragraph.join(""));
        }
    }

    Rendered {
        text: out.join("\n"),
        translated,
        total,
        untranslated: gaps,
    }
}

fn translate_header(line: &str) -> Option<String> {
    let kinds = [
        ("Thunderstorm Advisory", "雷暴提示"),
        ("Thunderstorm Watch", "雷暴监测"),
        ("Thunderstorm Information", "雷暴信息"),
        ("Heavy Rainfall Warning", "强降雨警告"),
        ("Rainfall Advisory", "降雨提示"),
        ("Flood Advisory", "洪水提示"),
        ("Flood Warning", "洪水警告"),
        ("Gale Warning", "大风警告"),
    ];
    for (english, chinese) in kinds {
        if line.len() < english.len() || !line[..english.len()].eq_ignore_ascii_case(english) {
            continue;
        }
        let rest = line[english.len()..].trim();
        let number = rest
            .strip_prefix("No.")
            .map(str::trim)
            .and_then(|after| after.split_whitespace().next())
            .filter(|value| value.chars().all(|ch| ch.is_ascii_digit()));
        let code = rest
            .split_whitespace()
            .find(|word| word.starts_with('#'))
            .map(|word| word.trim_start_matches('#').to_owned());

        let mut text = String::from(chinese);
        if let Some(number) = number {
            text.push_str(&format!("第 {number} 号"));
        }
        if let Some(code) = code {
            text.push_str(&format!("（{code}）"));
        }
        return Some(text);
    }

    // "Weather System: ..." 之外的独立英文行不处理
    None
}

fn translate_issued(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let start = lower.find("issued at")? + "issued at".len();
    let rest = line[start..].trim_start_matches(|ch: char| ch == ':' || ch.is_whitespace());
    let (clock, day) = parse_issued(rest)?;
    Some(format!("发布于 {day} {clock}"))
}

fn parse_issued(value: &str) -> Option<(String, String)> {
    let cleaned = value.replace(',', " ");
    let tokens: Vec<&str> = cleaned.split_whitespace().collect();
    let mut hour = None;
    let mut minute = None;
    let mut meridiem = None;
    let mut day = None;
    let mut month = None;
    let mut year = None;

    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index].trim_matches(|ch: char| !ch.is_ascii_alphanumeric());
        if token.contains(':') && hour.is_none() {
            let mut parts = token.split(':');
            hour = parts.next().and_then(|part| part.parse::<u32>().ok());
            minute = parts
                .next()
                .and_then(|part| part.chars().take(2).collect::<String>().parse().ok());
        } else if token.eq_ignore_ascii_case("AM") || token.eq_ignore_ascii_case("PM") {
            meridiem = Some(token.to_ascii_uppercase());
        } else if let Some(month_number) = month_number(token) {
            month = Some(month_number);
        } else if day.is_none() && token.chars().all(|ch| ch.is_ascii_digit()) && token.len() <= 2 {
            day = token.parse::<u32>().ok();
        } else if year.is_none() {
            // "2026" 或 "2026(Tuesday)" 都取前四位数字
            let head: String = token.chars().take(4).collect();
            if head.len() == 4 && head.chars().all(|ch| ch.is_ascii_digit()) {
                year = head.parse::<u32>().ok();
            }
        }
        index += 1;
    }

    let mut hour = hour?;
    let minute = minute.unwrap_or(0);
    if let Some(meridiem) = meridiem {
        if meridiem == "PM" && hour != 12 {
            hour += 12;
        } else if meridiem == "AM" && hour == 12 {
            hour = 0;
        }
    }
    let day = day?;
    let month = month?;
    let year = year.unwrap_or(0);
    let date = if year > 0 {
        format!("{year}年{month}月{day}日")
    } else {
        format!("{month}月{day}日")
    };
    Some((format!("{hour:02}:{minute:02}"), date))
}

fn translate_weather_system(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if !lower.starts_with("weather system") {
        return None;
    }
    let value = line.split(':').skip(1).collect::<Vec<_>>().join(":").trim().to_owned();
    if value.is_empty() {
        return None;
    }
    Some(format!("天气系统：{}", translate_system_name(&value)))
}

fn translate_system_name(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    let name = if lower.contains("northeast monsoon") {
        "东北季风"
    } else if lower.contains("southwest monsoon") {
        "西南季风"
    } else if lower.contains("easterlies") {
        "东风带"
    } else if lower.contains("shear line") {
        "切变线"
    } else if lower.contains("intertropical convergence") || lower.contains("itcz") {
        "赤道辐合带"
    } else if lower.contains("low pressure area") || lower.contains("lpa") {
        "低压区"
    } else if lower.contains("tropical depression") {
        "热带低压"
    } else if lower.contains("tropical storm") {
        "热带风暴"
    } else if lower.contains("typhoon") {
        "台风"
    } else {
        return value.to_owned();
    };
    // 保留 PAGASA 括号中的本地名称，例如 (Amihan)
    match value.find('(') {
        Some(index) => format!("{name}{}", &value[index..]),
        None => name.to_owned(),
    }
}

fn split_sentences(paragraph: &str) -> Vec<String> {
    // 菲律宾地名里 Sto. / Sta. / Brgy. 很常见，不能当作句末
    const ABBREVIATIONS: [&str; 25] = [
        "sto", "sta", "st", "ste", "mt", "brgy", "no", "nos", "dr", "mr", "mrs", "ms", "jr", "sr",
        "gen", "gov", "lt", "pres", "capt", "col", "sgt", "hon", "sec", "rep", "atty",
    ];

    let chars: Vec<char> = paragraph.chars().collect();
    let mut sentences = Vec::new();
    let mut current = String::new();
    let mut index = 0;
    let mut depth = 0i32;

    while index < chars.len() {
        let ch = chars[index];
        current.push(ch);
        match ch {
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            _ => {}
        }

        let is_end = ch == '!' || ch == '?';
        let is_dot = ch == '.';
        let mut boundary = is_end && depth == 0;

        if is_dot && depth == 0 {
            // 取点号前最后一个词
            let mut word = String::new();
            for previous in current.chars().rev().skip(1) {
                if previous.is_ascii_alphabetic() {
                    word.insert(0, previous);
                } else {
                    break;
                }
            }
            // 单个大写字母（"Carlos P. Garcia"）也是缩写
            let single_initial = word.chars().count() == 1
                && word.chars().all(|ch| ch.is_ascii_uppercase());
            let is_abbreviation = single_initial
                || ABBREVIATIONS
                    .iter()
                    .any(|abbr| word.eq_ignore_ascii_case(abbr));
            // 下一个非空字符必须是大写或结尾，才算句末
            let next = chars[index + 1..]
                .iter()
                .find(|candidate| !candidate.is_whitespace())
                .copied();
            let next_upper = next.map(|candidate| candidate.is_uppercase()).unwrap_or(true);
            boundary = !is_abbreviation && next_upper;
        }

        if boundary {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_owned());
            }
            current.clear();
        }
        index += 1;
    }

    let tail = current.trim();
    if !tail.is_empty() {
        sentences.push(tail.to_owned());
    }
    sentences
}
fn translate_sentence(sentence: &str) -> Option<String> {
    let lower = sentence.to_ascii_lowercase();

    // 通用提醒句
    if lower.starts_with("all are advised to take precautionary measures") {
        return Some("请采取防范措施，警惕山洪和山体滑坡等风险。".into());
    }
    if lower.starts_with("all are advised to continue monitoring")
        || lower.starts_with("keep monitoring for updates")
        || lower.starts_with("continue monitoring for updates")
    {
        return Some("请持续关注最新通报。".into());
    }
    if lower.starts_with("flooding is possible") || lower.starts_with("flooding may occur") {
        return Some(format!(
            "低洼地区可能出现积水。{}",
            if lower.contains("landslide") {
                "山区需防范滑坡。"
            } else {
                ""
            }
        ));
    }
    if lower.starts_with("residents in") && lower.contains("advised to evacuate") {
        return Some("请相关地区居民留意官方疏散通知。".into());
    }

    // 雷暴可能 / 不可能
    if lower.starts_with("thunderstorm is more likely to develop over")
        || lower.starts_with("thunderstorms are more likely to develop over")
    {
        let places = places_after(sentence, "over")?;
        let hours = hours_within(sentence).unwrap_or(12);
        return Some(format!(
            "{places} 在未来 {hours} 小时内较可能出现雷暴。"
        ));
    }
    if lower.starts_with("thunderstorm is less likely to develop over")
        || lower.starts_with("thunderstorms are less likely to develop over")
    {
        let places = places_after(sentence, "over")?;
        let hours = hours_within(sentence).unwrap_or(12);
        return Some(format!(
            "{places} 在未来 {hours} 小时内较不可能出现雷暴。"
        ));
    }

    // "The above conditions are being experienced in X which may persist ..."
    if lower.starts_with("the above conditions are being experienced")
        || lower.starts_with("same conditions are being experienced")
    {
        let places = places_after(sentence, "in")?;
        let mut text = format!("{places} 正在出现上述天气");
        if let Some(duration) = duration_zh(sentence) {
            text.push_str(&format!("，可能持续 {duration}，并影响周边地区"));
        }
        text.push('。');
        return Some(text);
    }

    // 已经出现 / 预计出现的降雨
    if lower.contains("rainshowers") || lower.contains("rains") || lower.contains("rainfall") {
        let intensity = if lower.contains("moderate to heavy") {
            "中到大雨"
        } else if lower.contains("light to moderate to occasionally heavy") {
            "小到中雨，局地大雨"
        } else if lower.contains("light to moderate") {
            "小到中雨"
        } else if lower.contains("heavy") {
            "大雨"
        } else {
            "降雨"
        };
        let thunder = lower.contains("lightning") || lower.contains("thunder");
        let wind = lower.contains("strong wind");
        let places = places_after(sentence, "over")
            .or_else(|| places_after(sentence, "in"))
            .or_else(|| places_after(sentence, "at"))?;

        let mut text = String::new();
        if lower.contains("are being experienced") || lower.contains("is being experienced") {
            text.push_str(&format!("{places} 正在出现{intensity}"));
        } else {
            text.push_str(&format!("{places} 预计出现{intensity}"));
        }
        if thunder && wind {
            text.push_str("，伴有雷电和强风");
        } else if thunder {
            text.push_str("，伴有雷电");
        } else if wind {
            text.push_str("，伴有强风");
        }
        if let Some(duration) = duration_zh(sentence) {
            if lower.contains("next") {
                text.push_str(&format!("，预计在未来 {duration}内"));
            } else {
                text.push_str(&format!("，可能持续 {duration}"));
            }
        }
        text.push('。');
        return Some(text);
    }

    None
}

/// 去掉 # 号，并在驼峰式地名中补空格：`#NegrosOriental` → `Negros Oriental`。
fn clean_place_marks(value: &str) -> String {
    let without_hash = value.replace('#', "");
    let mut out = String::with_capacity(without_hash.len());
    let mut previous_lower = false;
    for ch in without_hash.chars() {
        if ch.is_ascii_uppercase() && previous_lower {
            out.push(' ');
        }
        previous_lower = ch.is_ascii_lowercase();
        out.push(ch);
    }
    out
}

/// 取出介词之后到句末的地点串。
fn places_after(sentence: &str, preposition: &str) -> Option<String> {
    let lower = sentence.to_ascii_lowercase();
    let needle = format!("{preposition} ");
    let start = lower.find(&needle)? + needle.len();
    let tail = &sentence[start..];
    let end = tail
        .find(|ch: char| ch == '.' || ch == ';')
        .unwrap_or(tail.len());
    let places = clean_place_marks(tail[..end].trim());
    if places.is_empty() {
        return None;
    }
    // 地点串后面常跟时间短语或从句，按关键词截断
    let mut places = places;
    for cut in [
        " which may persist",
        " and may affect",
        " within ",
        " in the next ",
        " are expected",
        " are being",
        " is expected",
    ] {
        if let Some(index) = places.to_ascii_lowercase().find(cut) {
            places.truncate(index);
        }
    }
    Some(places.trim().to_owned())
}

/// 把 "within 1 to 2 hours" / "within the next 30 minutes to an hour" 换成中文时长。
fn duration_zh(sentence: &str) -> Option<String> {
    let lower = sentence.to_ascii_lowercase();
    let start = lower.find("within")? + "within".len();
    let tail = &lower[start..];
    let end = tail
        .find("hour")
        .map(|index| index + 4)
        .or_else(|| tail.find("minute").map(|index| index + 6))?;
    let raw = tail[..end].trim().trim_start_matches("the next").trim();

    let unit = |text: &str| {
        if text.contains("minute") {
            "分钟"
        } else {
            "小时"
        }
    };
    let number = |text: &str| -> Option<u32> {
        if text.contains("an hour") || text.contains("a minute") {
            return Some(1);
        }
        text.split(|ch: char| !ch.is_ascii_digit())
            .find(|part| !part.is_empty())
            .and_then(|part| part.parse().ok())
    };

    match raw.split_once(" to ") {
        Some((first, second)) => {
            let from = number(first)?;
            let to = number(second)?;
            Some(if unit(first) == unit(second) {
                format!("{from} 至 {to} {}", unit(second))
            } else {
                format!("{from} {}至 {to} {}", unit(first), unit(second))
            })
        }
        None => {
            let value = number(raw)?;
            Some(format!("{value} {}", unit(raw)))
        }
    }
}
/// "within 1 to 2 hours" → (1, 2)
fn hour_range(sentence: &str) -> Option<(u32, u32)> {
    let lower = sentence.to_ascii_lowercase();
    let start = lower.find("within ")? + "within ".len();
    let tail = &lower[start..];
    let end = tail.find("hour")?;
    let numbers: Vec<u32> = tail[..end]
        .split(|ch: char| !ch.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect();
    match numbers.as_slice() {
        [from, to] => Some((*from, *to)),
        [single] => Some((*single, *single)),
        _ => None,
    }
}

/// "within 12 hours" → 12
fn hours_within(sentence: &str) -> Option<u32> {
    hour_range(sentence).map(|(from, _)| from)
}

fn month_number(name: &str) -> Option<u32> {
    let name = name.trim_matches(|ch: char| !ch.is_ascii_alphabetic());
    Some(match name.to_ascii_lowercase().as_str() {
        "january" | "jan" => 1,
        "february" | "feb" => 2,
        "march" | "mar" => 3,
        "april" | "apr" => 4,
        "may" => 5,
        "june" | "jun" => 6,
        "july" | "jul" => 7,
        "august" | "aug" => 8,
        "september" | "sep" | "sept" => 9,
        "october" | "oct" => 10,
        "november" | "nov" => 11,
        "december" | "dec" => 12,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const THUNDERSTORM: &str = "Thunderstorm Advisory No. 1  #SLPRSD\nIssued at 12:30 PM 10 November 2026\n\nModerate to heavy rainshowers with lightning and strong winds are being experienced in #Masbate(SanPascual and Claveria) which may persist within 1 to 2 hours and may affect nearby areas.\n\nAll are advised to take precautionary measures against the impacts associated with these hazards which include flash floods and landslides.\n\nKeep monitoring for updates.";

    const WATCH: &str = "Thunderstorm Watch #VISPRSD\nIssued at 10:00 AM 10 November 2026\n\nThunderstorm is MORE LIKELY to develop over #Cebu, #Bohol, #Siquijor, #NegrosOriental within 12 hours.\n\nAll are advised to continue monitoring for updates.";

    #[test]
    fn thunderstorm_advisory_is_rendered() {
        let rendered = render(THUNDERSTORM);
        assert!(rendered.is_useful());
        assert!(rendered.text.contains("雷暴提示第 1 号"));
        assert!(rendered.text.contains("2026年11月10日 12:30"));
        assert!(rendered.text.contains("中到大雨"));
        assert!(rendered.text.contains("雷电和强风"));
        assert!(rendered.text.contains("1 至 2 小时"));
        assert!(rendered.text.contains("防范措施"));
        assert!(rendered.text.contains("持续关注"));
        assert!(!rendered.text.contains('#'));
    }

    #[test]
    fn watch_translates_places_and_likelihood() {
        let rendered = render(WATCH);
        assert!(rendered.is_useful());
        assert!(rendered.text.contains("较可能出现雷暴"));
        assert!(rendered.text.contains("Cebu"));
        assert!(rendered.text.contains("Negros Oriental"));
    }

    #[test]
    fn unknown_sentences_stay_in_english() {
        let rendered = render("Thunderstorm Advisory #X\nIssued at 1:00 PM 1 January 2026\n\nSomething entirely new happened.");
        assert!(rendered.text.contains("Something entirely new happened."));
        assert!(!rendered.is_useful());
    }

    #[test]
    fn issued_date_variants_parse() {
        let rendered = render("Rainfall Advisory No. 2 #NCR_PRSD\nWeather System: Northeast Monsoon (Amihan)\nIssued at: 11:00 AM, 10 November 2026(Tuesday)\n\nLight to moderate rains are expected over Rizal(Tanay, Rodriguez) and Laguna(Santa Maria).");
        assert!(rendered.text.contains("天气系统：东北季风(Amihan)"), "got: {}", rendered.text);
        assert!(rendered.text.contains("发布于 2026年11月10日 11:00"), "got: {}", rendered.text);
        assert!(rendered.text.contains("小到中雨"));
    }
}
