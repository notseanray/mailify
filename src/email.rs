use lettre::{
    address::AddressError, transport::smtp::authentication::Credentials, Message, SmtpTransport,
    Transport,
};

use crate::config::Config;

pub fn email(
    body: &str,
    self_address: &str,
    address: &str,
    subject: &str,
    config: &Config,
) -> Result<(), AddressError> {
    let message = Message::builder()
        .from(self_address.parse().unwrap())
        .to(address.parse().unwrap())
        .subject(subject)
        .body(body.to_string())
        .unwrap();

    let creds = Credentials::new(config.username.clone(), config.password.clone());
    let mail_server = match &config.smtp {
        Some(v) => v,
        _ => "smpt.gmail.com",
    };

    // Open a remote connection to gmail
    let mailer = SmtpTransport::starttls_relay(mail_server)
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&message) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
    Ok(())
}
