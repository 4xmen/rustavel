#[cfg(test)]
mod tests {
    use rustavel_core::mail::envelope::{Address, Envelope};

    #[test]
    fn creates_empty_envelope_by_default() {
        let envelope = Envelope::new();

        assert_eq!(envelope.from, None);
        assert!(envelope.to.is_empty());
        assert!(envelope.cc.is_empty());
        assert!(envelope.bcc.is_empty());
        assert!(envelope.reply_to.is_empty());
        assert_eq!(envelope.subject, None);
        assert!(envelope.tags.is_empty());
        assert!(envelope.metadata.is_empty());
    }

    #[test]
    fn creates_address_without_name() {
        let address = Address::new("user@example.com");

        assert_eq!(address.email, "user@example.com");
        assert_eq!(address.name, None);
    }

    #[test]
    fn creates_address_with_name() {
        let address =
            Address::with_name("user@example.com", "John Doe");

        assert_eq!(address.email, "user@example.com");
        assert_eq!(address.name.as_deref(), Some("John Doe"));
    }

    #[test]
    fn adds_recipients_without_replacing_existing_ones() {
        let mut envelope = Envelope::new();

        envelope.add_to([
            Address::new("alice@example.com"),
        ]);

        envelope.add_to([
            Address::new("bob@example.com"),
        ]);

        assert_eq!(envelope.to.len(), 2);
        assert_eq!(envelope.to[0].email, "alice@example.com");
        assert_eq!(envelope.to[1].email, "bob@example.com");
    }

    #[test]
    fn sets_subject_and_from() {
        let mut envelope = Envelope::new();

        envelope.set_subject("Order shipped");
        envelope.set_from(
            Address::with_name("orders@example.com", "Orders"),
        );

        assert_eq!(
            envelope.subject.as_deref(),
            Some("Order shipped")
        );

        assert_eq!(
            envelope.from.as_ref().unwrap().email,
            "orders@example.com"
        );
    }

    #[test]
    fn adds_tags_and_metadata() {
        let mut envelope = Envelope::new();

        envelope.add_tag("orders");
        envelope.add_tags(["transactional", "important"]);

        envelope.set_metadata("order_id", "123");
        envelope.set_metadata("order_id", "456");

        assert_eq!(
            envelope.tags,
            vec!["orders", "transactional", "important"]
        );

        assert_eq!(
            envelope.metadata.get("order_id").map(String::as_str),
            Some("456")
        );
    }

    #[test]
    fn default_is_equivalent_to_new() {
        assert_eq!(Envelope::default(), Envelope::new());
    }
}