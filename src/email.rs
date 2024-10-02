//! Utilities for sending emails.

use std::{env::VarError, sync::LazyLock};

use askama::Template;
use html2text::render::text_renderer::TrivialDecorator;
use lettre::{
    message::{Mailbox, MultiPart},
    transport::smtp::{authentication::Credentials, extension::ClientId},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
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

/// An email template informing a user that someone tried to sign up with their email despite them
/// already having an account.
#[derive(Template)]
#[template(path = "email/email_taken.html")]
pub(crate) struct EmailTakenMessage<'a> {
    /// The email address used to try to sign up.
    pub(crate) email: &'a str,
}

impl MessageTemplate for EmailTakenMessage<'_> {
    fn subject(&self) -> String {
        "Sign-up failed for existing account - File Garden".into()
    }
}

/// The mailbox automated emails are sent from.
static FROM_MAILBOX: LazyLock<Mailbox> = LazyLock::new(|| {
    dotenvy::var("FROM_MAILBOX")
        .expect("environment variable `FROM_MAILBOX` should be a valid string")
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
        let plain = html2text::config::with_decorator(TrivialDecorator::new())
            .string_from_read(html.as_bytes(), usize::MAX)
            .expect("message HTML should be convertible to text");

        Message::builder()
            .from(FROM_MAILBOX.clone())
            .to(mailbox)
            .subject(self.subject())
            .multipart(MultiPart::alternative_plain_html(plain, html))
            .expect("message should be valid")
    }
}

/// The SMTP transport used to send automated emails.
static MAILER: LazyLock<AsyncSmtpTransport<Tokio1Executor>> = LazyLock::new(|| {
    let hostname = dotenvy::var("SMTP_HOSTNAME")
        .expect("environment variable `SMTP_HOSTNAME` should be a valid string");
    let username = dotenvy::var("SMTP_USERNAME")
        .expect("environment variable `SMTP_USERNAME` should be a valid string");
    let password = dotenvy::var("SMTP_PASSWORD")
        .expect("environment variable `SMTP_PASSWORD` should be a valid string");

    let mut smtp_transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&hostname)
        .expect("SMTP relay couldn't be initialized")
        .credentials(Credentials::new(username, password));

    match dotenvy::var("SMTP_HELO_DOMAIN") {
        // If the environment variable is unset, let `lettre` default to using the OS hostname.
        Err(dotenvy::Error::EnvVar(VarError::NotPresent)) => {}

        helo_domain => {
            let helo_domain = helo_domain
                .expect("environment variable `SMTP_HELO_DOMAIN` should be a valid string if set");

            smtp_transport = smtp_transport.hello_name(ClientId::Domain(helo_domain));
        }
    }

    smtp_transport.build()
});

/// A trait for sending messages using the SMTP configuration from `.env`.
pub(crate) trait SendMessage {
    /// Sends the message in the background.
    ///
    /// Errors are ignored so they can't propagate to end users. Otherwise, users could tell if an
    /// email sent successfully or not, which can allow for user enumeration in some circumstances.
    fn send(self);
}

impl SendMessage for Message {
    fn send(self) {
        tokio::spawn(MAILER.send(self));
    }
}
