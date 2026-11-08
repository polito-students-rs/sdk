use std::error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ResponseContent<T> {
    pub status: reqwest::StatusCode,
    pub content: String,
    pub entity: Option<T>,
}

#[derive(Debug)]
pub enum Error<T> {
    Reqwest(reqwest::Error),
    ReqwestMiddleware(reqwest_middleware::Error),
    Serde(serde_json::Error),
    Io(std::io::Error),
    ResponseError(ResponseContent<T>),
}

impl <T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (module, e) = match self {
            Error::Reqwest(e) => ("reqwest", e.to_string()),
            Error::ReqwestMiddleware(e) => ("reqwest-middleware", e.to_string()),
            Error::Serde(e) => ("serde", e.to_string()),
            Error::Io(e) => ("IO", e.to_string()),
            Error::ResponseError(e) => ("response", format!("status code {}", e.status)),
        };
        write!(f, "error in {}: {}", module, e)
    }
}

impl <T: fmt::Debug> error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(match self {
            Error::Reqwest(e) => e,
            Error::ReqwestMiddleware(e) => e,
            Error::Serde(e) => e,
            Error::Io(e) => e,
            Error::ResponseError(_) => return None,
        })
    }
}

impl <T> From<reqwest::Error> for Error<T> {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl<T> From<reqwest_middleware::Error> for Error<T> {
    fn from(e: reqwest_middleware::Error) -> Self {
        Error::ReqwestMiddleware(e)
    }
}

impl <T> From<serde_json::Error> for Error<T> {
    fn from(e: serde_json::Error) -> Self {
        Error::Serde(e)
    }
}

impl <T> From<std::io::Error> for Error<T> {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub fn urlencode<T: AsRef<str>>(s: T) -> String {
    ::url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

pub fn parse_deep_object(prefix: &str, value: &serde_json::Value) -> Vec<(String, String)> {
    if let serde_json::Value::Object(object) = value {
        let mut params = vec![];

        for (key, value) in object {
            match value {
                serde_json::Value::Object(_) => params.append(&mut parse_deep_object(
                    &format!("{}[{}]", prefix, key),
                    value,
                )),
                serde_json::Value::Array(array) => {
                    for (i, value) in array.iter().enumerate() {
                        params.append(&mut parse_deep_object(
                            &format!("{}[{}][{}]", prefix, key, i),
                            value,
                        ));
                    }
                },
                serde_json::Value::String(s) => params.push((format!("{}[{}]", prefix, key), s.clone())),
                _ => params.push((format!("{}[{}]", prefix, key), value.to_string())),
            }
        }

        return params;
    }

    unimplemented!("Only objects are supported with style=deepObject")
}

/// Internal use only
/// A content type supported by this client.
#[allow(dead_code)]
enum ContentType {
    Json,
    Text,
    Unsupported(String)
}

impl From<&str> for ContentType {
    fn from(content_type: &str) -> Self {
        if content_type.starts_with("application") && content_type.contains("json") {
            return Self::Json;
        } else if content_type.starts_with("text/plain") {
            return Self::Text;
        } else {
            return Self::Unsupported(content_type.to_string());
        }
    }
}

pub mod announcements_api;
pub mod auth_api;
pub mod bookings_api;
pub mod courses_api;
pub mod esc_api;
pub mod event_admissions_api;
pub mod exams_api;
pub mod job_offers_api;
pub mod lectures_api;
pub mod news_api;
pub mod offering_api;
pub mod people_api;
pub mod places_api;
pub mod student_api;
pub mod surveys_api;
pub mod tickets_api;

pub mod configuration;

use std::sync::Arc;

pub trait Api {
    fn announcements_api(&self) -> &dyn announcements_api::AnnouncementsApi;
    fn auth_api(&self) -> &dyn auth_api::AuthApi;
    fn bookings_api(&self) -> &dyn bookings_api::BookingsApi;
    fn courses_api(&self) -> &dyn courses_api::CoursesApi;
    fn esc_api(&self) -> &dyn esc_api::EscApi;
    fn event_admissions_api(&self) -> &dyn event_admissions_api::EventAdmissionsApi;
    fn exams_api(&self) -> &dyn exams_api::ExamsApi;
    fn job_offers_api(&self) -> &dyn job_offers_api::JobOffersApi;
    fn lectures_api(&self) -> &dyn lectures_api::LecturesApi;
    fn news_api(&self) -> &dyn news_api::NewsApi;
    fn offering_api(&self) -> &dyn offering_api::OfferingApi;
    fn people_api(&self) -> &dyn people_api::PeopleApi;
    fn places_api(&self) -> &dyn places_api::PlacesApi;
    fn student_api(&self) -> &dyn student_api::StudentApi;
    fn surveys_api(&self) -> &dyn surveys_api::SurveysApi;
    fn tickets_api(&self) -> &dyn tickets_api::TicketsApi;
}

pub struct ApiClient {
    announcements_api: Box<dyn announcements_api::AnnouncementsApi>,
    auth_api: Box<dyn auth_api::AuthApi>,
    bookings_api: Box<dyn bookings_api::BookingsApi>,
    courses_api: Box<dyn courses_api::CoursesApi>,
    esc_api: Box<dyn esc_api::EscApi>,
    event_admissions_api: Box<dyn event_admissions_api::EventAdmissionsApi>,
    exams_api: Box<dyn exams_api::ExamsApi>,
    job_offers_api: Box<dyn job_offers_api::JobOffersApi>,
    lectures_api: Box<dyn lectures_api::LecturesApi>,
    news_api: Box<dyn news_api::NewsApi>,
    offering_api: Box<dyn offering_api::OfferingApi>,
    people_api: Box<dyn people_api::PeopleApi>,
    places_api: Box<dyn places_api::PlacesApi>,
    student_api: Box<dyn student_api::StudentApi>,
    surveys_api: Box<dyn surveys_api::SurveysApi>,
    tickets_api: Box<dyn tickets_api::TicketsApi>,
}

impl ApiClient {
    pub fn new(configuration: Arc<configuration::Configuration>) -> Self {
        Self {
            announcements_api: Box::new(announcements_api::AnnouncementsApiClient::new(configuration.clone())),
            auth_api: Box::new(auth_api::AuthApiClient::new(configuration.clone())),
            bookings_api: Box::new(bookings_api::BookingsApiClient::new(configuration.clone())),
            courses_api: Box::new(courses_api::CoursesApiClient::new(configuration.clone())),
            esc_api: Box::new(esc_api::EscApiClient::new(configuration.clone())),
            event_admissions_api: Box::new(event_admissions_api::EventAdmissionsApiClient::new(configuration.clone())),
            exams_api: Box::new(exams_api::ExamsApiClient::new(configuration.clone())),
            job_offers_api: Box::new(job_offers_api::JobOffersApiClient::new(configuration.clone())),
            lectures_api: Box::new(lectures_api::LecturesApiClient::new(configuration.clone())),
            news_api: Box::new(news_api::NewsApiClient::new(configuration.clone())),
            offering_api: Box::new(offering_api::OfferingApiClient::new(configuration.clone())),
            people_api: Box::new(people_api::PeopleApiClient::new(configuration.clone())),
            places_api: Box::new(places_api::PlacesApiClient::new(configuration.clone())),
            student_api: Box::new(student_api::StudentApiClient::new(configuration.clone())),
            surveys_api: Box::new(surveys_api::SurveysApiClient::new(configuration.clone())),
            tickets_api: Box::new(tickets_api::TicketsApiClient::new(configuration.clone())),
        }
    }
}

impl Api for ApiClient {
    fn announcements_api(&self) -> &dyn announcements_api::AnnouncementsApi {
        self.announcements_api.as_ref()
    }
    fn auth_api(&self) -> &dyn auth_api::AuthApi {
        self.auth_api.as_ref()
    }
    fn bookings_api(&self) -> &dyn bookings_api::BookingsApi {
        self.bookings_api.as_ref()
    }
    fn courses_api(&self) -> &dyn courses_api::CoursesApi {
        self.courses_api.as_ref()
    }
    fn esc_api(&self) -> &dyn esc_api::EscApi {
        self.esc_api.as_ref()
    }
    fn event_admissions_api(&self) -> &dyn event_admissions_api::EventAdmissionsApi {
        self.event_admissions_api.as_ref()
    }
    fn exams_api(&self) -> &dyn exams_api::ExamsApi {
        self.exams_api.as_ref()
    }
    fn job_offers_api(&self) -> &dyn job_offers_api::JobOffersApi {
        self.job_offers_api.as_ref()
    }
    fn lectures_api(&self) -> &dyn lectures_api::LecturesApi {
        self.lectures_api.as_ref()
    }
    fn news_api(&self) -> &dyn news_api::NewsApi {
        self.news_api.as_ref()
    }
    fn offering_api(&self) -> &dyn offering_api::OfferingApi {
        self.offering_api.as_ref()
    }
    fn people_api(&self) -> &dyn people_api::PeopleApi {
        self.people_api.as_ref()
    }
    fn places_api(&self) -> &dyn places_api::PlacesApi {
        self.places_api.as_ref()
    }
    fn student_api(&self) -> &dyn student_api::StudentApi {
        self.student_api.as_ref()
    }
    fn surveys_api(&self) -> &dyn surveys_api::SurveysApi {
        self.surveys_api.as_ref()
    }
    fn tickets_api(&self) -> &dyn tickets_api::TicketsApi {
        self.tickets_api.as_ref()
    }
}

#[cfg(feature = "mockall")]
pub struct MockApiClient {
    pub announcements_api_mock: announcements_api::MockAnnouncementsApi,
    pub auth_api_mock: auth_api::MockAuthApi,
    pub bookings_api_mock: bookings_api::MockBookingsApi,
    pub courses_api_mock: courses_api::MockCoursesApi,
    pub esc_api_mock: esc_api::MockEscApi,
    pub event_admissions_api_mock: event_admissions_api::MockEventAdmissionsApi,
    pub exams_api_mock: exams_api::MockExamsApi,
    pub job_offers_api_mock: job_offers_api::MockJobOffersApi,
    pub lectures_api_mock: lectures_api::MockLecturesApi,
    pub news_api_mock: news_api::MockNewsApi,
    pub offering_api_mock: offering_api::MockOfferingApi,
    pub people_api_mock: people_api::MockPeopleApi,
    pub places_api_mock: places_api::MockPlacesApi,
    pub student_api_mock: student_api::MockStudentApi,
    pub surveys_api_mock: surveys_api::MockSurveysApi,
    pub tickets_api_mock: tickets_api::MockTicketsApi,
}

#[cfg(feature = "mockall")]
impl MockApiClient {
    pub fn new() -> Self {
        Self {
            announcements_api_mock: announcements_api::MockAnnouncementsApi::new(),
            auth_api_mock: auth_api::MockAuthApi::new(),
            bookings_api_mock: bookings_api::MockBookingsApi::new(),
            courses_api_mock: courses_api::MockCoursesApi::new(),
            esc_api_mock: esc_api::MockEscApi::new(),
            event_admissions_api_mock: event_admissions_api::MockEventAdmissionsApi::new(),
            exams_api_mock: exams_api::MockExamsApi::new(),
            job_offers_api_mock: job_offers_api::MockJobOffersApi::new(),
            lectures_api_mock: lectures_api::MockLecturesApi::new(),
            news_api_mock: news_api::MockNewsApi::new(),
            offering_api_mock: offering_api::MockOfferingApi::new(),
            people_api_mock: people_api::MockPeopleApi::new(),
            places_api_mock: places_api::MockPlacesApi::new(),
            student_api_mock: student_api::MockStudentApi::new(),
            surveys_api_mock: surveys_api::MockSurveysApi::new(),
            tickets_api_mock: tickets_api::MockTicketsApi::new(),
        }
    }
}

#[cfg(feature = "mockall")]
impl Api for MockApiClient {
    fn announcements_api(&self) -> &dyn announcements_api::AnnouncementsApi {
        &self.announcements_api_mock
    }
    fn auth_api(&self) -> &dyn auth_api::AuthApi {
        &self.auth_api_mock
    }
    fn bookings_api(&self) -> &dyn bookings_api::BookingsApi {
        &self.bookings_api_mock
    }
    fn courses_api(&self) -> &dyn courses_api::CoursesApi {
        &self.courses_api_mock
    }
    fn esc_api(&self) -> &dyn esc_api::EscApi {
        &self.esc_api_mock
    }
    fn event_admissions_api(&self) -> &dyn event_admissions_api::EventAdmissionsApi {
        &self.event_admissions_api_mock
    }
    fn exams_api(&self) -> &dyn exams_api::ExamsApi {
        &self.exams_api_mock
    }
    fn job_offers_api(&self) -> &dyn job_offers_api::JobOffersApi {
        &self.job_offers_api_mock
    }
    fn lectures_api(&self) -> &dyn lectures_api::LecturesApi {
        &self.lectures_api_mock
    }
    fn news_api(&self) -> &dyn news_api::NewsApi {
        &self.news_api_mock
    }
    fn offering_api(&self) -> &dyn offering_api::OfferingApi {
        &self.offering_api_mock
    }
    fn people_api(&self) -> &dyn people_api::PeopleApi {
        &self.people_api_mock
    }
    fn places_api(&self) -> &dyn places_api::PlacesApi {
        &self.places_api_mock
    }
    fn student_api(&self) -> &dyn student_api::StudentApi {
        &self.student_api_mock
    }
    fn surveys_api(&self) -> &dyn surveys_api::SurveysApi {
        &self.surveys_api_mock
    }
    fn tickets_api(&self) -> &dyn tickets_api::TicketsApi {
        &self.tickets_api_mock
    }
}

