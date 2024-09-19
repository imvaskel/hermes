use std::sync::LazyLock;

use comrak::{plugins::syntect::SyntectAdapter, Plugins, PluginsBuilder, RenderPluginsBuilder};

static SYNTECT_PLUGIN: LazyLock<SyntectAdapter> = LazyLock::new(|| {
    comrak::plugins::syntect::SyntectAdapterBuilder::new()
        .theme("base16-eighties.dark")
        .build()
});

static PLUGINS: LazyLock<Plugins> = LazyLock::new(|| {
    PluginsBuilder::default()
        .render(
            RenderPluginsBuilder::default()
                .codefence_syntax_highlighter(Some(&*SYNTECT_PLUGIN))
                .build()
                .unwrap(),
        )
        .build()
        .unwrap()
});

pub fn to_markdown(content: impl ToString) -> ::askama::Result<String> {
    let mut options = comrak::Options::default();
    options.extension.autolink = true;
    Ok(comrak::markdown_to_html_with_plugins(
        &content.to_string().replace("\n\n", "\n"),
        &options,
        &PLUGINS,
    ))
}
