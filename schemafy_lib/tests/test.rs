use schemafy_lib::Expander;

#[test]
fn schema() {
    let json = std::fs::read_to_string("src/schema.json").expect("Read schema JSON file");

    let schema = serde_json::from_str(&json).unwrap_or_else(|err| panic!("{}", err));
    let mut expander = Expander::new(Some("Schema"), "UNUSED", &schema);

    expander.expand(&schema);
}

#[test]
fn test_str_to_ident() {
    use proc_macro2::Span;
    use schemafy_lib::str_to_ident;
    use syn::Ident;

    assert_eq!(
        str_to_ident("normalField"),
        Ident::new("normalField", Span::call_site())
    );

    assert_eq!(str_to_ident("ref"), Ident::new("ref_", Span::call_site()));

    assert_eq!(str_to_ident(""), Ident::new("empty_", Span::call_site()));
    assert_eq!(
        str_to_ident("_"),
        Ident::new("underscore_", Span::call_site())
    );
    assert_eq!(
        str_to_ident("__"),
        Ident::new("underscore_", Span::call_site())
    );

    assert_eq!(str_to_ident("_7_"), Ident::new("_7_", Span::call_site()));
    assert_eq!(
        str_to_ident("thieves' tools"),
        // only one underscore
        Ident::new("thieves_tools", Span::call_site())
    );
}
