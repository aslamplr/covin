use tera::{Context as TeraContext, Tera};

use super::alert_session::AlertSession;

pub trait TemplateEngine {
    type Error: std::error::Error + Sync + Send + 'static;

    fn generate_alert_content(
        &self,
        sessions_to_alert: &[AlertSession],
    ) -> Result<String, Self::Error>;
}

pub struct TeraTemplateEngine {
    tera: Tera,
}

impl TeraTemplateEngine {
    pub fn try_init() -> Result<Self, tera::Error> {
        Ok(Self {
            tera: Self::get_tera_template()?,
        })
    }

    pub fn get_tera_template() -> Result<Tera, tera::Error> {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
      ("container", r###"
      {%- for session in sessions -%}
          {%- include "available_session" -%}
      {%- endfor -%}
      "###),
      (
          "available_session",
          r###"
<tr style="border-collapse:collapse">
<td align="left" style="margin:0;padding-top:5px;padding-bottom:5px;padding-left:40px;padding-right:40px">
<table width="100%" cellspacing="0" cellpadding="0" style="mso-table-lspace:0pt;mso-table-rspace:0pt;border-collapse:collapse;border-spacing:0px">
 <tr style="border-collapse:collapse">
  <td valign="top" align="center" style="padding:0;margin:0;width:518px">
   <table style="mso-table-lspace:0pt;mso-table-rspace:0pt;border-collapse:separate;border-spacing:0px;border-left:3px solid #6aa84f;border-right:1px solid #dddddd;border-top:1px solid #dddddd;border-bottom:1px solid #dddddd;background-color:#ffffff;border-top-left-radius:2px;border-top-right-radius:2px;border-bottom-right-radius:2px;border-bottom-left-radius:2px" width="100%" cellspacing="0" cellpadding="0" bgcolor="#ffffff" role="presentation">
    <tr style="border-collapse:collapse">
     <td style="padding:0;margin:0;padding-top:5px;padding-bottom:5px;padding-left:5px">
      <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
       {{ session.center.name }}, {{ session.center.block_name }}, {{ session.center.district_name }}, {{ session.center.pincode }}
      </p>
     </td>
    </tr>
    <tr style="border-collapse:collapse">
     <td style="padding:5px;margin:0">
      <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
          fee type: {{ session.center.fee_type }}
      </p>
      <hr />
      <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
          date: {{ session.session.date }}
      </p>
      <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
          available capacity (all): {{ session.session.available_capacity }}
      </p>
      <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
          available capacity (dose 1): {{ session.session.available_capacity_dose1 }}
      </p>
      <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
          available capacity (dose 2): {{ session.session.available_capacity_dose2 }}
      </p>
      <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
          min age limit: {{ session.session.min_age_limit }}
      </p>
      <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
          slots: {{ session.session.slots | join(sep = ", ") }}
      </p>
     </td>
    </tr></table></td></tr></table></td></tr>"###,
      ),
  ])?;
        Ok(tera)
    }
}

impl TemplateEngine for TeraTemplateEngine {
    type Error = tera::Error;

    #[tracing::instrument(level = "debug", skip(self))]
    fn generate_alert_content(
        &self,
        sessions_to_alert: &[AlertSession],
    ) -> Result<String, Self::Error> {
        let mut tera_context = TeraContext::new();
        tera_context.insert("sessions", &sessions_to_alert);
        let content = self.tera.render("container", &tera_context)?;
        Ok(content)
    }
}

#[cfg(test)]
mod test {
    use super::{TemplateEngine, TeraTemplateEngine};
    use crate::{
        alert_engine::alert_session::AlertSession,
        covin::centers::{Center, Session},
    };

    #[test]
    fn test_email_template() {
        let template_engine = TeraTemplateEngine::try_init().unwrap();

        let mut alert_content = template_engine
            .generate_alert_content(&[AlertSession {
                center: &Center {
                    center_id: 1,
                    name: "Dummy Center 1".to_string(),
                    block_name: "Dummy Block".to_string(),
                    district_name: "Dummy District".to_string(),
                    fee_type: "Free".to_string(),
                    pincode: 612343,
                    from: "09:00:00".to_string(),
                    state_name: "Kerala".to_string(),
                    ..Default::default()
                },
                session: &Session {
                    session_id: "dummy-session-1".to_string(),
                    date: "12-01-2021".to_string(),
                    min_age_limit: 18,
                    available_capacity: 1_f32,
                    available_capacity_dose1: 1_f32,
                    available_capacity_dose2: 0_f32,
                    slots: vec![],
                },
            }])
            .unwrap();

        let mut expected_alert_content = r###"
<tr style="border-collapse:collapse">
<td align="left" style="margin:0;padding-top:5px;padding-bottom:5px;padding-left:40px;padding-right:40px">
<table width="100%" cellspacing="0" cellpadding="0" style="mso-table-lspace:0pt;mso-table-rspace:0pt;border-collapse:collapse;border-spacing:0px">
 <tr style="border-collapse:collapse">
   <td valign="top" align="center" style="padding:0;margin:0;width:518px">
    <table style="mso-table-lspace:0pt;mso-table-rspace:0pt;border-collapse:separate;border-spacing:0px;border-left:3px solid #6aa84f;border-right:1px solid #dddddd;border-top:1px solid #dddddd;border-bottom:1px solid #dddddd;background-color:#ffffff;border-top-left-radius:2px;border-top-right-radius:2px;border-bottom-right-radius:2px;border-bottom-left-radius:2px" width="100%" cellspacing="0" cellpadding="0" bgcolor="#ffffff" role="presentation">
     <tr style="border-collapse:collapse">
      <td style="padding:0;margin:0;padding-top:5px;padding-bottom:5px;padding-left:5px">
       <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
        Dummy Center 1, Dummy Block, Dummy District, 612343
       </p>
      </td>
     </tr>
     <tr style="border-collapse:collapse">
      <td style="padding:5px;margin:0">
       <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
           fee type: Free
       </p>
       <hr />
       <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
           date: 12-01-2021
       </p>
       <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
           available capacity (all): 1
       </p>
       <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
           available capacity (dose 1): 1
       </p>
       <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
           available capacity (dose 2): 0
       </p>
       <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
           min age limit: 18
       </p>
       <p style="margin:0;-webkit-text-size-adjust:none;-ms-text-size-adjust:none;mso-line-height-rule:exactly;font-family:helvetica, 'helvetica neue', arial, verdana, sans-serif;line-height:23px;color:#555555;font-size:15px">
           slots: 
       </p>
      </td>
     </tr></table></td></tr></table></td></tr>"###.to_string();

        alert_content.retain(|c| !c.is_whitespace());
        expected_alert_content.retain(|c| !c.is_whitespace());

        assert_eq!(alert_content, expected_alert_content);
    }
}
