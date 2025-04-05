pub use markdown::*;

pub const SLIDE_MARKDOWN_PARSE_OPTIONS: ParseOptions = ParseOptions {
    constructs: Constructs {
        attention: true,
        autolink: true,
        block_quote: false,
        character_escape: false,
        character_reference: true,
        code_indented: false,
        code_fenced: false,
        code_text: true,
        definition: false,
        frontmatter: false,
        gfm_autolink_literal: false,
        gfm_footnote_definition: false,
        gfm_label_start_footnote: false,
        gfm_strikethrough: true,
        gfm_table: false,
        gfm_task_list_item: false,
        hard_break_escape: false,
        hard_break_trailing: false,
        heading_atx: true,
        heading_setext: false,
        html_flow: false,
        html_text: false,
        label_start_image: false,
        label_start_link: false,
        label_end: false,
        list_item: true,
        math_flow: true,
        math_text: true,
        mdx_esm: false,
        mdx_expression_flow: false,
        mdx_expression_text: false,
        mdx_jsx_flow: false,
        mdx_jsx_text: false,
        thematic_break: false,
    },
    gfm_strikethrough_single_tilde: true,
    math_text_single_dollar: true,
    mdx_expression_parse: None,
    mdx_esm_parse: None,
};

pub fn render_markdown<W: std::io::Write>(w: &mut W, node: mdast::Node) -> std::io::Result<()> {
    match node {
        mdast::Node::Root(root) => render_root(w, root),
        mdast::Node::Blockquote(blockquote) => todo!(),
        mdast::Node::FootnoteDefinition(footnote_definition) => todo!(),
        mdast::Node::MdxJsxFlowElement(mdx_jsx_flow_element) => todo!(),
        mdast::Node::List(list) => render_list(w, list),
        mdast::Node::MdxjsEsm(mdxjs_esm) => todo!(),
        mdast::Node::Toml(toml) => todo!(),
        mdast::Node::Yaml(yaml) => todo!(),
        mdast::Node::Break(_) => todo!(),
        mdast::Node::InlineCode(inline_code) => todo!(),
        mdast::Node::InlineMath(inline_math) => todo!(),
        mdast::Node::Delete(delete) => todo!(),
        mdast::Node::Emphasis(emphasis) => render_emphasis(w, emphasis),
        mdast::Node::MdxTextExpression(mdx_text_expression) => todo!(),
        mdast::Node::FootnoteReference(footnote_reference) => todo!(),
        mdast::Node::Html(html) => todo!(),
        mdast::Node::Image(image) => todo!(),
        mdast::Node::ImageReference(image_reference) => todo!(),
        mdast::Node::MdxJsxTextElement(mdx_jsx_text_element) => todo!(),
        mdast::Node::Link(link) => todo!(),
        mdast::Node::LinkReference(link_reference) => todo!(),
        mdast::Node::Strong(strong) => todo!(),
        mdast::Node::Text(text) => render_text(w, text),
        mdast::Node::Code(code) => todo!(),
        mdast::Node::Math(math) => todo!(),
        mdast::Node::MdxFlowExpression(mdx_flow_expression) => todo!(),
        mdast::Node::Heading(heading) => render_heading(w, heading),
        mdast::Node::Table(table) => todo!(),
        mdast::Node::ThematicBreak(thematic_break) => todo!(),
        mdast::Node::TableRow(table_row) => todo!(),
        mdast::Node::TableCell(table_cell) => todo!(),
        mdast::Node::ListItem(list_item) => render_list_item(w, list_item),
        mdast::Node::Definition(definition) => todo!(),
        mdast::Node::Paragraph(paragraph) => render_paragraph(w, paragraph),
    }
}

fn render_emphasis<W: std::io::Write>(
    w: &mut W,
    emphasis: mdast::Emphasis,
) -> Result<(), std::io::Error> {
    write!(w, "<span class=\"emphasis\">")?;
    for child in emphasis.children {
        render_markdown(w, child)?;
    }
    write!(w, "</span>")?;
    Ok(())
}

fn render_list_item<W: std::io::Write>(
    w: &mut W,
    list_item: mdast::ListItem,
) -> Result<(), std::io::Error> {
    writeln!(w, "  <li>")?;
    for child in list_item.children {
        render_markdown(w, child)?;
    }
    Ok(())
}

fn render_list<W: std::io::Write>(w: &mut W, list: mdast::List) -> Result<(), std::io::Error> {
    let tag = if list.ordered { "ol" } else { "ul" };
    writeln!(w, "<{tag} class=\"label-text\">")?;
    for child in list.children {
        render_markdown(w, child)?;
    }
    writeln!(w, "</{tag}>")?;
    Ok(())
}

fn render_heading<W: std::io::Write>(
    w: &mut W,
    heading: mdast::Heading,
) -> Result<(), std::io::Error> {
    writeln!(w, "<h{} class=\"label-header\">", heading.depth)?;
    for child in heading.children {
        render_markdown(w, child)?;
    }
    writeln!(w, "</h{}>", heading.depth)?;
    Ok(())
}

fn render_text<W: std::io::Write>(w: &mut W, text: mdast::Text) -> Result<(), std::io::Error> {
    write!(w, "{}", text.value)
}

fn render_paragraph<W: std::io::Write>(
    w: &mut W,
    paragraph: mdast::Paragraph,
) -> Result<(), std::io::Error> {
    write!(w, "<p class=\"label-text\">")?;
    for child in paragraph.children {
        render_markdown(w, child)?;
    }
    writeln!(w, "</p>")?;
    Ok(())
}

fn render_root<W: std::io::Write>(w: &mut W, root: mdast::Root) -> Result<(), std::io::Error> {
    for child in root.children {
        render_markdown(w, child)?;
    }
    Ok(())
}
