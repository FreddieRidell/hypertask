use super::abstract_date::AbstractDate;
use super::parsing_error::{PrimitiveParsingError, PrimitiveParsingResult};
use super::period::Period;
use chrono::prelude::*;
use regex::Regex;

fn parse_as_partial_iso(
    get_now: &Fn() -> DateTime<Utc>,
    string: &str,
) -> PrimitiveParsingResult<DateTime<Utc>> {
    let captures = vec![
        r"^(?P<month>\d{2})-(?P<day>\d{2})$",
        r"^(?P<year>\d{4})$",
        r"^(?P<year>\d{4})-(?P<month>\d{2})$",
        r"^(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})$",
    ]
    .iter()
    .find_map(|re| Regex::new(re).unwrap().captures(string))
    .ok_or(PrimitiveParsingError::NotAPartialIsoDate(String::from(
        string,
    )))?;

    let mut date = get_now();

    if let Some(year) = captures.name("year") {
        date = date
            .with_year(year.as_str().parse::<i32>().unwrap())
            .unwrap();
    }

    if let Some(month) = captures.name("month") {
        date = date
            .with_month(month.as_str().parse::<u32>().unwrap())
            .unwrap();
    }

    if let Some(day) = captures.name("day") {
        date = date.with_day(day.as_str().parse::<u32>().unwrap()).unwrap();
    }

    Ok(date
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap())
}

#[derive(Debug, Eq, PartialEq)]
pub enum Prop {
    Archived(Option<bool>),
    Canonical(Option<String>),
    CreatedAt(AbstractDate),
    Description(Option<String>),
    Done(Option<AbstractDate>),
    Due(Option<AbstractDate>),
    Icon(Option<String>),
    Image(Option<String>),
    Keywords(Option<Vec<String>>),
    Recur(Option<Period>),
    SiteName(Option<String>),
    Snooze(Option<AbstractDate>),
    Subject(Option<String>),
    MetaTags(Option<Vec<String>>),
    Title(Option<String>),
    Url(Option<String>),
    Wait(Option<AbstractDate>),
}

impl Prop {
    fn parse_boolean(
        wrapper: &Fn(Option<bool>) -> Prop,
        name: &str,
        value: &str,
    ) -> Result<Prop, PrimitiveParsingError> {
        match value {
            "true" => Ok(wrapper(Some(true))),
            "t" => Ok(wrapper(Some(true))),
            "yes" => Ok(wrapper(Some(true))),
            "y" => Ok(wrapper(Some(true))),
            "false" => Ok(wrapper(Some(false))),
            "f" => Ok(wrapper(Some(false))),
            "no" => Ok(wrapper(Some(false))),
            "n" => Ok(wrapper(Some(false))),
            "" => Ok(wrapper(None)),
            _ => Err(PrimitiveParsingError::MalformedBoolean(format!(
                "{}:{}",
                name, value
            ))),
        }
    }

    fn parse_plain(
        wrapper: &Fn(Option<String>) -> Prop,
        _name: &str,
        string: &str,
    ) -> Result<Prop, PrimitiveParsingError> {
        let wrapped = if string.len() == 0 {
            None
        } else {
            Some(String::from(string))
        };

        Ok(wrapper(wrapped))
    }

    fn parse_date_time_like(
        wrapper: &Fn(Option<AbstractDate>) -> Prop,
        name: &str,
        string: &str,
        get_now: &Fn() -> DateTime<Utc>,
    ) -> Result<Prop, PrimitiveParsingError> {
        if string.len() == 0 {
            return Ok(wrapper(None));
        }

        //attempts to parse the string through various different parsers in turn untill it
        //eventually gives up and returns an Err

        let parsed = match string {
            //first check for simple string matches
            "now" => Ok(AbstractDate::Definite(get_now())),
            "soon" => Ok(AbstractDate::Deferred(Box::new(
                |ds: &[DateTime<Utc>]| ds[0],
            ))),
            _ => Err(PrimitiveParsingError::MalformedDateLike(format!(
                "{}:{}",
                name, string
            ))),
        }
        .or_else(|_| {
            let date = parse_as_partial_iso(get_now, &string)?;

            Ok(AbstractDate::Definite(date))
        })
        .or_else(|_: PrimitiveParsingError| {
            //try to parse as a period
            let period = Period::from_string(string)?;

            Ok(AbstractDate::Definite(get_now() + period.to_duration()))
        })
        .or_else(|_: PrimitiveParsingError| {
            //if all this has failed, we should return an error saying so
            Err(PrimitiveParsingError::MalformedDateLike(format!(
                "{}:{}",
                name, string
            )))
        });

        match parsed {
            Ok(x) => Ok(wrapper(Some(x))),
            Err(e) => Err(e),
        }
    }

    fn parse_period(
        wrapper: &Fn(Option<Period>) -> Prop,
        name: &str,
        string: &str,
    ) -> Result<Prop, PrimitiveParsingError> {
        if string.len() == 0 {
            return Ok(wrapper(None));
        }

        match Period::from_string(string) {
            Err(_) => Err(PrimitiveParsingError::MalformedRecur(format!(
                "{}:{}",
                name, string
            ))),
            Ok(o) => Ok(wrapper(Some(o))),
        }
    }

    /// tries to parse a string to a prop
    /// returns None if the string is not a prop
    /// returns Some(Err) if the string is a malformed prop
    /// returns Some(Ok) if the string parsed correctly
    pub fn from_string(
        get_now: &Fn() -> DateTime<Utc>,
        string: &str,
    ) -> Option<Result<Self, PrimitiveParsingError>> {
        let mut colon_index: Option<usize> = None;

        for (i, c) in string.chars().enumerate() {
            if c == ':' {
                colon_index = Some(i)
            }
        }

        match colon_index {
            None => None,
            Some(i) => {
                let prop_name = &string[..i];
                let prop_value_raw = &string[i + 1..];

                let parsed = match prop_name {
                    "description" => {
                        Prop::parse_plain(&Prop::Description, &prop_name, &prop_value_raw)
                    }
                    "archived" => Prop::parse_boolean(&Prop::Archived, &prop_name, &prop_value_raw),
                    "due" => Prop::parse_date_time_like(
                        &Prop::Due,
                        &prop_name,
                        &prop_value_raw,
                        &get_now,
                    ),
                    "done" => Prop::parse_date_time_like(
                        &Prop::Done,
                        &prop_name,
                        &prop_value_raw,
                        &get_now,
                    ),
                    "snooze" => Prop::parse_date_time_like(
                        &Prop::Snooze,
                        &prop_name,
                        &prop_value_raw,
                        &get_now,
                    ),
                    "wait" => Prop::parse_date_time_like(
                        &Prop::Wait,
                        &prop_name,
                        &prop_value_raw,
                        &get_now,
                    ),
                    "recur" => Prop::parse_period(&Prop::Recur, &prop_name, &prop_value_raw),

                    _ => Err(PrimitiveParsingError::UnknownProp(String::from(string))),
                };

                Some(parsed)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn mock_get_now() -> DateTime<Utc> {
        Utc.ymd(2014, 7, 8).and_hms(9, 10, 11)
    }

    mod from_string {
        use super::*;

        #[test]
        fn returns_none_when_given_a_string_that_is_not_a_prop() {
            assert_eq!(Prop::from_string(&mock_get_now, "+tag"), None);
            assert_eq!(Prop::from_string(&mock_get_now, "plain"), None);
        }

        #[test]
        fn returns_err_when_given_invalid_prop() {
            assert_eq!(
                Prop::from_string(&mock_get_now, "foo:bar"),
                Some(Err(PrimitiveParsingError::UnknownProp(String::from(
                    "foo:bar"
                ))))
            );
        }

        #[test]
        fn parses_various_date_formats() {
            assert_eq!(
                Prop::from_string(&mock_get_now, "due:2018"),
                Some(Ok(Prop::Due(Some(AbstractDate::Definite(
                    Utc.ymd(2018, 7, 8).and_hms(0, 0, 0)
                )))))
            );
            assert_eq!(
                Prop::from_string(&mock_get_now, "due:2018-02"),
                Some(Ok(Prop::Due(Some(AbstractDate::Definite(
                    Utc.ymd(2018, 02, 8).and_hms(0, 0, 0)
                )))))
            );
            assert_eq!(
                Prop::from_string(&mock_get_now, "due:2018-02-03"),
                Some(Ok(Prop::Due(Some(AbstractDate::Definite(
                    Utc.ymd(2018, 2, 3).and_hms(0, 0, 0)
                )))))
            );
            assert_eq!(
                Prop::from_string(&mock_get_now, "due:02-03"),
                Some(Ok(Prop::Due(Some(AbstractDate::Definite(
                    Utc.ymd(2014, 2, 3).and_hms(0, 0, 0)
                )))))
            );
        }

        mod various_prop_names {
            use super::*;

            #[test]
            fn can_parse_archived() {
                assert_eq!(
                    Prop::from_string(&mock_get_now, "archived:true"),
                    Some(Ok(Prop::Archived(Some(true))))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "archived:"),
                    Some(Ok(Prop::Archived(None)))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "archived:foo"),
                    Some(Err(PrimitiveParsingError::MalformedBoolean(String::from(
                        "archived:foo"
                    ))))
                );
            }

            #[test]
            fn can_parse_description() {
                assert_eq!(
                    Prop::from_string(&mock_get_now, "description:foo"),
                    Some(Ok(Prop::Description(Some(String::from("foo")))))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "description:"),
                    Some(Ok(Prop::Description(None)))
                );
            }

            #[test]
            fn can_parse_done() {
                assert_eq!(
                    Prop::from_string(&mock_get_now, "done:now"),
                    Some(Ok(Prop::Done(Some(AbstractDate::Definite(mock_get_now())))))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "done:"),
                    Some(Ok(Prop::Done(None)))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "done:foo"),
                    Some(Err(PrimitiveParsingError::MalformedDateLike(String::from(
                        "done:foo"
                    ))))
                );
            }

            #[test]
            fn can_parse_due() {
                assert_eq!(
                    Prop::from_string(&mock_get_now, "due:now"),
                    Some(Ok(Prop::Due(Some(AbstractDate::Definite(mock_get_now())))))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "due:"),
                    Some(Ok(Prop::Due(None)))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "due:foo"),
                    Some(Err(PrimitiveParsingError::MalformedDateLike(String::from(
                        "due:foo"
                    ))))
                );
            }

            #[test]
            fn can_parse_recur() {
                assert_eq!(
                    Prop::from_string(&mock_get_now, "recur:1d"),
                    Some(Ok(Prop::Recur(Some(Period::Day(1)))))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "recur:"),
                    Some(Ok(Prop::Recur(None)))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "recur:foo"),
                    Some(Err(PrimitiveParsingError::MalformedRecur(String::from(
                        "recur:foo"
                    ))))
                );
            }

            #[test]
            fn can_parse_snooze() {
                assert_eq!(
                    Prop::from_string(&mock_get_now, "snooze:now"),
                    Some(Ok(Prop::Snooze(Some(AbstractDate::Definite(
                        mock_get_now()
                    )))))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "snooze:"),
                    Some(Ok(Prop::Snooze(None)))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "snooze:foo"),
                    Some(Err(PrimitiveParsingError::MalformedDateLike(String::from(
                        "snooze:foo"
                    ))))
                );
            }

            #[test]
            fn can_parse_wait() {
                assert_eq!(
                    Prop::from_string(&mock_get_now, "wait:now"),
                    Some(Ok(Prop::Wait(Some(AbstractDate::Definite(mock_get_now())))))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "wait:"),
                    Some(Ok(Prop::Wait(None)))
                );
                assert_eq!(
                    Prop::from_string(&mock_get_now, "wait:foo"),
                    Some(Err(PrimitiveParsingError::MalformedDateLike(String::from(
                        "wait:foo"
                    ))))
                );
            }
        }
    }
}
