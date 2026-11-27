//! Data from the OpenAPI document: servers and per-operation security.
//!
//! Generated from `spec/templates/spec.mustache`: edit the template, not this file.

/// A server declared in `servers`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Server {
    pub url: &'static str,
    pub description: &'static str,
}

/// One alternative of a `security` list: the names of the schemes that must
/// all be satisfied. An empty requirement (`{}`) means anonymous access.
pub type SecurityRequirement = &'static [&'static str];

/// An operation declared in `paths`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operation {
    pub operation_id: Option<&'static str>,
    /// Uppercase HTTP method, e.g. `GET`.
    pub method: &'static str,
    /// Path template relative to the server URL, e.g. `/courses/{courseId}`.
    pub path: &'static str,
    /// Operation-level `security`; `None` inherits [`GLOBAL_SECURITY`].
    pub security: Option<&'static [SecurityRequirement]>,
}

impl Operation {
    /// Security that applies to the operation, after inheritance.
    pub fn effective_security(&self) -> &'static [SecurityRequirement] {
        match self.security {
            Some(security) => security,
            None => GLOBAL_SECURITY,
        }
    }

    /// True when the operation can be called without credentials: no
    /// requirement at all, or an empty alternative (`security: [{}]`).
    pub fn allows_anonymous(&self) -> bool {
        let security = self.effective_security();
        security.is_empty() || security.iter().any(|requirement| requirement.is_empty())
    }
}

/// Servers in declaration order.
pub const SERVERS: &[Server] = &[
    Server {
        url: r#"https://app.didattica.polito.it/api"#,
        description: r#"Production server"#,
    },
    Server {
        url: r#"https://app.didattica.polito.it/mock/api"#,
        description: r#"Mock server (uses example data)"#,
    },
];

/// The first server is the default one, as in the OpenAPI specification.
pub const DEFAULT_SERVER: &Server = &SERVERS[0];

/// Top-level `security`.
pub const GLOBAL_SECURITY: &[SecurityRequirement] = &[
    &[r#"BearerAuth"#, ],
];

/// Every operation, in declaration order.
pub const OPERATIONS: &[Operation] = &[
    Operation { operation_id: Some(r#"Announcements_getAnnouncements"#), path: r#"/announcements"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Announcements_markAnnouncementAsRead"#), path: r#"/announcements/{announcementId}/read"#, security: None, method: "PUT" },
    Operation { operation_id: Some(r#"Auth_appInfo"#), path: r#"/auth/client"#, security: None, method: "PATCH" },
    Operation { operation_id: Some(r#"Auth_login"#), path: r#"/auth/login"#, security: Some(&[&[], ]), method: "POST" },
    Operation { operation_id: Some(r#"Auth_logout"#), path: r#"/auth/logout"#, security: None, method: "DELETE" },
    Operation { operation_id: Some(r#"Mfa_fetchChallenge"#), path: r#"/auth/mfa/challenge"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Mfa_enrolMfa"#), path: r#"/auth/mfa/enrol"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"Mfa_getMfaStatus"#), path: r#"/auth/mfa/status"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Mfa_validateMfa"#), path: r#"/auth/mfa/validate"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"Profile_getProfile"#), path: r#"/auth/profile"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Profile_getProfilePicture"#), path: r#"/auth/profile/picture"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Profile_getAccessToken"#), path: r#"/auth/profile/token"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"ServiceLink_getLiveClassLink"#), path: r#"/auth/serviceLink/liveClass/{meetingID}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"ServiceLink_getMailLink"#), path: r#"/auth/serviceLink/mail"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"ServiceLink_getMoodleLink"#), path: r#"/auth/serviceLink/moodle/{course}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"ServiceLink_getSmartCardLink"#), path: r#"/auth/serviceLink/smartcard"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Auth_switchCareer"#), path: r#"/auth/switch-career"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"BookingTopics_getBookingTopics"#), path: r#"/booking-topics"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"BookingTopics_getBookingSlots"#), path: r#"/booking-topics/{bookingTopicId}/slots"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"BookingTopics_getBookingSeats"#), path: r#"/booking-topics/{bookingTopicId}/slots/{bookingSlotId}/seats"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"BookingsManagement_getBookings"#), path: r#"/bookings"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"BookingsManagement_createBooking"#), path: r#"/bookings"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"BookingsManagement_deleteBooking"#), path: r#"/bookings/{bookingId}"#, security: None, method: "DELETE" },
    Operation { operation_id: Some(r#"BookingsManagement_updateBooking"#), path: r#"/bookings/{bookingId}"#, security: None, method: "PATCH" },
    Operation { operation_id: Some(r#"Courses_getCourse"#), path: r#"/courses/{courseId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Courses_getCourseAssignments"#), path: r#"/courses/{courseId}/assignments"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Courses_uploadCourseAssignment"#), path: r#"/courses/{courseId}/assignments"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"Courses_updateAssignment"#), path: r#"/courses/{courseId}/assignments/{assignmentId}"#, security: None, method: "PATCH" },
    Operation { operation_id: Some(r#"Courses_getCourseFiles"#), path: r#"/courses/{courseId}/files"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Courses_getCourseFile"#), path: r#"/courses/{courseId}/files/{fileId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Courses_getCourseGuide"#), path: r#"/courses/{courseId}/guide"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Courses_getNextLecture"#), path: r#"/courses/{courseId}/nextLecture"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Courses_getCourseNotices"#), path: r#"/courses/{courseId}/notices"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Courses_updateCoursePreferences"#), path: r#"/courses/{courseId}/preferences"#, security: None, method: "PATCH" },
    Operation { operation_id: Some(r#"Courses_getCourseVideolectures"#), path: r#"/courses/{courseId}/videolectures"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Courses_getCourseVirtualClassrooms"#), path: r#"/courses/{courseId}/virtual-classrooms"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"StudentNs_getDeadlines"#), path: r#"/deadlines"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Departments_getDepartments"#), path: r#"/departments"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Esc_escGet"#), path: r#"/esc"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Esc_escDelete"#), path: r#"/esc"#, security: None, method: "DELETE" },
    Operation { operation_id: Some(r#"Esc_escRequest"#), path: r#"/esc/create"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"EventAdmission_getEventAdmissions"#), path: r#"/event-admissions"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Exams_getExams"#), path: r#"/exams"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Exams_bookExam"#), path: r#"/exams/{examId}/booking"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"Exams_deleteExamBookingById"#), path: r#"/exams/{examId}/booking"#, security: None, method: "DELETE" },
    Operation { operation_id: Some(r#"Exams_rescheduleExam"#), path: r#"/exams/{examId}/rescheduleRequest"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"StudentNs_getStudentGrades"#), path: r#"/grades"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"StudentNs_getGuides"#), path: r#"/guides"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"JobOffers_getJobOffers"#), path: r#"/job-offers"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"JobOffers_getJobOffer"#), path: r#"/job-offers/{jobOfferId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Lectures_getLectures"#), path: r#"/lectures"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Messages_getMessages"#), path: r#"/messages"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Messages_deleteMessage"#), path: r#"/messages/{messageId}"#, security: None, method: "DELETE" },
    Operation { operation_id: Some(r#"Messages_markMessageAsRead"#), path: r#"/messages/{messageId}/read"#, security: None, method: "PUT" },
    Operation { operation_id: Some(r#"News_getNews"#), path: r#"/news"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"News_getNewsItem"#), path: r#"/news/{newsItemId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Notifications_getNotifications"#), path: r#"/notifications"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Notifications_getNotificationPreferences"#), path: r#"/notifications/preferences"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Notifications_updateNotificationPreferences"#), path: r#"/notifications/preferences"#, security: None, method: "PATCH" },
    Operation { operation_id: Some(r#"Notifications_deleteNotification"#), path: r#"/notifications/{notificationId}"#, security: None, method: "DELETE" },
    Operation { operation_id: Some(r#"Notifications_markNotificationAsRead"#), path: r#"/notifications/{notificationId}/read"#, security: None, method: "PUT" },
    Operation { operation_id: Some(r#"Offering_getOffering"#), path: r#"/offering"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Offering_getOfferingCourse"#), path: r#"/offering/courses/{courseShortcode}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Offering_getCourseStatistics"#), path: r#"/offering/courses/{courseShortcode}/statistics"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Offering_getOfferingDegree"#), path: r#"/offering/degrees/{degreeId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"People_getPeople"#), path: r#"/people"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"People_getPerson"#), path: r#"/people/{personId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"StudentNs_updateDevicePreferences"#), path: r#"/preferences"#, security: None, method: "PATCH" },
    Operation { operation_id: Some(r#"ProvisionalGrades_getStudentProvisionalGrades"#), path: r#"/provisional-grades"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"ProvisionalGrades_acceptProvisionalGrade"#), path: r#"/provisional-grades/{provisionalGradeId}/accept"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"ProvisionalGrades_rejectProvisionalGrade"#), path: r#"/provisional-grades/{provisionalGradeId}/reject"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"StudentNew_getStudentCareer"#), path: r#"/student/career"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"TicketFAQs_searchTicketFAQs"#), path: r#"/student/ticket-faqs"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"TicketTopics_getTicketTopics"#), path: r#"/student/ticket-topics"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Tickets_getTickets"#), path: r#"/student/tickets"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Tickets_createTicket"#), path: r#"/student/tickets"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"Tickets_getTicket"#), path: r#"/student/tickets/{ticketId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Tickets_getTicketAttachment"#), path: r#"/student/tickets/{ticketId}/attachments/{attachmentId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Tickets_provideTicketFeedback"#), path: r#"/student/tickets/{ticketId}/feedback"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"Tickets_markTicketAsRead"#), path: r#"/student/tickets/{ticketId}/read"#, security: None, method: "PUT" },
    Operation { operation_id: Some(r#"Tickets_replyToTicket"#), path: r#"/student/tickets/{ticketId}/replies"#, security: None, method: "POST" },
    Operation { operation_id: Some(r#"Tickets_getTicketReplyAttachment"#), path: r#"/student/tickets/{ticketId}/replies/{replyId}/attachments/{attachmentId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Tickets_setTicketReplyFeedback"#), path: r#"/student/tickets/{ticketId}/replies/{replyId}/feedback"#, security: None, method: "PUT" },
    Operation { operation_id: Some(r#"Tickets_markTicketAsClosed"#), path: r#"/student/tickets/{ticketId}/resolve"#, security: None, method: "PUT" },
    Operation { operation_id: Some(r#"Surveys_getSurveys"#), path: r#"/surveys"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"StudentNs_getUnreadEmailsNumber"#), path: r#"/unreadEmails"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"CoursesV2_getCourses"#), path: r#"/v2/courses"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"PlaceCategories_getPlaceCategories"#), path: r#"/v2/place-categories"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Places_getPlaces"#), path: r#"/v2/places"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Places_getPlace"#), path: r#"/v2/places/{placeId}"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Sites_getSites"#), path: r#"/v2/sites"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Sites_getBuildings"#), path: r#"/v2/sites/{siteId}/buildings"#, security: None, method: "GET" },
    Operation { operation_id: Some(r#"Sites_getFreeRooms"#), path: r#"/v2/sites/{siteId}/free-rooms"#, security: None, method: "GET" },
];
