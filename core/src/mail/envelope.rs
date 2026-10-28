use std::collections::HashMap;

/// Represents the envelope of an email (from, to, subject, etc.).
/// inspired by Laravel Envelope
/// as possible while staying idiomatic and explicit Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Envelope {
    /// Sender address. None means "use the default from the mailer config".
    pub from: Option<Address>,

    /// Primary recipients.
    pub to: Vec<Address>,

    /// Carbon copy recipients.
    pub cc: Vec<Address>,

    /// Blind carbon copy recipients.
    pub bcc: Vec<Address>,

    /// Reply-To addresses.
    pub reply_to: Vec<Address>,

    /// Email subject. None means the subject will be set later or left empty.
    pub subject: Option<String>,

    /// Optional tags (useful for some providers like Mailgun, Postmark...).
    pub tags: Vec<String>,

    /// Arbitrary metadata key-value pairs.
    pub metadata: std::collections::HashMap<String, String>,
}

/// A simple email address with optional display name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    pub email: String,
    pub name: Option<String>,
}

impl Address {
    /// Creates a new Address.
    ///
    /// # Examples
    /// ```
    /// use rustavel_core::mail::envelope::Address;
    /// let addr = Address::new("user@example.com");
    /// ```
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    /// Creates an address with both email and display name.
    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }
}

impl Envelope {
    /// Creates an empty envelope. All fields start empty/None.
    /// Prefer setting fields explicitly after creation.
    pub fn new() -> Self {
        Self {
            from: None,
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: Vec::new(),
            subject: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Sets the subject. Returns self for optional chaining if you want it.
    /// We keep the method because subject is the most commonly set field.
    pub fn set_subject(&mut self, subject: impl Into<String>) {
        self.subject = Some(subject.into());
    }

    /// Adds one or more "to" recipients.
    pub fn add_to<I>(&mut self, addresses: I)
    where
        I: IntoIterator<Item = Address>,
    {
        self.to.extend(addresses);
    }

    /// Adds one or more "cc" recipients.
    pub fn add_cc<I>(&mut self, addresses: I)
    where
        I: IntoIterator<Item = Address>,
    {
        self.cc.extend(addresses);
    }

    /// Adds one or more "bcc" recipients.
    pub fn add_bcc<I>(&mut self, addresses: I)
    where
        I: IntoIterator<Item = Address>,
    {
        self.bcc.extend(addresses);
    }

    /// Sets the from address.
    pub fn set_from(&mut self, address: Address) {
        self.from = Some(address);
    }

    /// Adds reply-to addresses.
    pub fn add_reply_to<I>(&mut self, addresses: I)
    where
        I: IntoIterator<Item = Address>,
    {
        self.reply_to.extend(addresses);
    }

    // single tag
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    /// add some tags
    pub fn add_tags<I>(&mut self, tags: I)
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(Into::into));
    }

    /// set meta data
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

}

impl Default for Envelope {
    fn default() -> Self {
        Self::new()
    }
}
