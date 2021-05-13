use anyhow::Result;

fn main() -> Result<()> {
    let content = std::fs::read_to_string("./email_template/alert_sample.html")?;
    let json = serde_json::json!({
        "Template": {
          "TemplateName": "CovinAlert",
          "SubjectPart": "Covin Notification!",
          "HtmlPart": content,
          "TextPart": r###"Text email template is not available for now.
We are working on to get a text email ready soon! 
We appologize for the inconvenience"###
        }
    });
    std::fs::write("./email_template/alert_template.json", json.to_string())?;
    Ok(())
}
