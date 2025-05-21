use bson::DateTime;
use chrono::{NaiveDate, Utc};

/// 从日期字符串 ("YYYY-MM-DD" as UTC+8) 转换为 BSON DateTime 的起止时间
pub fn date_string_to_bson_range(date_str: &str) -> anyhow::Result<(DateTime, DateTime)> {
    // 解析日期字符串为 NaiveDate
    let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| anyhow::anyhow!("Invalid date format: {}", e))?;

    // 计算起止时间 UTC+8
    let start_of_day = naive_date
        .pred_opt()
        .and_then(|d| d.and_hms_opt(16, 0, 0))
        .ok_or(anyhow::anyhow!("Invalid start_of_day"))?
        .and_local_timezone(Utc)
        .single()
        .ok_or(anyhow::anyhow!(
            "Ambiguous or nonexistent start_of_day timezone"
        ))?;

    let end_of_day = naive_date
        .and_hms_opt(15, 59, 59)
        .ok_or(anyhow::anyhow!("Invalid end_of_day"))?
        .and_local_timezone(Utc)
        .single()
        .ok_or(anyhow::anyhow!(
            "Ambiguous or nonexistent end_of_day timezone"
        ))?;

    // 转换为 BSON DateTime
    let bson_start = DateTime::from_millis(start_of_day.timestamp_millis());
    let bson_end = DateTime::from_millis(end_of_day.timestamp_millis());

    Ok((bson_start, bson_end))
}

/// 从 BSON DateTime 转换回当地日期字符串 ("YYYY-MM-DD")
pub fn bson_to_date_string(bson_datetime: &DateTime) -> String {
    let chrono_datetime = bson_datetime.to_chrono().with_timezone(&chrono::Local);
    // println!("BSON DateTime: {:?}", bson_datetime);
    chrono_datetime.date_naive().to_string()
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_convert() {
        let date_str = "2025-05-21";
        let (start, end) = date_string_to_bson_range(date_str).unwrap();
        println!("Start: {}, End: {}", start, end);

        let bson_date_st = bson_to_date_string(&start);
        let bson_date_en = bson_to_date_string(&end);
        println!("BSON Start: {}, BSON End: {}", bson_date_st, bson_date_en);
        assert_eq!(date_str, bson_date_st);
        assert_eq!(date_str, bson_date_en);
    }
}
