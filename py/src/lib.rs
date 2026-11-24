#[pyo3::pymodule]
mod _core {
    use mdhtml2term::CiteStyle;
    use pyo3::{exceptions::PyValueError, prelude::*};

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> { m.add("__version__", env!("CARGO_PKG_VERSION")) }

    fn style(name: Option<&str>) -> PyResult<Option<CiteStyle>> {
        name.map(CiteStyle::parse).transpose().map_err(PyValueError::new_err)
    }

    /// Original Markdown and rendered terminal rows for a selectable block.
    #[pyclass(module = "mdhtml2term", frozen, get_all)]
    struct Section { source: String, rows: Vec<String>, code: bool }

    /// Render a complete Markdown document as ANSI-styled rows.
    #[pyfunction(signature = (source, width, *, cite_style=None))]
    fn render(source: &str, width: u16, cite_style: Option<&str>) -> PyResult<Vec<String>> {
        Ok(mdhtml2term::render_with_style(source, width, style(cite_style)?))
    }

    /// Render selectable blocks with their original Markdown source.
    #[pyfunction(signature = (source, width, *, cite_style=None))]
    fn sections(source: &str, width: u16, cite_style: Option<&str>) -> PyResult<Vec<Section>> {
        Ok(mdhtml2term::sections_with_style(source, width, style(cite_style)?).into_iter().map(|s|
            Section { source: s.source.into_owned(), rows: s.rows, code: s.code }).collect())
    }

    /// Render an HTML fragment without interpreting Markdown.
    #[pyfunction]
    fn render_html(html: &str, width: u16) -> Vec<String> { mdhtml2term::render_html(html, width) }

    /// Highlight literal source code as terminal rows.
    #[pyfunction]
    fn highlight(text: &str, language: &str, width: u16) -> Vec<String> { mdhtml2term::highlight(text, language, width) }

    /// Return (SGR prefix, original text) pairs without wrapping.
    #[pyfunction]
    fn highlight_spans<'a>(text: &'a str, language: &str) -> Vec<(String, &'a str)> { mdhtml2term::highlight_spans(text, language) }
}
