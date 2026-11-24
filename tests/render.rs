use mdhtml2term::{highlight, highlight_spans, render};
use strip_ansi_escapes::strip_str;
use unicode_width::UnicodeWidthStr;

#[test]
fn scientific_reply_reflows_without_losing_its_meaning() {
    let md = r#"# Training

**See** [@ruder]{ref=text}.

| Model | Description                    | Count |
|:------|:-------------------------------|------:|
| base  | A short run                    |     7 |
| tuned | Full training with a warmup    |  1234 |
: Training runs {#tbl-runs}

- [Data](https://data.test)
- <https://run.test>[^settings]

```python
def train():
    return "done"
```

[^settings]: Use the same optimizer.

::: bibliography
Howard and Ruder {#ruder year=2018}
: Jeremy Howard and Sebastian Ruder. *ULMFiT*.
:::"#;
    for width in [22, 72] {
        let sections = mdhtml2term::sections(md, width);
        let sources: Vec<_> = sections.iter().map(|s| s.source.trim()).collect();
        assert_eq!(&sources[..2], ["# Training", "**See** [@ruder]{ref=text}."]);
        assert!(sources[2].starts_with("| Model") && sources[2].ends_with(": Training runs {#tbl-runs}"));
        assert!(sources[3].starts_with("- [Data]") && sources[3].contains("\n- <https://run.test>"));
        assert!(sources.last().unwrap().starts_with("[^settings]:"));
        assert!(sections[0].rows[0].contains("\x1b[1m"));
        assert_eq!(sections.iter().filter(|s| s.code && s.source.starts_with("```python")).count(), 1);
        let blocks = sections.into_iter().flat_map(|s| s.rows).collect();
        for rows in [blocks, render(md, width)] {
            assert!(rows.iter().any(|r| r.contains("\x1b[38;2;") && r.contains("return")));
            let plain: Vec<_> = rows.iter().map(strip_str).collect();
            assert!(plain.iter().all(|r| r.width() <= usize::from(width) && !r.contains('\n')));
            let text = plain.join(" ");
            assert!(text.contains("Howard and Ruder (2018)") && text.contains("ULMFiT"));
            assert!(text.contains("Table 1: Training runs") && text.contains("optimizer"));
            assert!(text.contains("Data") && text.matches("https://data.test").count() == 1 && text.matches("https://run.test").count() == 1);
            let end = |value: &str| {
                let row = plain.iter().find(|r| r.contains(value)).unwrap();
                row[..row.find(value).unwrap()].width() + value.len()
            };
            assert_eq!(end("7"), end("1234"));
        }
    }
    let invalid = "# Results {#same}\n\n# Discussion {#same}";
    let fallback = mdhtml2term::sections(invalid, 72);
    assert_eq!(fallback.len(), 1);
    assert_eq!(fallback[0].source, invalid);
    assert_eq!(fallback[0].rows.join("\n"), invalid);
    assert_eq!(render(invalid, 72), fallback[0].rows);

    assert!(!render("not an escape: \x1b]52;c;AA==\x07", 80).join("").contains("\x1b]"));
    let narrow = render("> 1. We say **café** 👩‍💻 and `code`.\n> 2. [label](https://test.invalid)\n\n|A|B|C|D|\n|---|---|---|---|\n|α|β|γ|δ|", 4);
    let plain: Vec<_> = narrow.iter().map(strip_str).collect();
    assert!(plain.iter().all(|s| s.width() <= 4));
    let text = plain.concat();
    assert!("αβγδ".chars().all(|ch| text.contains(ch)));
}

#[test]
fn a_notebook_shows_literal_source_and_rendered_html_output() {
    let html = "<strong>**Café**</strong> &amp; π";
    let code = format!("def label():\n\treturn '{html}'  \n");
    let spans = highlight_spans(&code, "python");
    assert_eq!(spans.iter().map(|(_, text)| *text).collect::<String>(), code);
    let style = |word: &str| spans.iter().find(|(_, text)| text.contains(word)).unwrap().0.as_str();
    assert!(style("return").starts_with("\x1b[") && style("return") != style("label"));

    let shown = highlight(&code, "python", 80).join("\n");
    assert!(shown.contains("\x1b[38;2;") && strip_str(shown).contains(html));
    assert_eq!(strip_str(highlight(html, "unknown", 80).join("")), html);

    let output = mdhtml2term::render_html(html, 80).join("");
    assert_eq!(strip_str(&output), "**Café** & π");
    assert!(output.contains("\x1b[1m**Café**\x1b[0m"));
}
