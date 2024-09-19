pub fn to_markdown(content: impl ToString) -> ::askama::Result<String> {
    Ok(comrak::markdown_to_html(&content.to_string(), &comrak::Options::default()))
}