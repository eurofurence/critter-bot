use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use color_eyre::eyre::{self, Context};
use reqwest::{
    Client, ClientBuilder, StatusCode, Url,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use std::{borrow::Cow, iter::repeat, sync::Arc};
use uuid::Uuid;

use crate::events::Shift;

#[derive(Clone)]
pub struct Api {
    api_url: Arc<Url>,
    client: Client,
}

impl Api {
    pub fn new(base_url: &str, token: &str) -> eyre::Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))?,
        );
        let client = ClientBuilder::new().default_headers(headers).build()?;

        Ok(Self {
            api_url: Arc::new(Url::parse(base_url).context("Invalid base url provided")?),
            client,
        })
    }

    #[tracing::instrument(name = "api_verify", skip(self))]
    pub async fn verify(
        &self,
        token: Uuid,
        uname: String,
    ) -> eyre::Result<Result<i64, Cow<'static, str>>> {
        #[derive(serde::Deserialize)]
        struct Response {
            user_id: i64,
        }

        // hack but what ever
        let url = Url::parse_with_params(
            self.api_url.join("api/v2/bot/verify")?.as_str(),
            &[("token", token.to_string()), ("uname", uname)],
        )?;
        let resp = self.client.get(url).send().await?;
        match resp.status() {
            StatusCode::OK => Ok(Ok(resp.json::<Response>().await?.user_id)),
            StatusCode::NOT_FOUND => Ok(Err(Cow::Borrowed(
                "Unknown or invalid authentication token provided",
            ))),
            code => {
                let ray = Uuid::new_v4();
                let text = resp.text().await;

                error!(
                    ray = ray.to_string(),
                    status = code.to_string(),
                    body = format!("{text:?}"),
                    "Received invalid response from "
                );

                Ok(Err(Cow::Owned(format!(
                    "An unknown error occured, ray={ray}"
                ))))
            }
        }
    }

    #[tracing::instrument(name = "api_shifts", skip(self))]
    pub async fn shifts(&self, date: NaiveDate, tz: Tz) -> eyre::Result<Vec<Shift>> {
        // hack but what ever
        let url = Url::parse_with_params(
            self.api_url.join("api/v2/shift-manager/shifts")?.as_str(),
            &[("date", date.to_string())],
        )?;

        let shifts = self
            .client
            .get(url)
            .send()
            .await?
            .json::<ApiShifts>()
            .await?;

        Ok(shifts
            .shifts
            .into_iter()
            .map(|shift| Shift {
                id: shift.id,
                title: shift.title,
                r#type: shift.r#type,
                location: shift.location,
                start: shift.start_ts,
                end: shift.end_ts,
                critters: shift
                    .assignments
                    .into_iter()
                    .flat_map(|assignment| {
                        assignment
                            .users
                            .into_iter()
                            .zip(repeat(assignment.angle_type_name))
                            .map(|(user, angle_type_name)| {
                                (user.user_name, angle_type_name, user.user_id, user.is_staff)
                            })
                    })
                    .collect(),
                // FIXME: wait for api changes!
                managers: vec![],
                req: shift.required as usize,
                ppe: shift.eligibility.needs_cert,
                tz,
            })
            .collect())
    }

    #[tracing::instrument(name = "api_dates", skip(self))]
    pub async fn dates(&self) -> eyre::Result<Vec<NaiveDate>> {
        let mut dates = self
            .client
            .get(self.api_url.join("api/v2/shift-manager/dates")?)
            .send()
            .await?
            .json::<ApiDates>()
            .await?
            .dates;
        dates.sort_by_key(|d| d.day);
        Ok(dates.into_iter().map(|d| d.date).collect())
    }
}

#[derive(serde::Deserialize)]
struct ApiShifts {
    shifts: Vec<ApiShift>,
}

#[derive(serde::Deserialize)]
struct ApiShift {
    id: i64,
    title: Arc<str>,
    r#type: Arc<str>,
    location: Arc<str>,
    start_ts: DateTime<Utc>,
    end_ts: DateTime<Utc>,
    required: i32,
    eligibility: ApiEligibility,
    assignments: Vec<ApiAssignment>,
}

#[derive(serde::Deserialize)]
struct ApiAssignment {
    angle_type_name: Arc<str>,
    users: Vec<ApiUser>,
}

#[derive(serde::Deserialize)]
struct ApiUser {
    user_id: i64,
    user_name: Arc<str>,
    is_staff: bool,
}

#[derive(serde::Deserialize)]
struct ApiEligibility {
    needs_cert: bool,
}

#[derive(serde::Deserialize)]
struct ApiDates {
    // #[serde(rename = "ok")]
    // _ok: bool,
    dates: Vec<ApiDate>,
}

#[derive(serde::Deserialize)]
struct ApiDate {
    date: NaiveDate,
    // weekday: ApiWeekday,
    day: u32,
    // display: String,
}

// #[derive(serde::Deserialize)]
// enum ApiWeekday {
//     Mon,
//     Tue,
//     Wed,
//     Thu,
//     Fri,
//     Sat,
//     Sun,
// }
