#[test]
fn restored_lane_documents_keep_scoped_source() { for name in ["a","b","c"] { let text = std::fs::read_to_string(format!("docs/drill-{name}.md")).unwrap(); assert!(text.contains("lane")); } }
