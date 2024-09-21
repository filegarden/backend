//! Utilities for sending emails.

use std::sync::LazyLock;

use askama::Template;
use lettre::{
    message::{Mailbox, MultiPart},
    transport::smtp::{authentication::Credentials, extension::ClientId},
    AsyncSmtpTransport, Message, Tokio1Executor,
};

/// An email template asking a user to verify their email.
#[derive(Template)]
#[template(path = "email/verification.html")]
pub(crate) struct VerificationMessage<'a> {
    /// The email address being verified.
    pub(crate) email: &'a str,

    /// The URL the user must visit to verify their email.
    pub(crate) verification_url: &'a str,
}

impl MessageTemplate for VerificationMessage<'_> {
    fn subject(&self) -> String {
        "Verify your email - File Garden".into()
    }
}

/// The SMTP transport used to send automated emails.
pub(crate) static MAILER: LazyLock<AsyncSmtpTransport<Tokio1Executor>> = LazyLock::new(|| {
    let hostname =
        dotenvy::var("SMTP_HOSTNAME").expect("environment variable `SMTP_HOSTNAME` should be set");
    let username =
        dotenvy::var("SMTP_USERNAME").expect("environment variable `SMTP_USERNAME` should be set");
    let password =
        dotenvy::var("SMTP_PASSWORD").expect("environment variable `SMTP_PASSWORD` should be set");

    AsyncSmtpTransport::<Tokio1Executor>::relay(&hostname)
        .expect("SMTP relay couldn't be initialized")
        .credentials(Credentials::new(username, password))
        .build()
});

/// The mailbox automated emails are sent from.
static FROM_MAILBOX: LazyLock<Mailbox> = LazyLock::new(|| {
    dotenvy::var("FROM_MAILBOX")
        .expect("environment variable `FROM_MAILBOX` should be set")
        .parse()
        .expect("environment variable `FROM_MAILBOX` should be a valid mailbox")
});

/// An HTML [`Template`] for an email message.
pub(crate) trait MessageTemplate: Template {
    /// Gets the message's subject line.
    fn subject(&self) -> String;

    /// Generates a multipart HTML and plain text body for the email message template.
    fn to(&self, mailbox: Mailbox) -> Message {
        let html = self.to_string();
        let plain = html2text::from_read(html.as_bytes(), usize::MAX);

        Message::builder()
            .from(FROM_MAILBOX.clone())
            .to(mailbox)
            .subject(self.subject())
            .multipart(MultiPart::alternative_plain_html(plain, html))
            .expect("message should be valid")
    }
}
