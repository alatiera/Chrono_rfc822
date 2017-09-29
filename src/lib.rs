extern crate chrono;
extern crate regex;

use chrono::{DateTime, FixedOffset, ParseResult};
use regex::Regex;

use std::collections::HashMap;

// TODO: Setup Error-chain
pub fn sanitize_rfc822_like_date(s: &str) -> String {
    let mut foo = String::from(s);

    foo = pad_zeros(&foo);
    foo = remove_weekday(&foo);
    foo = replace_month(&foo);
    foo = replace_leading_zeros(&foo);

    // println!("{}", foo);
    foo
}

/// Pad HH:MM:SS with exta zeros if needed.
fn pad_zeros(s: &str) -> String {
    // If it matchers a pattern of 2:2:2, return.
    let ok = Regex::new(r"(\d{2}):(\d{2}):(\d{2})").unwrap();
    let skip = ok.find(&s);
    let mut foo = String::from(s);

    if let Some(_) = skip {
        return foo;
    }

    let re = Regex::new(r"(\d{1,2}):(\d{1,2}):(\d{1,2})").unwrap();
    // hours, minutes, seconds = cap[1], cap[2], cap[3]
    let cap = re.captures(&s).unwrap();
    let mut newtime = Vec::new();

    cap.iter()
        .skip(1)
        .map(|x| if let Some(y) = x {
            // if y.end() - y.start() == 1 {
            if y.as_str().len() == 1 {
                newtime.push(format!("0{}", y.as_str()));
            } else {
                newtime.push(y.as_str().to_string());
            }
        })
        // ignore this, it just discards the return value of map
        .fold((), |(), _| ());

    let ntime = &newtime.join(":");
    foo = foo.replace(cap.get(0).unwrap().as_str(), ntime);
    // println!("(\"{}\",\"{}\"),", s, foo);
    foo
}

/// Weekday name is not required for rfc2822
fn remove_weekday(s: &str) -> String {
    let weekdays = vec![
        "Mon,", "Tue,", "Wed,", "Thu,", "Fri,", "Sat,", "Sun,", "Monday,", "Tuesday,",
        "Wednesday,", "Thursday,", "Friday,", "Saturday,", "Sunday,",
    ];

    // IF anyone knows how to return from map like with return in the for loop,
    // please consider opening a pull request and uncommeting the following code,
    // or let me know with a mail or tweet.

    // let mut foo = String::from(s);
    // weekdays
    //     .iter()
    //     .map(|x| if foo.starts_with(x) {
    //         foo = format!("{}", &foo[x.len()..]);
    //         foo = foo.trim().to_string();
    //     })
    // ignore this, it just discards the return value of map
    //     .fold((), |(), _| ());

    // foo

    for d in weekdays {
        if s.contains(&d) {
            let foo = format!("{}", &s[d.len()..]).trim().to_string();
            return foo;
        }
    }

    s.to_string()
}

/// Replace long month names with 3 letter Abr as specified in RFC2822.
fn replace_month(s: &str) -> String {
    let mut months = HashMap::new();
    months.insert("January", "Jan");
    months.insert("February", "Feb");
    months.insert("March", "Mar");
    months.insert("April ", "Apr");
    months.insert("May", "May");
    months.insert("June", "Jun");
    months.insert("July", "Jul");
    months.insert("August", "Aug");
    months.insert("September", "Sep");
    months.insert("October", "Oct");
    months.insert("November", "Nov");
    months.insert("December", "Dec");

    // let mut foo = String::from(s);
    // months
    //     .iter()
    //     .map(|(k, v)| if s.contains(k) {
    //         foo = foo.replace(k, v);
    //     })
    //     ignore this, it just discards the return value of map
    //     .fold((), |(), _| ());
    // println!("(\"{}\",\"{}\"),", s, foo);
    // foo

    for (k, v) in months {
        if s.contains(&k) {
            return s.replace(&k, &v);
        }
    }

    s.to_string()
}

/// Convert -0000 to +0000
/// See #102, https://github.com/chronotope/chrono/issues/102
fn replace_leading_zeros(s: &str) -> String {
    if s.ends_with("-0000") {
        let foo = format!("{}+0000", &s[..s.len() - 5]);
        return foo;
    }

    s.to_string()
}

/// World is full of broken code and invalid rfc822/rfc2822 daytimes.
/// Higher order function that does what you wanted not what you said!
/// If it encounters an invalid daytime input it tries to fix it first.
///
/// This function acts like the normal DateTime::parse_from_rfc2822
/// would at first.
///
/// It calls DateTime::parse_from_rfc2822(s), if it succedes It returns the
/// normal result.
///
/// But if It fails, It will try to sanitize the String s, and fix common ways
/// date generators misshandle rfc822/rfc2822.
/// Then try to parse it again as DayTime.
///
/// BEWARE OF THE PERFORMANCE PENALTIES.
pub fn parse_from_rfc2822_with_fallback(s: &str) -> ParseResult<DateTime<FixedOffset>> {
    let date = DateTime::parse_from_rfc2822(&s);
    match date {
        Ok(_) => date,
        Err(err) => {
            let san = sanitize_rfc822_like_date(s);
            let dt = DateTime::parse_from_rfc2822(&san);
            if let Ok(_) = dt {
                return dt;
            } else {
                return Err(err);
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::DateTime;

    #[test]
    fn test_invalid_dates() {
        // left is raw date extracted from rss feeds.
        // right is corresponding valid rfc2822
        let dates = vec![
            ("Thu, 6 July 2017 15:30:00 PDT", "6 Jul 2017 15:30:00 PDT"),
            ("Mon, 10 July 2017 16:00:00 PDT", "10 Jul 2017 16:00:00 PDT"),
            ("Mon, 17 July 2017 17:00:00 PDT", "17 Jul 2017 17:00:00 PDT"),
            ("Mon, 24 July 2017 16:00:00 PDT", "24 Jul 2017 16:00:00 PDT"),
            ("Mon, 31 July 2017 16:00:00 PDT", "31 Jul 2017 16:00:00 PDT"),
            ("Thu, 30 Aug 2017 1:30:00 PDT", "30 Aug 2017 01:30:00 PDT"),
            (
                "Wed, 20 Sep 2017 10:00:00 -0000",
                "20 Sep 2017 10:00:00 +0000",
            ),
            (
                "Wed, 13 Sep 2017 10:00:00 -0000",
                "13 Sep 2017 10:00:00 +0000",
            ),
            (
                "Wed, 09 Aug 2017 10:00:00 -0000",
                "09 Aug 2017 10:00:00 +0000",
            ),
            (
                "Wed, 02 Aug 2017 10:00:00 -0000",
                "02 Aug 2017 10:00:00 +0000",
            ),
            (
                "Wed, 26 Jul 2017 10:00:00 -0000",
                "26 Jul 2017 10:00:00 +0000",
            ),
            (
                "Wed, 19 Jul 2017 10:00:00 -0000",
                "19 Jul 2017 10:00:00 +0000",
            ),
            (
                "Wed, 12 Jul 2017 10:00:00 -0000",
                "12 Jul 2017 10:00:00 +0000",
            ),
            (
                "Wed, 28 Jun 2017 10:00:00 -0000",
                "28 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 21 Jun 2017 10:00:00 -0000",
                "21 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 14 Jun 2017 10:00:00 -0000",
                "14 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 07 Jun 2017 10:00:00 -0000",
                "07 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 31 May 2017 10:00:00 -0000",
                "31 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 24 May 2017 10:00:00 -0000",
                "24 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 17 May 2017 10:00:00 -0000",
                "17 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 10 May 2017 10:00:00 -0000",
                "10 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 03 May 2017 10:00:00 -0000",
                "03 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 19 Apr 2017 10:00:00 -0000",
                "19 Apr 2017 10:00:00 +0000",
            ),
            (
                "Wed, 12 Apr 2017 10:00:00 -0000",
                "12 Apr 2017 10:00:00 +0000",
            ),
            (
                "Wed, 05 Apr 2017 10:00:00 -0000",
                "05 Apr 2017 10:00:00 +0000",
            ),
            (
                "Wed, 29 Mar 2017 10:00:00 -0000",
                "29 Mar 2017 10:00:00 +0000",
            ),
            (
                "Wed, 22 Mar 2017 10:00:00 -0000",
                "22 Mar 2017 10:00:00 +0000",
            ),
            (
                "Wed, 15 Mar 2017 10:00:00 -0000",
                "15 Mar 2017 10:00:00 +0000",
            ),
            (
                "Wed, 08 Mar 2017 11:00:00 -0000",
                "08 Mar 2017 11:00:00 +0000",
            ),
            (
                "Wed, 01 Mar 2017 11:00:00 -0000",
                "01 Mar 2017 11:00:00 +0000",
            ),
            (
                "Wed, 22 Feb 2017 11:00:00 -0000",
                "22 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 15 Feb 2017 11:00:00 -0000",
                "15 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 08 Feb 2017 11:00:00 -0000",
                "08 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 01 Feb 2017 11:00:00 -0000",
                "01 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 25 Jan 2017 11:00:00 -0000",
                "25 Jan 2017 11:00:00 +0000",
            ),
            (
                "Fri, 13 Jan 2017 18:38:00 -0000",
                "13 Jan 2017 18:38:00 +0000",
            ),
            (
                "Wed, 20 Sep 2017 03:30:00 -0000",
                "20 Sep 2017 03:30:00 +0000",
            ),
            (
                "Wed, 13 Sep 2017 03:15:00 -0000",
                "13 Sep 2017 03:15:00 +0000",
            ),
            (
                "Wed, 06 Sep 2017 03:15:00 -0000",
                "06 Sep 2017 03:15:00 +0000",
            ),
            (
                "Wed, 30 Aug 2017 03:15:00 -0000",
                "30 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 23 Aug 2017 03:15:00 -0000",
                "23 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 16 Aug 2017 03:15:00 -0000",
                "16 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 09 Aug 2017 03:15:00 -0000",
                "09 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 02 Aug 2017 03:00:00 -0000",
                "02 Aug 2017 03:00:00 +0000",
            ),
            (
                "Tue, 11 Jul 2017 17:14:45 -0000",
                "11 Jul 2017 17:14:45 +0000",
            ),
            (
                "Thu, 03 August 2017 06:00:00 -0400",
                "03 Aug 2017 06:00:00 -0400",
            ),
            (
                "Thu, 27 July 2017 06:00:00 -0400",
                "27 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 20 July 2017 06:00:00 -0400",
                "20 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 13 July 2017 06:00:00 -0400",
                "13 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 06 July 2017 06:00:00 -0400",
                "06 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 28 June 2017 06:00:00 -0400",
                "28 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 17 Jul 2013 06:00:03 -0400",
                "17 Jul 2013 06:00:03 -0400",
            ),
            (
                "Thu, 02 Apr 2014 06:00:03 -0400",
                "02 Apr 2014 06:00:03 -0400",
            ),
            (
                "Wed, 14 Jan 2016 06:00:03 -0400",
                "14 Jan 2016 06:00:03 -0400",
            ),
            (
                "Thu, 22 June 2017 06:00:00 -0400",
                "22 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 June 2017 06:00:00 -0400",
                "15 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 7 June 2017 06:00:00 -0400",
                "7 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 1 June 2017 06:00:00 -0400",
                "1 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 23 Dec 2015 06:00:03 -0400",
                "23 Dec 2015 06:00:03 -0400",
            ),
            (
                "Thu, 14 Feb 2014 06:00:03 -0400",
                "14 Feb 2014 06:00:03 -0400",
            ),
            (
                "Thu, 04 Dec 2013 06:00:03 -0400",
                "04 Dec 2013 06:00:03 -0400",
            ),
            (
                "Thu, 20 Dec 2016 06:00:00 -0400",
                "20 Dec 2016 06:00:00 -0400",
            ),
            (
                "Thu, 23 Nov 2016 06:00:00 -0400",
                "23 Nov 2016 06:00:00 -0400",
            ),
            (
                "Thu, 05 Aug 2016 06:00:00 -0400",
                "05 Aug 2016 06:00:00 -0400",
            ),
            (
                "Fri, 09 Jun 2016 12:00:00 -0400",
                "09 Jun 2016 12:00:00 -0400",
            ),
            (
                "Thu, 10 May 2017 06:00:00 -0400",
                "10 May 2017 06:00:00 -0400",
            ),
            (
                "Thu, 22 Feb 2017 06:00:00 -0400",
                "22 Feb 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 Feb 2017 06:00:00 -0400",
                "15 Feb 2017 06:00:00 -0400",
            ),
        ];

        dates
            .iter()
            .map(|&(bad, good)| {
                assert_eq!(
                    parse_from_rfc2822_with_fallback(bad),
                    DateTime::parse_from_rfc2822(good)
                )
            })
            .fold((), |(), _| ());
    }


    #[test]
    fn test_sanitize_rfc822_like_date() {
        // left is raw date extracted from rss feeds.
        // right is corresponding valid rfc2822
        let dates = vec![
            ("Thu, 6 July 2017 15:30:00 PDT", "6 Jul 2017 15:30:00 PDT"),
            ("Mon, 10 July 2017 16:00:00 PDT", "10 Jul 2017 16:00:00 PDT"),
            ("Mon, 17 July 2017 17:00:00 PDT", "17 Jul 2017 17:00:00 PDT"),
            ("Mon, 24 July 2017 16:00:00 PDT", "24 Jul 2017 16:00:00 PDT"),
            ("Mon, 31 July 2017 16:00:00 PDT", "31 Jul 2017 16:00:00 PDT"),
            ("Thu, 30 Aug 2017 1:30:00 PDT", "30 Aug 2017 01:30:00 PDT"),
            (
                "Wed, 20 Sep 2017 10:00:00 -0000",
                "20 Sep 2017 10:00:00 +0000",
            ),
            (
                "Wed, 13 Sep 2017 10:00:00 -0000",
                "13 Sep 2017 10:00:00 +0000",
            ),
            (
                "Wed, 09 Aug 2017 10:00:00 -0000",
                "09 Aug 2017 10:00:00 +0000",
            ),
            (
                "Wed, 02 Aug 2017 10:00:00 -0000",
                "02 Aug 2017 10:00:00 +0000",
            ),
            (
                "Wed, 26 Jul 2017 10:00:00 -0000",
                "26 Jul 2017 10:00:00 +0000",
            ),
            (
                "Wed, 19 Jul 2017 10:00:00 -0000",
                "19 Jul 2017 10:00:00 +0000",
            ),
            (
                "Wed, 12 Jul 2017 10:00:00 -0000",
                "12 Jul 2017 10:00:00 +0000",
            ),
            (
                "Wed, 28 Jun 2017 10:00:00 -0000",
                "28 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 21 Jun 2017 10:00:00 -0000",
                "21 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 14 Jun 2017 10:00:00 -0000",
                "14 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 07 Jun 2017 10:00:00 -0000",
                "07 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 31 May 2017 10:00:00 -0000",
                "31 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 24 May 2017 10:00:00 -0000",
                "24 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 17 May 2017 10:00:00 -0000",
                "17 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 10 May 2017 10:00:00 -0000",
                "10 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 03 May 2017 10:00:00 -0000",
                "03 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 19 Apr 2017 10:00:00 -0000",
                "19 Apr 2017 10:00:00 +0000",
            ),
            (
                "Wed, 12 Apr 2017 10:00:00 -0000",
                "12 Apr 2017 10:00:00 +0000",
            ),
            (
                "Wed, 05 Apr 2017 10:00:00 -0000",
                "05 Apr 2017 10:00:00 +0000",
            ),
            (
                "Wed, 29 Mar 2017 10:00:00 -0000",
                "29 Mar 2017 10:00:00 +0000",
            ),
            (
                "Wed, 22 Mar 2017 10:00:00 -0000",
                "22 Mar 2017 10:00:00 +0000",
            ),
            (
                "Wed, 15 Mar 2017 10:00:00 -0000",
                "15 Mar 2017 10:00:00 +0000",
            ),
            (
                "Wed, 08 Mar 2017 11:00:00 -0000",
                "08 Mar 2017 11:00:00 +0000",
            ),
            (
                "Wed, 01 Mar 2017 11:00:00 -0000",
                "01 Mar 2017 11:00:00 +0000",
            ),
            (
                "Wed, 22 Feb 2017 11:00:00 -0000",
                "22 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 15 Feb 2017 11:00:00 -0000",
                "15 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 08 Feb 2017 11:00:00 -0000",
                "08 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 01 Feb 2017 11:00:00 -0000",
                "01 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 25 Jan 2017 11:00:00 -0000",
                "25 Jan 2017 11:00:00 +0000",
            ),
            (
                "Fri, 13 Jan 2017 18:38:00 -0000",
                "13 Jan 2017 18:38:00 +0000",
            ),
            (
                "Wed, 20 Sep 2017 03:30:00 -0000",
                "20 Sep 2017 03:30:00 +0000",
            ),
            (
                "Wed, 13 Sep 2017 03:15:00 -0000",
                "13 Sep 2017 03:15:00 +0000",
            ),
            (
                "Wed, 06 Sep 2017 03:15:00 -0000",
                "06 Sep 2017 03:15:00 +0000",
            ),
            (
                "Wed, 30 Aug 2017 03:15:00 -0000",
                "30 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 23 Aug 2017 03:15:00 -0000",
                "23 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 16 Aug 2017 03:15:00 -0000",
                "16 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 09 Aug 2017 03:15:00 -0000",
                "09 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 02 Aug 2017 03:00:00 -0000",
                "02 Aug 2017 03:00:00 +0000",
            ),
            (
                "Tue, 11 Jul 2017 17:14:45 -0000",
                "11 Jul 2017 17:14:45 +0000",
            ),
            (
                "Thu, 03 August 2017 06:00:00 -0400",
                "03 Aug 2017 06:00:00 -0400",
            ),
            (
                "Thu, 27 July 2017 06:00:00 -0400",
                "27 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 20 July 2017 06:00:00 -0400",
                "20 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 13 July 2017 06:00:00 -0400",
                "13 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 06 July 2017 06:00:00 -0400",
                "06 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 28 June 2017 06:00:00 -0400",
                "28 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 17 Jul 2013 06:00:03 -0400",
                "17 Jul 2013 06:00:03 -0400",
            ),
            (
                "Thu, 02 Apr 2014 06:00:03 -0400",
                "02 Apr 2014 06:00:03 -0400",
            ),
            (
                "Wed, 14 Jan 2016 06:00:03 -0400",
                "14 Jan 2016 06:00:03 -0400",
            ),
            (
                "Thu, 22 June 2017 06:00:00 -0400",
                "22 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 June 2017 06:00:00 -0400",
                "15 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 7 June 2017 06:00:00 -0400",
                "7 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 1 June 2017 06:00:00 -0400",
                "1 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 23 Dec 2015 06:00:03 -0400",
                "23 Dec 2015 06:00:03 -0400",
            ),
            (
                "Thu, 14 Feb 2014 06:00:03 -0400",
                "14 Feb 2014 06:00:03 -0400",
            ),
            (
                "Thu, 04 Dec 2013 06:00:03 -0400",
                "04 Dec 2013 06:00:03 -0400",
            ),
            (
                "Thu, 20 Dec 2016 06:00:00 -0400",
                "20 Dec 2016 06:00:00 -0400",
            ),
            (
                "Thu, 23 Nov 2016 06:00:00 -0400",
                "23 Nov 2016 06:00:00 -0400",
            ),
            (
                "Thu, 05 Aug 2016 06:00:00 -0400",
                "05 Aug 2016 06:00:00 -0400",
            ),
            (
                "Fri, 09 Jun 2016 12:00:00 -0400",
                "09 Jun 2016 12:00:00 -0400",
            ),
            (
                "Thu, 10 May 2017 06:00:00 -0400",
                "10 May 2017 06:00:00 -0400",
            ),
            (
                "Thu, 22 Feb 2017 06:00:00 -0400",
                "22 Feb 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 Feb 2017 06:00:00 -0400",
                "15 Feb 2017 06:00:00 -0400",
            ),
        ];

        dates
            .iter()
            .map(|&(bad, good)| {
                assert_eq!(sanitize_rfc822_like_date(bad), good)
            })
            .fold((), |(), _| ());
    }

    #[test]
    fn test_remove_weekday() {
        let foo = vec![
            ("Thu, 6 July 2017 15:30:00 PDT", "6 July 2017 15:30:00 PDT"),
            (
                "Mon, 10 July 2017 16:00:00 PDT",
                "10 July 2017 16:00:00 PDT",
            ),
            (
                "Mon, 17 July 2017 17:00:00 PDT",
                "17 July 2017 17:00:00 PDT",
            ),
            (
                "Mon, 24 July 2017 16:00:00 PDT",
                "24 July 2017 16:00:00 PDT",
            ),
            (
                "Mon, 31 July 2017 16:00:00 PDT",
                "31 July 2017 16:00:00 PDT",
            ),
            ("Thu, 30 Aug 2017 1:30:00 PDT", "30 Aug 2017 1:30:00 PDT"),
            (
                "Wed, 20 Sep 2017 10:00:00 -0000",
                "20 Sep 2017 10:00:00 -0000",
            ),
            (
                "Wed, 13 Sep 2017 10:00:00 -0000",
                "13 Sep 2017 10:00:00 -0000",
            ),
            (
                "Wed, 09 Aug 2017 10:00:00 -0000",
                "09 Aug 2017 10:00:00 -0000",
            ),
            (
                "Wed, 02 Aug 2017 10:00:00 -0000",
                "02 Aug 2017 10:00:00 -0000",
            ),
            (
                "Wed, 26 Jul 2017 10:00:00 -0000",
                "26 Jul 2017 10:00:00 -0000",
            ),
            (
                "Wed, 19 Jul 2017 10:00:00 -0000",
                "19 Jul 2017 10:00:00 -0000",
            ),
            (
                "Wed, 12 Jul 2017 10:00:00 -0000",
                "12 Jul 2017 10:00:00 -0000",
            ),
            (
                "Wed, 28 Jun 2017 10:00:00 -0000",
                "28 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 21 Jun 2017 10:00:00 -0000",
                "21 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 14 Jun 2017 10:00:00 -0000",
                "14 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 07 Jun 2017 10:00:00 -0000",
                "07 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 31 May 2017 10:00:00 -0000",
                "31 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 24 May 2017 10:00:00 -0000",
                "24 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 17 May 2017 10:00:00 -0000",
                "17 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 10 May 2017 10:00:00 -0000",
                "10 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 03 May 2017 10:00:00 -0000",
                "03 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 19 Apr 2017 10:00:00 -0000",
                "19 Apr 2017 10:00:00 -0000",
            ),
            (
                "Wed, 12 Apr 2017 10:00:00 -0000",
                "12 Apr 2017 10:00:00 -0000",
            ),
            (
                "Wed, 05 Apr 2017 10:00:00 -0000",
                "05 Apr 2017 10:00:00 -0000",
            ),
            (
                "Wed, 29 Mar 2017 10:00:00 -0000",
                "29 Mar 2017 10:00:00 -0000",
            ),
            (
                "Wed, 22 Mar 2017 10:00:00 -0000",
                "22 Mar 2017 10:00:00 -0000",
            ),
            (
                "Wed, 15 Mar 2017 10:00:00 -0000",
                "15 Mar 2017 10:00:00 -0000",
            ),
            (
                "Wed, 08 Mar 2017 11:00:00 -0000",
                "08 Mar 2017 11:00:00 -0000",
            ),
            (
                "Wed, 01 Mar 2017 11:00:00 -0000",
                "01 Mar 2017 11:00:00 -0000",
            ),
            (
                "Wed, 22 Feb 2017 11:00:00 -0000",
                "22 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 15 Feb 2017 11:00:00 -0000",
                "15 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 08 Feb 2017 11:00:00 -0000",
                "08 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 01 Feb 2017 11:00:00 -0000",
                "01 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 25 Jan 2017 11:00:00 -0000",
                "25 Jan 2017 11:00:00 -0000",
            ),
            (
                "Fri, 13 Jan 2017 18:38:00 -0000",
                "13 Jan 2017 18:38:00 -0000",
            ),
            (
                "Wed, 20 Sep 2017 03:30:00 -0000",
                "20 Sep 2017 03:30:00 -0000",
            ),
            (
                "Wed, 13 Sep 2017 03:15:00 -0000",
                "13 Sep 2017 03:15:00 -0000",
            ),
            (
                "Wed, 06 Sep 2017 03:15:00 -0000",
                "06 Sep 2017 03:15:00 -0000",
            ),
            (
                "Wed, 30 Aug 2017 03:15:00 -0000",
                "30 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 23 Aug 2017 03:15:00 -0000",
                "23 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 16 Aug 2017 03:15:00 -0000",
                "16 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 09 Aug 2017 03:15:00 -0000",
                "09 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 02 Aug 2017 03:00:00 -0000",
                "02 Aug 2017 03:00:00 -0000",
            ),
            (
                "Tue, 11 Jul 2017 17:14:45 -0000",
                "11 Jul 2017 17:14:45 -0000",
            ),
            (
                "Thu, 03 August 2017 06:00:00 -0400",
                "03 August 2017 06:00:00 -0400",
            ),
            (
                "Thu, 27 July 2017 06:00:00 -0400",
                "27 July 2017 06:00:00 -0400",
            ),
            (
                "Thu, 20 July 2017 06:00:00 -0400",
                "20 July 2017 06:00:00 -0400",
            ),
            (
                "Thu, 13 July 2017 06:00:00 -0400",
                "13 July 2017 06:00:00 -0400",
            ),
            (
                "Thu, 06 July 2017 06:00:00 -0400",
                "06 July 2017 06:00:00 -0400",
            ),
            (
                "Thu, 28 June 2017 06:00:00 -0400",
                "28 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 17 Jul 2013 06:00:03 -0400",
                "17 Jul 2013 06:00:03 -0400",
            ),
            (
                "Thu, 02 Apr 2014 06:00:03 -0400",
                "02 Apr 2014 06:00:03 -0400",
            ),
            (
                "Wed, 14 Jan 2016 06:00:03 -0400",
                "14 Jan 2016 06:00:03 -0400",
            ),
            (
                "Thu, 22 June 2017 06:00:00 -0400",
                "22 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 June 2017 06:00:00 -0400",
                "15 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 7 June 2017 06:00:00 -0400",
                "7 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 1 June 2017 06:00:00 -0400",
                "1 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 23 Dec 2015 06:00:03 -0400",
                "23 Dec 2015 06:00:03 -0400",
            ),
            (
                "Thu, 14 Feb 2014 06:00:03 -0400",
                "14 Feb 2014 06:00:03 -0400",
            ),
            (
                "Thu, 04 Dec 2013 06:00:03 -0400",
                "04 Dec 2013 06:00:03 -0400",
            ),
            (
                "Thu, 20 Dec 2016 06:00:00 -0400",
                "20 Dec 2016 06:00:00 -0400",
            ),
            (
                "Thu, 23 Nov 2016 06:00:00 -0400",
                "23 Nov 2016 06:00:00 -0400",
            ),
            (
                "Thu, 05 Aug 2016 06:00:00 -0400",
                "05 Aug 2016 06:00:00 -0400",
            ),
            (
                "Fri, 09 Jun 2016 12:00:00 -0400",
                "09 Jun 2016 12:00:00 -0400",
            ),
            (
                "Thu, 10 May 2017 06:00:00 -0400",
                "10 May 2017 06:00:00 -0400",
            ),
            (
                "Thu, 22 Feb 2017 06:00:00 -0400",
                "22 Feb 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 Feb 2017 06:00:00 -0400",
                "15 Feb 2017 06:00:00 -0400",
            ),
        ];

        foo.iter()
            .map(|&(bad, good)| assert_eq!(remove_weekday(bad), good))
            .fold((), |(), _| ());
    }

    #[test]
    fn test_pad_zeros() {
        // Would be nice If we had more test cases,
        // If you stuble(d) upon any online please consider opening a Pullrequest.
        let foo = vec![
            (
                "Thu, 30 Aug 2017 1:30:00 PDT",
                "Thu, 30 Aug 2017 01:30:00 PDT",
            ),
        ];

        foo.iter()
            .map(|&(bad, good)| assert_eq!(pad_zeros(bad), good))
            .fold((), |(), _| ());
    }

    #[test]
    fn test_replace_month() {
        let foo = vec![
            (
                "Thu, 6 July 2017 15:30:00 PDT",
                "Thu, 6 Jul 2017 15:30:00 PDT",
            ),
            (
                "Thu, 6 July 2017 15:30:00 PDT",
                "Thu, 6 Jul 2017 15:30:00 PDT",
            ),
            (
                "Mon, 10 July 2017 16:00:00 PDT",
                "Mon, 10 Jul 2017 16:00:00 PDT",
            ),
            (
                "Mon, 10 July 2017 16:00:00 PDT",
                "Mon, 10 Jul 2017 16:00:00 PDT",
            ),
            (
                "Mon, 17 July 2017 17:00:00 PDT",
                "Mon, 17 Jul 2017 17:00:00 PDT",
            ),
            (
                "Mon, 17 July 2017 17:00:00 PDT",
                "Mon, 17 Jul 2017 17:00:00 PDT",
            ),
            (
                "Mon, 24 July 2017 16:00:00 PDT",
                "Mon, 24 Jul 2017 16:00:00 PDT",
            ),
            (
                "Mon, 24 July 2017 16:00:00 PDT",
                "Mon, 24 Jul 2017 16:00:00 PDT",
            ),
            (
                "Mon, 31 July 2017 16:00:00 PDT",
                "Mon, 31 Jul 2017 16:00:00 PDT",
            ),
            (
                "Mon, 31 July 2017 16:00:00 PDT",
                "Mon, 31 Jul 2017 16:00:00 PDT",
            ),
            (
                "Thu, 30 Aug 2017 1:30:00 PDT",
                "Thu, 30 Aug 2017 1:30:00 PDT",
            ),
            (
                "Thu, 30 Aug 2017 1:30:00 PDT",
                "Thu, 30 Aug 2017 1:30:00 PDT",
            ),
            (
                "Wed, 20 Sep 2017 10:00:00 -0000",
                "Wed, 20 Sep 2017 10:00:00 -0000",
            ),
            (
                "Wed, 20 Sep 2017 10:00:00 -0000",
                "Wed, 20 Sep 2017 10:00:00 -0000",
            ),
            (
                "Wed, 13 Sep 2017 10:00:00 -0000",
                "Wed, 13 Sep 2017 10:00:00 -0000",
            ),
            (
                "Wed, 13 Sep 2017 10:00:00 -0000",
                "Wed, 13 Sep 2017 10:00:00 -0000",
            ),
            (
                "Wed, 09 Aug 2017 10:00:00 -0000",
                "Wed, 09 Aug 2017 10:00:00 -0000",
            ),
            (
                "Wed, 09 Aug 2017 10:00:00 -0000",
                "Wed, 09 Aug 2017 10:00:00 -0000",
            ),
            (
                "Wed, 02 Aug 2017 10:00:00 -0000",
                "Wed, 02 Aug 2017 10:00:00 -0000",
            ),
            (
                "Wed, 02 Aug 2017 10:00:00 -0000",
                "Wed, 02 Aug 2017 10:00:00 -0000",
            ),
            (
                "Wed, 26 Jul 2017 10:00:00 -0000",
                "Wed, 26 Jul 2017 10:00:00 -0000",
            ),
            (
                "Wed, 26 Jul 2017 10:00:00 -0000",
                "Wed, 26 Jul 2017 10:00:00 -0000",
            ),
            (
                "Wed, 19 Jul 2017 10:00:00 -0000",
                "Wed, 19 Jul 2017 10:00:00 -0000",
            ),
            (
                "Wed, 19 Jul 2017 10:00:00 -0000",
                "Wed, 19 Jul 2017 10:00:00 -0000",
            ),
            (
                "Wed, 12 Jul 2017 10:00:00 -0000",
                "Wed, 12 Jul 2017 10:00:00 -0000",
            ),
            (
                "Wed, 12 Jul 2017 10:00:00 -0000",
                "Wed, 12 Jul 2017 10:00:00 -0000",
            ),
            (
                "Wed, 28 Jun 2017 10:00:00 -0000",
                "Wed, 28 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 28 Jun 2017 10:00:00 -0000",
                "Wed, 28 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 21 Jun 2017 10:00:00 -0000",
                "Wed, 21 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 21 Jun 2017 10:00:00 -0000",
                "Wed, 21 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 14 Jun 2017 10:00:00 -0000",
                "Wed, 14 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 14 Jun 2017 10:00:00 -0000",
                "Wed, 14 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 07 Jun 2017 10:00:00 -0000",
                "Wed, 07 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 07 Jun 2017 10:00:00 -0000",
                "Wed, 07 Jun 2017 10:00:00 -0000",
            ),
            (
                "Wed, 31 May 2017 10:00:00 -0000",
                "Wed, 31 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 31 May 2017 10:00:00 -0000",
                "Wed, 31 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 24 May 2017 10:00:00 -0000",
                "Wed, 24 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 24 May 2017 10:00:00 -0000",
                "Wed, 24 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 17 May 2017 10:00:00 -0000",
                "Wed, 17 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 17 May 2017 10:00:00 -0000",
                "Wed, 17 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 10 May 2017 10:00:00 -0000",
                "Wed, 10 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 10 May 2017 10:00:00 -0000",
                "Wed, 10 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 03 May 2017 10:00:00 -0000",
                "Wed, 03 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 03 May 2017 10:00:00 -0000",
                "Wed, 03 May 2017 10:00:00 -0000",
            ),
            (
                "Wed, 19 Apr 2017 10:00:00 -0000",
                "Wed, 19 Apr 2017 10:00:00 -0000",
            ),
            (
                "Wed, 19 Apr 2017 10:00:00 -0000",
                "Wed, 19 Apr 2017 10:00:00 -0000",
            ),
            (
                "Wed, 12 Apr 2017 10:00:00 -0000",
                "Wed, 12 Apr 2017 10:00:00 -0000",
            ),
            (
                "Wed, 12 Apr 2017 10:00:00 -0000",
                "Wed, 12 Apr 2017 10:00:00 -0000",
            ),
            (
                "Wed, 05 Apr 2017 10:00:00 -0000",
                "Wed, 05 Apr 2017 10:00:00 -0000",
            ),
            (
                "Wed, 05 Apr 2017 10:00:00 -0000",
                "Wed, 05 Apr 2017 10:00:00 -0000",
            ),
            (
                "Wed, 29 Mar 2017 10:00:00 -0000",
                "Wed, 29 Mar 2017 10:00:00 -0000",
            ),
            (
                "Wed, 29 Mar 2017 10:00:00 -0000",
                "Wed, 29 Mar 2017 10:00:00 -0000",
            ),
            (
                "Wed, 22 Mar 2017 10:00:00 -0000",
                "Wed, 22 Mar 2017 10:00:00 -0000",
            ),
            (
                "Wed, 22 Mar 2017 10:00:00 -0000",
                "Wed, 22 Mar 2017 10:00:00 -0000",
            ),
            (
                "Wed, 15 Mar 2017 10:00:00 -0000",
                "Wed, 15 Mar 2017 10:00:00 -0000",
            ),
            (
                "Wed, 15 Mar 2017 10:00:00 -0000",
                "Wed, 15 Mar 2017 10:00:00 -0000",
            ),
            (
                "Wed, 08 Mar 2017 11:00:00 -0000",
                "Wed, 08 Mar 2017 11:00:00 -0000",
            ),
            (
                "Wed, 08 Mar 2017 11:00:00 -0000",
                "Wed, 08 Mar 2017 11:00:00 -0000",
            ),
            (
                "Wed, 01 Mar 2017 11:00:00 -0000",
                "Wed, 01 Mar 2017 11:00:00 -0000",
            ),
            (
                "Wed, 01 Mar 2017 11:00:00 -0000",
                "Wed, 01 Mar 2017 11:00:00 -0000",
            ),
            (
                "Wed, 22 Feb 2017 11:00:00 -0000",
                "Wed, 22 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 22 Feb 2017 11:00:00 -0000",
                "Wed, 22 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 15 Feb 2017 11:00:00 -0000",
                "Wed, 15 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 15 Feb 2017 11:00:00 -0000",
                "Wed, 15 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 08 Feb 2017 11:00:00 -0000",
                "Wed, 08 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 08 Feb 2017 11:00:00 -0000",
                "Wed, 08 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 01 Feb 2017 11:00:00 -0000",
                "Wed, 01 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 01 Feb 2017 11:00:00 -0000",
                "Wed, 01 Feb 2017 11:00:00 -0000",
            ),
            (
                "Wed, 25 Jan 2017 11:00:00 -0000",
                "Wed, 25 Jan 2017 11:00:00 -0000",
            ),
            (
                "Wed, 25 Jan 2017 11:00:00 -0000",
                "Wed, 25 Jan 2017 11:00:00 -0000",
            ),
            (
                "Fri, 13 Jan 2017 18:38:00 -0000",
                "Fri, 13 Jan 2017 18:38:00 -0000",
            ),
            (
                "Fri, 13 Jan 2017 18:38:00 -0000",
                "Fri, 13 Jan 2017 18:38:00 -0000",
            ),
            (
                "Wed, 20 Sep 2017 03:30:00 -0000",
                "Wed, 20 Sep 2017 03:30:00 -0000",
            ),
            (
                "Wed, 20 Sep 2017 03:30:00 -0000",
                "Wed, 20 Sep 2017 03:30:00 -0000",
            ),
            (
                "Wed, 13 Sep 2017 03:15:00 -0000",
                "Wed, 13 Sep 2017 03:15:00 -0000",
            ),
            (
                "Wed, 13 Sep 2017 03:15:00 -0000",
                "Wed, 13 Sep 2017 03:15:00 -0000",
            ),
            (
                "Wed, 06 Sep 2017 03:15:00 -0000",
                "Wed, 06 Sep 2017 03:15:00 -0000",
            ),
            (
                "Wed, 06 Sep 2017 03:15:00 -0000",
                "Wed, 06 Sep 2017 03:15:00 -0000",
            ),
            (
                "Wed, 30 Aug 2017 03:15:00 -0000",
                "Wed, 30 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 30 Aug 2017 03:15:00 -0000",
                "Wed, 30 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 23 Aug 2017 03:15:00 -0000",
                "Wed, 23 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 23 Aug 2017 03:15:00 -0000",
                "Wed, 23 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 16 Aug 2017 03:15:00 -0000",
                "Wed, 16 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 16 Aug 2017 03:15:00 -0000",
                "Wed, 16 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 09 Aug 2017 03:15:00 -0000",
                "Wed, 09 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 09 Aug 2017 03:15:00 -0000",
                "Wed, 09 Aug 2017 03:15:00 -0000",
            ),
            (
                "Wed, 02 Aug 2017 03:00:00 -0000",
                "Wed, 02 Aug 2017 03:00:00 -0000",
            ),
            (
                "Wed, 02 Aug 2017 03:00:00 -0000",
                "Wed, 02 Aug 2017 03:00:00 -0000",
            ),
            (
                "Tue, 11 Jul 2017 17:14:45 -0000",
                "Tue, 11 Jul 2017 17:14:45 -0000",
            ),
            (
                "Tue, 11 Jul 2017 17:14:45 -0000",
                "Tue, 11 Jul 2017 17:14:45 -0000",
            ),
            (
                "Thu, 03 August 2017 06:00:00 -0400",
                "Thu, 03 Aug 2017 06:00:00 -0400",
            ),
            (
                "Thu, 03 August 2017 06:00:00 -0400",
                "Thu, 03 Aug 2017 06:00:00 -0400",
            ),
            (
                "Thu, 27 July 2017 06:00:00 -0400",
                "Thu, 27 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 27 July 2017 06:00:00 -0400",
                "Thu, 27 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 20 July 2017 06:00:00 -0400",
                "Thu, 20 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 20 July 2017 06:00:00 -0400",
                "Thu, 20 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 13 July 2017 06:00:00 -0400",
                "Thu, 13 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 13 July 2017 06:00:00 -0400",
                "Thu, 13 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 06 July 2017 06:00:00 -0400",
                "Thu, 06 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 06 July 2017 06:00:00 -0400",
                "Thu, 06 Jul 2017 06:00:00 -0400",
            ),
            (
                "Thu, 28 June 2017 06:00:00 -0400",
                "Thu, 28 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 28 June 2017 06:00:00 -0400",
                "Thu, 28 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 17 Jul 2013 06:00:03 -0400",
                "Thu, 17 Jul 2013 06:00:03 -0400",
            ),
            (
                "Thu, 17 Jul 2013 06:00:03 -0400",
                "Thu, 17 Jul 2013 06:00:03 -0400",
            ),
            (
                "Thu, 02 Apr 2014 06:00:03 -0400",
                "Thu, 02 Apr 2014 06:00:03 -0400",
            ),
            (
                "Thu, 02 Apr 2014 06:00:03 -0400",
                "Thu, 02 Apr 2014 06:00:03 -0400",
            ),
            (
                "Wed, 14 Jan 2016 06:00:03 -0400",
                "Wed, 14 Jan 2016 06:00:03 -0400",
            ),
            (
                "Wed, 14 Jan 2016 06:00:03 -0400",
                "Wed, 14 Jan 2016 06:00:03 -0400",
            ),
            (
                "Thu, 22 June 2017 06:00:00 -0400",
                "Thu, 22 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 22 June 2017 06:00:00 -0400",
                "Thu, 22 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 June 2017 06:00:00 -0400",
                "Thu, 15 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 June 2017 06:00:00 -0400",
                "Thu, 15 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 7 June 2017 06:00:00 -0400",
                "Thu, 7 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 7 June 2017 06:00:00 -0400",
                "Thu, 7 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 1 June 2017 06:00:00 -0400",
                "Thu, 1 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 1 June 2017 06:00:00 -0400",
                "Thu, 1 Jun 2017 06:00:00 -0400",
            ),
            (
                "Thu, 23 Dec 2015 06:00:03 -0400",
                "Thu, 23 Dec 2015 06:00:03 -0400",
            ),
            (
                "Thu, 23 Dec 2015 06:00:03 -0400",
                "Thu, 23 Dec 2015 06:00:03 -0400",
            ),
            (
                "Thu, 14 Feb 2014 06:00:03 -0400",
                "Thu, 14 Feb 2014 06:00:03 -0400",
            ),
            (
                "Thu, 14 Feb 2014 06:00:03 -0400",
                "Thu, 14 Feb 2014 06:00:03 -0400",
            ),
            (
                "Thu, 04 Dec 2013 06:00:03 -0400",
                "Thu, 04 Dec 2013 06:00:03 -0400",
            ),
            (
                "Thu, 04 Dec 2013 06:00:03 -0400",
                "Thu, 04 Dec 2013 06:00:03 -0400",
            ),
            (
                "Thu, 20 Dec 2016 06:00:00 -0400",
                "Thu, 20 Dec 2016 06:00:00 -0400",
            ),
            (
                "Thu, 20 Dec 2016 06:00:00 -0400",
                "Thu, 20 Dec 2016 06:00:00 -0400",
            ),
            (
                "Thu, 23 Nov 2016 06:00:00 -0400",
                "Thu, 23 Nov 2016 06:00:00 -0400",
            ),
            (
                "Thu, 23 Nov 2016 06:00:00 -0400",
                "Thu, 23 Nov 2016 06:00:00 -0400",
            ),
            (
                "Thu, 05 Aug 2016 06:00:00 -0400",
                "Thu, 05 Aug 2016 06:00:00 -0400",
            ),
            (
                "Thu, 05 Aug 2016 06:00:00 -0400",
                "Thu, 05 Aug 2016 06:00:00 -0400",
            ),
            (
                "Fri, 09 Jun 2016 12:00:00 -0400",
                "Fri, 09 Jun 2016 12:00:00 -0400",
            ),
            (
                "Fri, 09 Jun 2016 12:00:00 -0400",
                "Fri, 09 Jun 2016 12:00:00 -0400",
            ),
            (
                "Thu, 10 May 2017 06:00:00 -0400",
                "Thu, 10 May 2017 06:00:00 -0400",
            ),
            (
                "Thu, 10 May 2017 06:00:00 -0400",
                "Thu, 10 May 2017 06:00:00 -0400",
            ),
            (
                "Thu, 22 Feb 2017 06:00:00 -0400",
                "Thu, 22 Feb 2017 06:00:00 -0400",
            ),
            (
                "Thu, 22 Feb 2017 06:00:00 -0400",
                "Thu, 22 Feb 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 Feb 2017 06:00:00 -0400",
                "Thu, 15 Feb 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 Feb 2017 06:00:00 -0400",
                "Thu, 15 Feb 2017 06:00:00 -0400",
            ),
        ];

        foo.iter()
            .map(|&(bad, good)| assert_eq!(replace_month(bad), good))
            .fold((), |(), _| ());
    }

    #[test]
    fn test_replace_leading_zeroes() {
        let foo = vec![
            (
                "Thu, 6 July 2017 15:30:00 PDT",
                "Thu, 6 July 2017 15:30:00 PDT",
            ),
            (
                "Mon, 10 July 2017 16:00:00 PDT",
                "Mon, 10 July 2017 16:00:00 PDT",
            ),
            (
                "Mon, 17 July 2017 17:00:00 PDT",
                "Mon, 17 July 2017 17:00:00 PDT",
            ),
            (
                "Mon, 24 July 2017 16:00:00 PDT",
                "Mon, 24 July 2017 16:00:00 PDT",
            ),
            (
                "Mon, 31 July 2017 16:00:00 PDT",
                "Mon, 31 July 2017 16:00:00 PDT",
            ),
            (
                "Thu, 30 Aug 2017 1:30:00 PDT",
                "Thu, 30 Aug 2017 1:30:00 PDT",
            ),
            (
                "Wed, 20 Sep 2017 10:00:00 -0000",
                "Wed, 20 Sep 2017 10:00:00 +0000",
            ),
            (
                "Wed, 13 Sep 2017 10:00:00 -0000",
                "Wed, 13 Sep 2017 10:00:00 +0000",
            ),
            (
                "Wed, 09 Aug 2017 10:00:00 -0000",
                "Wed, 09 Aug 2017 10:00:00 +0000",
            ),
            (
                "Wed, 02 Aug 2017 10:00:00 -0000",
                "Wed, 02 Aug 2017 10:00:00 +0000",
            ),
            (
                "Wed, 26 Jul 2017 10:00:00 -0000",
                "Wed, 26 Jul 2017 10:00:00 +0000",
            ),
            (
                "Wed, 19 Jul 2017 10:00:00 -0000",
                "Wed, 19 Jul 2017 10:00:00 +0000",
            ),
            (
                "Wed, 12 Jul 2017 10:00:00 -0000",
                "Wed, 12 Jul 2017 10:00:00 +0000",
            ),
            (
                "Wed, 28 Jun 2017 10:00:00 -0000",
                "Wed, 28 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 21 Jun 2017 10:00:00 -0000",
                "Wed, 21 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 14 Jun 2017 10:00:00 -0000",
                "Wed, 14 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 07 Jun 2017 10:00:00 -0000",
                "Wed, 07 Jun 2017 10:00:00 +0000",
            ),
            (
                "Wed, 31 May 2017 10:00:00 -0000",
                "Wed, 31 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 24 May 2017 10:00:00 -0000",
                "Wed, 24 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 17 May 2017 10:00:00 -0000",
                "Wed, 17 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 10 May 2017 10:00:00 -0000",
                "Wed, 10 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 03 May 2017 10:00:00 -0000",
                "Wed, 03 May 2017 10:00:00 +0000",
            ),
            (
                "Wed, 19 Apr 2017 10:00:00 -0000",
                "Wed, 19 Apr 2017 10:00:00 +0000",
            ),
            (
                "Wed, 12 Apr 2017 10:00:00 -0000",
                "Wed, 12 Apr 2017 10:00:00 +0000",
            ),
            (
                "Wed, 05 Apr 2017 10:00:00 -0000",
                "Wed, 05 Apr 2017 10:00:00 +0000",
            ),
            (
                "Wed, 29 Mar 2017 10:00:00 -0000",
                "Wed, 29 Mar 2017 10:00:00 +0000",
            ),
            (
                "Wed, 22 Mar 2017 10:00:00 -0000",
                "Wed, 22 Mar 2017 10:00:00 +0000",
            ),
            (
                "Wed, 15 Mar 2017 10:00:00 -0000",
                "Wed, 15 Mar 2017 10:00:00 +0000",
            ),
            (
                "Wed, 08 Mar 2017 11:00:00 -0000",
                "Wed, 08 Mar 2017 11:00:00 +0000",
            ),
            (
                "Wed, 01 Mar 2017 11:00:00 -0000",
                "Wed, 01 Mar 2017 11:00:00 +0000",
            ),
            (
                "Wed, 22 Feb 2017 11:00:00 -0000",
                "Wed, 22 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 15 Feb 2017 11:00:00 -0000",
                "Wed, 15 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 08 Feb 2017 11:00:00 -0000",
                "Wed, 08 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 01 Feb 2017 11:00:00 -0000",
                "Wed, 01 Feb 2017 11:00:00 +0000",
            ),
            (
                "Wed, 25 Jan 2017 11:00:00 -0000",
                "Wed, 25 Jan 2017 11:00:00 +0000",
            ),
            (
                "Fri, 13 Jan 2017 18:38:00 -0000",
                "Fri, 13 Jan 2017 18:38:00 +0000",
            ),
            (
                "Wed, 20 Sep 2017 03:30:00 -0000",
                "Wed, 20 Sep 2017 03:30:00 +0000",
            ),
            (
                "Wed, 13 Sep 2017 03:15:00 -0000",
                "Wed, 13 Sep 2017 03:15:00 +0000",
            ),
            (
                "Wed, 06 Sep 2017 03:15:00 -0000",
                "Wed, 06 Sep 2017 03:15:00 +0000",
            ),
            (
                "Wed, 30 Aug 2017 03:15:00 -0000",
                "Wed, 30 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 23 Aug 2017 03:15:00 -0000",
                "Wed, 23 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 16 Aug 2017 03:15:00 -0000",
                "Wed, 16 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 09 Aug 2017 03:15:00 -0000",
                "Wed, 09 Aug 2017 03:15:00 +0000",
            ),
            (
                "Wed, 02 Aug 2017 03:00:00 -0000",
                "Wed, 02 Aug 2017 03:00:00 +0000",
            ),
            (
                "Tue, 11 Jul 2017 17:14:45 -0000",
                "Tue, 11 Jul 2017 17:14:45 +0000",
            ),
            (
                "Thu, 03 August 2017 06:00:00 -0400",
                "Thu, 03 August 2017 06:00:00 -0400",
            ),
            (
                "Thu, 27 July 2017 06:00:00 -0400",
                "Thu, 27 July 2017 06:00:00 -0400",
            ),
            (
                "Thu, 20 July 2017 06:00:00 -0400",
                "Thu, 20 July 2017 06:00:00 -0400",
            ),
            (
                "Thu, 13 July 2017 06:00:00 -0400",
                "Thu, 13 July 2017 06:00:00 -0400",
            ),
            (
                "Thu, 06 July 2017 06:00:00 -0400",
                "Thu, 06 July 2017 06:00:00 -0400",
            ),
            (
                "Thu, 28 June 2017 06:00:00 -0400",
                "Thu, 28 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 17 Jul 2013 06:00:03 -0400",
                "Thu, 17 Jul 2013 06:00:03 -0400",
            ),
            (
                "Thu, 02 Apr 2014 06:00:03 -0400",
                "Thu, 02 Apr 2014 06:00:03 -0400",
            ),
            (
                "Wed, 14 Jan 2016 06:00:03 -0400",
                "Wed, 14 Jan 2016 06:00:03 -0400",
            ),
            (
                "Thu, 22 June 2017 06:00:00 -0400",
                "Thu, 22 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 June 2017 06:00:00 -0400",
                "Thu, 15 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 7 June 2017 06:00:00 -0400",
                "Thu, 7 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 1 June 2017 06:00:00 -0400",
                "Thu, 1 June 2017 06:00:00 -0400",
            ),
            (
                "Thu, 23 Dec 2015 06:00:03 -0400",
                "Thu, 23 Dec 2015 06:00:03 -0400",
            ),
            (
                "Thu, 14 Feb 2014 06:00:03 -0400",
                "Thu, 14 Feb 2014 06:00:03 -0400",
            ),
            (
                "Thu, 04 Dec 2013 06:00:03 -0400",
                "Thu, 04 Dec 2013 06:00:03 -0400",
            ),
            (
                "Thu, 20 Dec 2016 06:00:00 -0400",
                "Thu, 20 Dec 2016 06:00:00 -0400",
            ),
            (
                "Thu, 23 Nov 2016 06:00:00 -0400",
                "Thu, 23 Nov 2016 06:00:00 -0400",
            ),
            (
                "Thu, 05 Aug 2016 06:00:00 -0400",
                "Thu, 05 Aug 2016 06:00:00 -0400",
            ),
            (
                "Fri, 09 Jun 2016 12:00:00 -0400",
                "Fri, 09 Jun 2016 12:00:00 -0400",
            ),
            (
                "Thu, 10 May 2017 06:00:00 -0400",
                "Thu, 10 May 2017 06:00:00 -0400",
            ),
            (
                "Thu, 22 Feb 2017 06:00:00 -0400",
                "Thu, 22 Feb 2017 06:00:00 -0400",
            ),
            (
                "Thu, 15 Feb 2017 06:00:00 -0400",
                "Thu, 15 Feb 2017 06:00:00 -0400",
            ),
        ];

        foo.iter()
            .map(|&(bad, good)| assert_eq!(replace_leading_zeros(bad), good))
            .fold((), |(), _| ());
    }
}
