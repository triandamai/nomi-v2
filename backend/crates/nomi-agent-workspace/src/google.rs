//! Calls to Google's APIs with a user's access token. Each returns compact JSON for the model.

use base64::engine::general_purpose::{URL_SAFE, URL_SAFE_NO_PAD};
use base64::Engine;
use serde_json::{json, Value};

use crate::connection::GoogleConfig;

/// A failed Google call, as the model should read it.
#[derive(Debug, Clone, PartialEq)]
pub enum GoogleError {
    /// 401: the token was rejected (revoked in the user's Google account).
    Unauthorized,
    Failed(String),
}

pub struct Google<'a> {
    pub http: &'a reqwest::Client,
    pub config: &'a GoogleConfig,
    pub token: &'a str,
}

fn url(base: &str, segments: &[&str]) -> Result<reqwest::Url, GoogleError> {
    let mut url = reqwest::Url::parse(base).map_err(|e| GoogleError::Failed(e.to_string()))?;
    url.path_segments_mut().map_err(|_| GoogleError::Failed("bad base url".to_string()))?.pop_if_empty().extend(segments);
    Ok(url)
}

impl Google<'_> {
    async fn send(&self, request: reqwest::RequestBuilder) -> Result<Value, GoogleError> {
        let response = request.bearer_auth(self.token).send().await.map_err(|e| GoogleError::Failed(e.to_string()))?;
        let status = response.status();
        if status.as_u16() == 401 {
            return Err(GoogleError::Unauthorized);
        }
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            let message = serde_json::from_str::<Value>(&body)
                .ok()
                .and_then(|v| v.pointer("/error/message").and_then(|m| m.as_str()).map(str::to_string))
                .unwrap_or(body);
            return Err(GoogleError::Failed(format!("Google returned {status}: {message}")));
        }
        Ok(if body.trim().is_empty() { Value::Null } else { serde_json::from_str(&body).unwrap_or(Value::String(body)) })
    }

    // ---- Gmail ----

    pub async fn gmail_search(&self, query: &str, max: u32) -> Result<Value, GoogleError> {
        let mut list_url = url(&self.config.gmail_base, &["gmail", "v1", "users", "me", "messages"])?;
        list_url.query_pairs_mut().append_pair("q", query).append_pair("maxResults", &max.to_string());
        let list = self.send(self.http.get(list_url)).await?;
        let mut found = Vec::new();
        for id in list.get("messages").and_then(|m| m.as_array()).into_iter().flatten().filter_map(|m| m.get("id").and_then(|i| i.as_str())) {
            let mut get_url = url(&self.config.gmail_base, &["gmail", "v1", "users", "me", "messages", id])?;
            get_url
                .query_pairs_mut()
                .append_pair("format", "metadata")
                .append_pair("metadataHeaders", "From")
                .append_pair("metadataHeaders", "Subject")
                .append_pair("metadataHeaders", "Date");
            let message = self.send(self.http.get(get_url)).await?;
            found.push(json!({
                "id": id,
                "from": header(&message, "From"),
                "subject": header(&message, "Subject"),
                "date": header(&message, "Date"),
                "snippet": message.get("snippet").cloned().unwrap_or(Value::Null),
            }));
        }
        Ok(json!({ "messages": found }))
    }

    pub async fn gmail_read(&self, id: &str) -> Result<Value, GoogleError> {
        let mut get_url = url(&self.config.gmail_base, &["gmail", "v1", "users", "me", "messages", id])?;
        get_url.query_pairs_mut().append_pair("format", "full");
        let message = self.send(self.http.get(get_url)).await?;
        let mut body = String::new();
        if let Some(payload) = message.get("payload") {
            collect_text(payload, &mut body);
        }
        const MAX_BODY: usize = 12_000;
        if body.len() > MAX_BODY {
            let cut = (0..=MAX_BODY).rev().find(|i| body.is_char_boundary(*i)).unwrap_or(0);
            body.truncate(cut);
            body.push_str("\n[… cut]");
        }
        Ok(json!({
            "id": id,
            "thread_id": message.get("threadId"),
            "from": header(&message, "From"),
            "to": header(&message, "To"),
            "subject": header(&message, "Subject"),
            "date": header(&message, "Date"),
            "body": body,
        }))
    }

    /// The raw RFC 822 message, threaded under `reply_to` when given.
    async fn compose(&self, to: &str, subject: &str, body: &str, reply_to: Option<&str>) -> Result<(String, Option<String>), GoogleError> {
        let mut headers = vec![format!("To: {to}")];
        let mut subject = subject.to_string();
        let mut thread_id = None;
        if let Some(id) = reply_to {
            let mut get_url = url(&self.config.gmail_base, &["gmail", "v1", "users", "me", "messages", id])?;
            get_url.query_pairs_mut().append_pair("format", "metadata").append_pair("metadataHeaders", "Message-ID").append_pair("metadataHeaders", "Subject");
            let original = self.send(self.http.get(get_url)).await?;
            thread_id = original.get("threadId").and_then(|t| t.as_str()).map(str::to_string);
            if let Some(message_id) = header(&original, "Message-ID").as_str() {
                headers.push(format!("In-Reply-To: {message_id}"));
                headers.push(format!("References: {message_id}"));
            }
            if subject.is_empty() {
                let original_subject = header(&original, "Subject").as_str().unwrap_or_default().to_string();
                subject = if original_subject.to_lowercase().starts_with("re:") { original_subject } else { format!("Re: {original_subject}") };
            }
        }
        headers.push(format!("Subject: =?UTF-8?B?{}?=", base64::engine::general_purpose::STANDARD.encode(subject.as_bytes())));
        headers.push("MIME-Version: 1.0".to_string());
        headers.push("Content-Type: text/plain; charset=UTF-8".to_string());
        let raw = format!("{}\r\n\r\n{}", headers.join("\r\n"), body);
        Ok((URL_SAFE_NO_PAD.encode(raw.as_bytes()), thread_id))
    }

    pub async fn gmail_draft(&self, to: &str, subject: &str, body: &str, reply_to: Option<&str>) -> Result<Value, GoogleError> {
        let (raw, thread_id) = self.compose(to, subject, body, reply_to).await?;
        let mut message = json!({ "raw": raw });
        if let Some(thread_id) = thread_id {
            message["threadId"] = json!(thread_id);
        }
        let draft = self.send(self.http.post(url(&self.config.gmail_base, &["gmail", "v1", "users", "me", "drafts"])?).json(&json!({ "message": message }))).await?;
        Ok(json!({ "draft_id": draft.get("id"), "status": "draft saved in Gmail" }))
    }

    pub async fn gmail_send(&self, to: &str, subject: &str, body: &str, reply_to: Option<&str>) -> Result<Value, GoogleError> {
        let (raw, thread_id) = self.compose(to, subject, body, reply_to).await?;
        let mut message = json!({ "raw": raw });
        if let Some(thread_id) = thread_id {
            message["threadId"] = json!(thread_id);
        }
        let sent = self.send(self.http.post(url(&self.config.gmail_base, &["gmail", "v1", "users", "me", "messages", "send"])?).json(&message)).await?;
        Ok(json!({ "message_id": sent.get("id"), "status": "sent" }))
    }

    // ---- Sheets ----

    pub async fn sheets_read(&self, spreadsheet_id: &str, range: Option<&str>) -> Result<Value, GoogleError> {
        match range {
            Some(range) => {
                let values = self.send(self.http.get(url(&self.config.sheets_base, &["v4", "spreadsheets", spreadsheet_id, "values", range])?)).await?;
                Ok(json!({ "range": values.get("range"), "values": values.get("values").cloned().unwrap_or(json!([])) }))
            }
            None => {
                let mut info_url = url(&self.config.sheets_base, &["v4", "spreadsheets", spreadsheet_id])?;
                info_url.query_pairs_mut().append_pair("fields", "properties.title,spreadsheetUrl,sheets.properties.title");
                let info = self.send(self.http.get(info_url)).await?;
                let tabs: Vec<Value> = info
                    .get("sheets")
                    .and_then(|s| s.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|s| s.pointer("/properties/title").cloned())
                    .collect();
                Ok(json!({ "title": info.pointer("/properties/title"), "url": info.get("spreadsheetUrl"), "tabs": tabs }))
            }
        }
    }

    pub async fn sheets_append(&self, spreadsheet_id: &str, range: &str, rows: &Value) -> Result<Value, GoogleError> {
        let mut append_url = url(&self.config.sheets_base, &["v4", "spreadsheets", spreadsheet_id, "values", &format!("{range}:append")])?;
        append_url.query_pairs_mut().append_pair("valueInputOption", "USER_ENTERED").append_pair("insertDataOption", "INSERT_ROWS");
        let result = self.send(self.http.post(append_url).json(&json!({ "values": rows }))).await?;
        Ok(json!({ "updated_range": result.pointer("/updates/updatedRange"), "rows_added": result.pointer("/updates/updatedRows") }))
    }

    pub async fn sheets_update(&self, spreadsheet_id: &str, range: &str, rows: &Value) -> Result<Value, GoogleError> {
        let mut update_url = url(&self.config.sheets_base, &["v4", "spreadsheets", spreadsheet_id, "values", range])?;
        update_url.query_pairs_mut().append_pair("valueInputOption", "USER_ENTERED");
        let result = self.send(self.http.put(update_url).json(&json!({ "values": rows }))).await?;
        Ok(json!({ "updated_range": result.get("updatedRange"), "cells_updated": result.get("updatedCells") }))
    }

    pub async fn sheets_create(&self, title: &str) -> Result<Value, GoogleError> {
        let created = self.send(self.http.post(url(&self.config.sheets_base, &["v4", "spreadsheets"])?).json(&json!({ "properties": { "title": title } }))).await?;
        Ok(json!({ "spreadsheet_id": created.get("spreadsheetId"), "url": created.get("spreadsheetUrl") }))
    }

    // ---- Docs ----

    pub async fn docs_read(&self, document_id: &str) -> Result<Value, GoogleError> {
        let doc = self.send(self.http.get(url(&self.config.docs_base, &["v1", "documents", document_id])?)).await?;
        let mut text = String::new();
        for element in doc.pointer("/body/content").and_then(|c| c.as_array()).into_iter().flatten() {
            for run in element.pointer("/paragraph/elements").and_then(|e| e.as_array()).into_iter().flatten() {
                if let Some(content) = run.pointer("/textRun/content").and_then(|c| c.as_str()) {
                    text.push_str(content);
                }
            }
        }
        Ok(json!({ "title": doc.get("title"), "text": text }))
    }

    pub async fn docs_append(&self, document_id: &str, text: &str) -> Result<Value, GoogleError> {
        let request = json!({ "requests": [{ "insertText": { "text": text, "endOfSegmentLocation": {} } }] });
        self.send(self.http.post(url(&self.config.docs_base, &["v1", "documents", &format!("{document_id}:batchUpdate")])?).json(&request)).await?;
        Ok(json!({ "document_id": document_id, "url": doc_link(document_id), "status": "text added" }))
    }

    pub async fn docs_create(&self, title: &str, text: Option<&str>) -> Result<Value, GoogleError> {
        let created = self.send(self.http.post(url(&self.config.docs_base, &["v1", "documents"])?).json(&json!({ "title": title }))).await?;
        let id = created.get("documentId").and_then(|d| d.as_str()).unwrap_or_default().to_string();
        if let Some(text) = text.filter(|t| !t.is_empty()) {
            self.docs_append(&id, text).await?;
        }
        Ok(json!({ "document_id": id, "url": doc_link(&id) }))
    }

    // ---- Drive ----

    pub async fn drive_search(&self, query: &str, max: u32) -> Result<Value, GoogleError> {
        let escaped = query.replace('\\', "\\\\").replace('\'', "\\'");
        let mut search_url = url(&self.config.drive_base, &["drive", "v3", "files"])?;
        search_url
            .query_pairs_mut()
            .append_pair("q", &format!("name contains '{escaped}' and trashed = false"))
            .append_pair("pageSize", &max.to_string())
            .append_pair("orderBy", "modifiedTime desc")
            .append_pair("fields", "files(id,name,mimeType,webViewLink,modifiedTime)");
        let result = self.send(self.http.get(search_url)).await?;
        Ok(json!({ "files": result.get("files").cloned().unwrap_or(json!([])) }))
    }

    // ---- Calendar ----

    pub async fn calendar_events(&self, time_min: &str, time_max: Option<&str>, max: u32) -> Result<Value, GoogleError> {
        let mut events_url = url(&self.config.calendar_base, &["calendar", "v3", "calendars", "primary", "events"])?;
        {
            let mut q = events_url.query_pairs_mut();
            q.append_pair("timeMin", time_min).append_pair("singleEvents", "true").append_pair("orderBy", "startTime").append_pair("maxResults", &max.to_string());
            if let Some(time_max) = time_max {
                q.append_pair("timeMax", time_max);
            }
        }
        let result = self.send(self.http.get(events_url)).await?;
        let events: Vec<Value> = result
            .get("items")
            .and_then(|i| i.as_array())
            .into_iter()
            .flatten()
            .map(|e| json!({ "summary": e.get("summary"), "start": e.get("start"), "end": e.get("end"), "location": e.get("location"), "link": e.get("htmlLink") }))
            .collect();
        Ok(json!({ "events": events }))
    }

    pub async fn calendar_add_event(&self, event: &Value) -> Result<Value, GoogleError> {
        let mut add_url = url(&self.config.calendar_base, &["calendar", "v3", "calendars", "primary", "events"])?;
        if event.get("attendees").and_then(|a| a.as_array()).is_some_and(|a| !a.is_empty()) {
            add_url.query_pairs_mut().append_pair("sendUpdates", "all");
        }
        let created = self.send(self.http.post(add_url).json(event)).await?;
        Ok(json!({ "event_id": created.get("id"), "url": created.get("htmlLink") }))
    }
}

pub fn doc_link(id: &str) -> String {
    format!("https://docs.google.com/document/d/{id}/edit")
}

pub fn sheet_link(id: &str) -> String {
    format!("https://docs.google.com/spreadsheets/d/{id}/edit")
}

fn header(message: &Value, name: &str) -> Value {
    message
        .pointer("/payload/headers")
        .and_then(|h| h.as_array())
        .and_then(|headers| headers.iter().find(|h| h.get("name").and_then(|n| n.as_str()).is_some_and(|n| n.eq_ignore_ascii_case(name))))
        .and_then(|h| h.get("value").cloned())
        .unwrap_or(Value::Null)
}

/// The message's plain text, falling back to its HTML with tags stripped.
fn collect_text(part: &Value, out: &mut String) {
    let mime = part.get("mimeType").and_then(|m| m.as_str()).unwrap_or_default();
    let data = part.pointer("/body/data").and_then(|d| d.as_str());
    if let (true, Some(data)) = (mime == "text/plain", data) {
        if let Ok(bytes) = URL_SAFE.decode(data).or_else(|_| URL_SAFE_NO_PAD.decode(data)) {
            out.push_str(&String::from_utf8_lossy(&bytes));
            return;
        }
    }
    let parts = part.get("parts").and_then(|p| p.as_array());
    for child in parts.into_iter().flatten() {
        collect_text(child, out);
    }
    if out.is_empty() && mime == "text/html" {
        if let Some(Ok(bytes)) = data.map(|d| URL_SAFE.decode(d).or_else(|_| URL_SAFE_NO_PAD.decode(d))) {
            out.push_str(&strip_tags(&String::from_utf8_lossy(&bytes)));
        }
    }
}

fn strip_tags(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_found_inside_a_multipart_message() {
        let message = json!({ "mimeType": "multipart/alternative", "parts": [
            { "mimeType": "text/plain", "body": { "data": URL_SAFE.encode("Invoice total: Rp 2.850.000") } },
            { "mimeType": "text/html", "body": { "data": URL_SAFE.encode("<p>ignored</p>") } }
        ]});
        let mut out = String::new();
        collect_text(&message, &mut out);
        assert_eq!(out, "Invoice total: Rp 2.850.000");
    }

    #[test]
    fn html_only_messages_are_read_without_their_tags() {
        let message = json!({ "mimeType": "text/html", "body": { "data": URL_SAFE.encode("<p>Hi <b>Ana</b></p>") } });
        let mut out = String::new();
        collect_text(&message, &mut out);
        assert_eq!(out, "Hi Ana");
    }
}
