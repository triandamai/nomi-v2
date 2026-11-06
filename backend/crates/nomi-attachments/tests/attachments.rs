//! What a file is from its bytes, how much may be attached, and what the server reads out of
//! documents, slides and images itself.

use std::io::Write;

use nomi_attachments::{extract, sniff, Kind, Limits, Sniffed};

fn sniffed(kind: Kind, mime: &str) -> Sniffed {
    Sniffed { kind, mime: mime.to_string() }
}

/// A zip with the given entries, like an Office or OpenDocument file.
fn zip_of(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut out);
        for (name, body) in entries {
            writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
            writer.write_all(body.as_bytes()).unwrap();
        }
        writer.finish().unwrap();
    }
    out.into_inner()
}

const OLE: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0, 0, 0, 0, 0, 0, 0, 0];

#[test]
fn kinds_round_trip_through_their_names() {
    for kind in [Kind::Text, Kind::Pdf, Kind::Document, Kind::Spreadsheet, Kind::Presentation, Kind::Image, Kind::Audio, Kind::Voice, Kind::Video, Kind::Other] {
        assert_eq!(Kind::parse(kind.as_str()), Some(kind));
    }
    assert_eq!(Kind::parse("file"), Some(Kind::Text), "the old name for text");
    assert_eq!(Kind::parse("zip"), None);
    assert!(Kind::Voice.needs_model() && Kind::Video.needs_model() && !Kind::Pdf.needs_model());
}

#[test]
fn programs_and_empty_files_are_refused() {
    use nomi_attachments::kind::Refused;
    assert_eq!(sniff(b"", "a.txt", "text/plain", false), Err(Refused::Empty));
    assert_eq!(sniff(b"MZ\x90\x00rest", "setup.pdf", "application/pdf", false), Err(Refused::Executable));
    assert_eq!(sniff(b"\x7fELF\x02\x01\x01", "x", "", false), Err(Refused::Executable));
    assert_eq!(sniff(b"\xcf\xfa\xed\xfe\x07", "x", "", false), Err(Refused::Executable));
}

#[test]
fn media_is_told_apart_by_bytes_then_by_name_and_claimed_type() {
    let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
    assert_eq!(sniff(&png, "photo.jpg", "image/jpeg", false).unwrap(), sniffed(Kind::Image, "image/png"));
    assert_eq!(sniff(b"%PDF-1.7\n", "x", "", false).unwrap().kind, Kind::Pdf);
    let mp3 = b"ID3\x04\x00\x00\x00\x00\x00\x00";
    assert_eq!(sniff(mp3, "song.mp3", "", false).unwrap().kind, Kind::Audio);
    assert_eq!(sniff(mp3, "note.mp3", "", true).unwrap().kind, Kind::Voice);

    let webm = [0x1A, 0x45, 0xDF, 0xA3, 0x9F, 0x42, 0x86, 0x81, 0x01, 0x42, 0x82, 0x84, b'w', b'e', b'b', b'm'];
    assert_eq!(sniff(&webm, "screen.webm", "video/webm", false).unwrap(), sniffed(Kind::Video, "video/webm"));
    assert_eq!(sniff(&webm, "note.webm", "audio/webm;codecs=opus", false).unwrap(), sniffed(Kind::Audio, "audio/webm"));
    assert_eq!(sniff(&webm, "rec.webm", "", true).unwrap(), sniffed(Kind::Voice, "audio/webm"));
}

#[test]
fn office_files_follow_their_container_and_name() {
    assert_eq!(sniff(OLE, "budget.xls", "", false).unwrap().kind, Kind::Spreadsheet);
    assert_eq!(sniff(OLE, "deck.ppt", "", false).unwrap().kind, Kind::Presentation);
    assert_eq!(sniff(OLE, "letter.doc", "", false).unwrap(), sniffed(Kind::Document, "application/msword"));

    let plain_zip = zip_of(&[("readme.txt", "hi")]);
    assert_eq!(sniff(&plain_zip, "a.docx", "", false).unwrap().kind, Kind::Document);
    assert_eq!(sniff(&plain_zip, "a.xlsx", "", false).unwrap().kind, Kind::Spreadsheet);
    assert_eq!(sniff(&plain_zip, "a.pptx", "", false).unwrap().kind, Kind::Presentation);
    assert_eq!(sniff(&plain_zip, "a.zip", "", false).unwrap().kind, Kind::Other);
}

#[test]
fn text_is_read_as_text_with_a_type_from_its_name() {
    for (name, mime) in [("a.csv", "text/csv"), ("a.tsv", "text/tab-separated-values"), ("a.md", "text/markdown"), ("a.yml", "application/yaml"), ("a.log", "text/plain")] {
        assert_eq!(sniff(b"date,amount\n2026-11-01,45000\n", name, "", false).unwrap(), sniffed(Kind::Text, mime), "{name}");
    }
    // A last character cut off mid-way is still text.
    let mut cut = "harga Rp 45.000 — ".as_bytes().to_vec();
    cut.extend_from_slice(&"é".as_bytes()[..1]);
    assert_eq!(sniff(&cut, "a.txt", "", false).unwrap().kind, Kind::Text);
    // Bytes that are neither text nor anything known.
    assert_eq!(sniff(&[0x01, 0x02, 0x03, 0xff, 0xfe, 0x00, 0x10], "blob", "", false).unwrap(), sniffed(Kind::Other, "application/octet-stream"));
    assert_eq!(sniff(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10], "ctl", "", false).unwrap().kind, Kind::Other, "mostly control characters");
}

#[test]
fn limits_come_from_the_environment_and_videos_get_their_own() {
    std::env::set_var("ATTACHMENT_MAX_FILE_MB", "5");
    std::env::set_var("ATTACHMENT_MAX_VIDEO_MB", " 200 ");
    std::env::set_var("ATTACHMENT_MAX_MESSAGE_MB", "nope");
    std::env::set_var("ATTACHMENT_MAX_FILES", "3");
    let limits = Limits::from_env();
    assert_eq!(limits.max_file_bytes, 5 * 1024 * 1024);
    assert_eq!(limits.max_video_bytes, 200 * 1024 * 1024);
    assert_eq!(limits.max_message_bytes, Limits::default().max_message_bytes);
    assert_eq!(limits.max_files, 3);
    assert_eq!(limits.max_upload_bytes(), limits.max_video_bytes);
    assert_eq!(limits.max_bytes_for(Kind::Video), limits.max_video_bytes);
    assert_eq!(limits.max_bytes_for(Kind::Pdf), limits.max_file_bytes);
}

#[test]
fn word_and_opendocument_text_is_read_and_old_formats_are_explained() {
    let docx = zip_of(&[("word/document.xml", "<w:document><w:body><w:p><w:r><w:t>Invoice</w:t></w:r><w:tab/><w:r><w:t>Rp 2.850.000</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:t>A</w:t></w:p></w:tc></w:tr></w:tbl><w:p><w:br/></w:p></w:body></w:document>")]);
    let read = extract(&docx, &sniffed(Kind::Document, "application/vnd.openxmlformats-officedocument.wordprocessingml.document"));
    let text = read.text.unwrap();
    assert!(text.starts_with("Invoice\tRp 2.850.000\n"), "{text:?}");
    assert!(text.contains("A\n |"), "cells are separated: {text:?}");
    assert_eq!(read.details["words"], 5);

    let odt = zip_of(&[("content.xml", "<office:text><text:h>Title</text:h><text:p>Body<text:line-break/>more</text:p></office:text>")]);
    assert_eq!(extract(&odt, &sniffed(Kind::Document, "application/vnd.oasis.opendocument.text")).text.unwrap(), "Title\nBody\nmore");

    let neither = zip_of(&[("other.xml", "<x/>")]);
    assert!(extract(&neither, &sniffed(Kind::Document, "application/vnd.oasis.opendocument.text")).details["unreadable"].is_string());
    assert!(extract(b"not a zip", &sniffed(Kind::Document, "application/vnd.oasis.opendocument.text")).details["unreadable"].is_string());
    let doc = extract(OLE, &sniffed(Kind::Document, "application/msword"));
    assert!(doc.details["unreadable"].as_str().unwrap().contains(".doc"));

    let rtf = extract(br"{\rtf1\ansi{\fonttbl\f0 Arial;}\f0\fs24 Hello\par World\tab x \{braces\}\u-123 ok}", &sniffed(Kind::Document, "application/rtf"));
    let text = rtf.text.unwrap();
    assert!(text.contains("Hello\nWorld\tx {braces}"), "{text:?}");
}

#[test]
fn slides_are_read_in_order() {
    let pptx = zip_of(&[
        ("ppt/slides/slide2.xml", "<p:sld><a:p><a:t>Second</a:t></a:p></p:sld>"),
        ("ppt/slides/slide1.xml", "<p:sld><a:p><a:t>First</a:t><a:tab/><a:t>slide</a:t></a:p></p:sld>"),
        ("ppt/slides/slideX.xml", "<p:sld/>"),
    ]);
    let read = extract(&pptx, &sniffed(Kind::Presentation, "application/vnd.openxmlformats-officedocument.presentationml.presentation"));
    assert_eq!(read.text.unwrap(), "## Slide 1\nFirst\tslide\n\n## Slide 2\nSecond");
    assert_eq!(read.details["slides"], 2);

    let odp = zip_of(&[("content.xml", "<office:presentation><text:p>Agenda</text:p></office:presentation>")]);
    assert_eq!(extract(&odp, &sniffed(Kind::Presentation, "application/vnd.oasis.opendocument.presentation")).text.unwrap(), "Agenda");
    assert!(extract(OLE, &sniffed(Kind::Presentation, "application/vnd.ms-powerpoint")).details["unreadable"].as_str().unwrap().contains(".ppt"));
    assert!(extract(b"junk", &sniffed(Kind::Presentation, "application/vnd.oasis.opendocument.presentation")).details["unreadable"].is_string());
    assert!(extract(b"junk", &sniffed(Kind::Spreadsheet, "application/vnd.ms-excel")).details["unreadable"].is_string());
}

#[test]
fn images_get_a_small_preview_and_what_cannot_be_decoded_goes_to_a_model() {
    let big = image::RgbImage::from_pixel(2000, 1000, image::Rgb([200, 100, 50]));
    let mut png = Vec::new();
    image::DynamicImage::ImageRgb8(big).write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).unwrap();
    let read = extract(&png, &sniffed(Kind::Image, "image/png"));
    let preview = image::load_from_memory(&read.preview_jpeg.expect("a preview")).unwrap();
    assert_eq!((preview.width(), preview.height()), (1568, 784));
    assert!(read.needs_model);

    let heic = extract(b"\0\0\0\x18ftypheic", &sniffed(Kind::Image, "image/heic"));
    assert!(heic.preview_jpeg.is_none() && heic.needs_model);

    assert!(extract(b"x", &sniffed(Kind::Video, "video/mp4")).needs_model);
    assert!(extract(b"x", &sniffed(Kind::Other, "application/zip")).details["unreadable"].is_string());
    let text = extract(b"  a\r\nb  \n", &sniffed(Kind::Text, "text/plain"));
    assert_eq!(text.text.as_deref(), Some("a\nb"));
}
