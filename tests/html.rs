use rphtml::config::{ParseOptions, RenderOptions};
use rphtml::parser::*;
use rphtml::types::{GenResult, HResult};

fn parse(code: &str) -> GenResult<DocHolder> {
	let doc = Doc::parse(code, Default::default())?;
	Ok(doc)
}
fn render(doc: &DocHolder) -> String {
	doc.render(&Default::default())
}

fn to_static_str(content: String) -> &'static str {
	Box::leak(content.into_boxed_str())
}

fn get_attr_content(v: &Option<AttrData>) -> Option<&str> {
	v.as_ref()
		.map(|AttrData { content, .. }| to_static_str(content.iter().collect::<String>()))
}

#[test]
fn test_doctype() -> HResult {
	// normal doctype
	let code = r##"<!doctype html PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN" "http://www.w3.org/TR/html4/loose.dtd">"##;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	// html doctype, uppercase
	let code = r##"<!DOCTYPE html>"##;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	// wrong doctype name
	let code = r##"<!DOCTYPES html>"##;
	assert!(parse(code).is_err());
	// html doctype, with no attribute
	let code = r##"<!DOCTYPE>"##;
	assert!(parse(code).is_err());
	// doctype end slash, just ignore
	let code = r##"<!DOCTYPE html/>"##;
	assert!(parse(code).is_ok());
	Ok(())
}

#[test]
fn test_pre_tag() -> HResult {
	// pre render should keep spaces
	let code = r##"
    <pre> spaces </pre>
  "##;
	let doc = parse(code)?;
	let options = RenderOptions {
		minify_spaces: true,
		..Default::default()
	};
	assert_eq!(doc.render(&options), r#"<pre> spaces </pre>"#);
	// mix pre and others
	let code = r##"<Pre> abc </PRE> <a>  </a>"##;
	let doc = parse(code)?;
	let options = RenderOptions {
		lowercase_tagname: true,
		minify_spaces: true,
		..Default::default()
	};
	assert_eq!(doc.render(&options), r#"<pre> abc </pre><a> </a>"#);
	// pre tag
	assert!(parse(r##"<pre><a></a></pre>"##).is_ok());
	Ok(())
}

#[test]
fn test_title_tag() -> HResult {
	let code = "<title><div>&nbsp;tags are allowed here</div></title>";
	let doc = parse(code)?;
	let encoded = doc.render(&RenderOptions {
		encode_content: true,
		..Default::default()
	});
	assert_eq!(
		encoded,
		"<title>&lt;div&gt;&amp;nbsp;tags are allowed here&lt;/div&gt;</title>"
	);
	assert_eq!(render(&doc), code);
	Ok(())
}

#[test]
fn test_textarea_tag() -> HResult {
	let code = r#"<textarea name="content"><div> tags are allowed here</div></textarea>"#;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	Ok(())
}

#[test]
fn test_void_elements() {
	// void element allowed
	let code = r#"<br><br/><br >"#;
	assert!(parse(code).is_ok());
	// void element don't allowed
	let code = r#"<br></br>"#;
	assert!(parse(code).is_err());
}

#[test]
fn test_script_tag() -> HResult {
	let code = r#"
  <script type=text/javascript>
    var div=$("<div></div>");
    var script = "</script" + '>';
  </script>
  "#;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	Ok(())
}

#[test]
fn test_style_tag() -> HResult {
	let code = r#"
  <style type="text/css">
    body{
      color: lightblue;
    }
  </style>
  "#;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	Ok(())
}

#[test]
fn test_attrs() -> HResult {
	// a complex attribute
	let code = r##"<img src=http://site.com/abc.jpg alt =abc defer data-width= 60 data-name = "abc" data-size='60*60' data-msg="quote must escape.&quot;" class="js-img img" xpath="A\B\C\" />"##;
	let doc = parse(code)?;
	let root = doc.get_root_node();
	let root = root.borrow();
	let childs = root.childs.as_ref().unwrap();
	let first_child = &childs[0];
	let first_child = &first_child.borrow();
	let attrs = &first_child.meta.as_ref().unwrap().borrow().attrs;
	// attribute 1
	assert_eq!(get_attr_content(&attrs[0].key), Some("src"));
	assert_eq!(
		get_attr_content(&attrs[0].value),
		Some("http://site.com/abc.jpg")
	);
	// attribute 2
	assert_eq!(get_attr_content(&attrs[1].key), Some("alt"));
	assert_eq!(get_attr_content(&attrs[1].value), Some("abc"));
	// attribute 3
	assert_eq!(get_attr_content(&attrs[2].key), Some("defer"));
	assert!(&attrs[2].value.is_none());
	// attribute 4
	assert_eq!(get_attr_content(&attrs[3].key), Some("data-width"));
	assert_eq!(get_attr_content(&attrs[3].value), Some("60"));
	// attribute 5
	assert_eq!(get_attr_content(&attrs[4].key), Some("data-name"));
	assert_eq!(get_attr_content(&attrs[4].value), Some("abc"));
	assert_eq!(attrs[4].quote, Some('"'));
	// attribute 6
	assert_eq!(get_attr_content(&attrs[5].key), Some("data-size"));
	assert_eq!(get_attr_content(&attrs[5].value), Some("60*60"));
	assert_eq!(attrs[5].quote, Some('\''));
	// attribute 7
	assert_eq!(get_attr_content(&attrs[6].key), Some("data-msg"));
	assert_eq!(
		get_attr_content(&attrs[6].value),
		Some("quote must escape.&quot;")
	);
	// attribute 8
	assert_eq!(get_attr_content(&attrs[7].key), Some("class"));
	assert_eq!(get_attr_content(&attrs[7].value), Some("js-img img"));
	// attribute 9
	assert_eq!(get_attr_content(&attrs[8].key), Some("xpath"));
	assert_eq!(get_attr_content(&attrs[8].value), Some("A\\B\\C\\"));
	// wrong value
	assert!(parse(r#"<div id"1"></div>"#).is_ok());
	assert_eq!(parse(r#"<div "1"'2'></div>"#).is_ok(), true);
	assert_eq!(parse(r#"<div a="1\""></div>"#).is_ok(), true);
	Ok(())
}

#[test]
fn test_tag_close() -> HResult {
	// tag not closed
	let code = "<div>";
	assert_eq!(parse(code).is_err(), true);
	// allow code auto_fix
	let doc = Doc::parse(
		r#"<div id=1><div id=2><div id=3>3</div>"#,
		ParseOptions {
			auto_fix_unclosed_tag: true,
			..Default::default()
		},
	);
	assert_eq!(doc.is_ok(), true);
	assert_eq!(
		render(&doc?),
		"<div id=1><div id=2><div id=3>3</div></div></div>"
	);
	// wrong tag end
	assert_eq!(parse("<div></p>").is_err(), true);
	Ok(())
}

#[test]
fn test_comment() -> HResult {
	let code = r##"
  <!---
  // this is a comment
  // --allowed
  --->
  "##;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	// remove comments
	assert_eq!(
		doc.render(&RenderOptions {
			minify_spaces: true,
			remove_comment: true,
			..Default::default()
		}),
		""
	);
	// take '->' as comment's content
	let code = r##"<!-- 
  // still in comments->
  -->"##;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	Ok(())
}

#[test]
fn test_end_spaces() -> HResult {
	let code = "<div></div >";
	let doc = parse(code)?;
	assert_eq!(
		doc.render(&RenderOptions {
			remove_endtag_space: true,
			..Default::default()
		}),
		"<div></div>"
	);
	Ok(())
}

#[test]
fn test_svg_tag() -> HResult {
	let code = r#"<svg version="1.1" baseProfile="full" width="300" height="200" xmlns="http://www.w3.org/2000/svg"><rect width="100%" height="100%" fill="red" /><text x="250" y="150" font-family="Verdana" font-size="55"><![CDATA[<div> is something]]></text></svg>"#;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	// wrong svg sub nodes, such as text and other no tag nodes
	let code = r##"<svg>abc</svg>"##;
	assert!(parse(code).is_err());
	// svg allow style,script
	let code = r##"<svg><style></style><script></script></svg>"##;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	Ok(())
}

#[test]
fn test_mathml_tag() -> HResult {
	let code = r#"<math><mrow></mrow></math>"#;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	// wrong math nodes, the same as svg tag
	let code = r##"<math>abc</math>"##;
	assert!(parse(code).is_err());
	// math also can't has style or script tag
	let code = r##"<math><style></style></math>"##;
	assert!(parse(code).is_err());
	Ok(())
}

#[test]
fn test_tag_name() -> HResult {
	// case1
	let code = r#"<Form><Form.Item></Form.Item></Form>"#;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	// case2
	let code = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><script xlink:href="cool-script.js" type="text/ecmascript" /></svg>"#;
	let doc = parse(code)?;
	assert_eq!(render(&doc), code);
	// case3
	let code = r#"<abc<<></abc<<>"#;
	let doc = Doc::parse(code, Default::default())?;
	assert_eq!(render(&doc), code);
	Ok(())
}

#[test]
fn test_inner_html() -> HResult {
	// normal
	let inner_html = r##"<span class="sp">long time</span>"##;
	let code = format!("<div>{}</div>", inner_html);
	let doc = parse(code.as_str())?;
	let inner_code = doc.render(&RenderOptions {
		inner_html: true,
		..Default::default()
	});
	assert_eq!(inner_html, inner_code);
	assert_eq!(render(&doc), code);
	// pre tag
	let inner_html = r##"   minify spaces   "##;
	let code = format!("<pre class='pre'>{}</pre>", inner_html);
	let doc = parse(code.as_str())?;
	let inner_code = doc.render(&RenderOptions {
		inner_html: true,
		minify_spaces: true,
		..Default::default()
	});
	assert_eq!(inner_html, inner_code);
	assert_eq!(render(&doc), code);
	// special tag
	let inner_html = r##"var a = 1;var b = 2;"##;
	let code = format!("<script>{}</script>", inner_html);
	let doc = parse(code.as_str())?;
	let inner_code = doc.render(&RenderOptions {
		inner_html: true,
		..Default::default()
	});
	assert_eq!(inner_html, inner_code);
	assert_eq!(render(&doc), code);
	Ok(())
}
#[test]
#[should_panic]
fn test_wrong_inner_html() {
	let code = "abc";
	let doc = parse(code).unwrap();
	doc.render(&RenderOptions {
		inner_html: true,
		minify_spaces: true,
		..Default::default()
	});
}

#[test]
fn test_minify_spaces() -> HResult {
	// spaces between tags
	let inner_html = r##"<span class="sp">long time</span>"##;
	let code = format!("<div>   {}   </div>", inner_html);
	let doc = parse(code.as_str())?;
	let render_code = doc.render(&RenderOptions {
		minify_spaces: true,
		inner_html: true,
		..Default::default()
	});
	assert_eq!(inner_html, render_code);
	assert_eq!(render(&doc), code);
	// spaces between text
	let code = r#"<div>  whitespaces   repeat     </div>"#;
	let doc = parse(code)?;
	let render_code = doc.render(&RenderOptions {
		minify_spaces: true,
		..Default::default()
	});
	assert_eq!(render_code, r#"<div> whitespaces repeat </div>"#);
	// spaces in pre tag
	let code = r#"<pre>  whitespaces   repeat     </pre>"#;
	let doc = parse(code)?;
	let render_code = doc.render(&RenderOptions {
		minify_spaces: true,
		..Default::default()
	});
	assert_eq!(render_code, code);
	Ok(())
}

#[test]
fn test_self_closing() -> HResult {
	let code = r##"<component is="Header" />"##;
	let doc = Doc::parse(
		code,
		ParseOptions {
			allow_self_closing: true,
			..Default::default()
		},
	)?;
	let render_code = render(&doc);
	assert_eq!(render_code, code);
	assert!(parse(code).is_err());
	Ok(())
}

#[test]
fn test_wrong_tag() {
	let code = r##"<abc#def>"##;
	let result = parse(code);
	assert!(result.is_err());
	// wrong tag
	let code = r##"<123>"##;
	let result = parse(code);
	assert!(result.is_err());
}

#[test]
fn test_wrong_endtag() {
	let code = r##"<a>something</b>"##;
	let result = parse(code);
	assert!(result.is_err());
}

#[test]
fn test_unexpect_char() {
	let code = r##"<a class="" /]"##;
	let result = parse(code);
	assert!(result.is_err());
}

#[test]
fn test_auto_fix_unclosed_tag() {
	fn parse_param(content: &str) -> GenResult<DocHolder> {
		Doc::parse(
			content,
			ParseOptions {
				auto_fix_unclosed_tag: true,
				..Default::default()
			},
		)
	}
	// case1
	let code = r##"<a><b></b>"##;
	assert!(parse(code).is_err());
	assert!(parse_param(code).is_ok());
	// case2
	let code = r##"<b></b><a>text"##;
	assert!(parse(code).is_err());
	assert!(parse_param(code).is_ok());
}

#[test]
fn test_auto_fix_unexpected_endtag() -> HResult {
	fn parse_param(content: &str) -> GenResult<DocHolder> {
		Doc::parse(
			content,
			ParseOptions {
				auto_fix_unexpected_endtag: true,
				auto_fix_unclosed_tag: true,
				..Default::default()
			},
		)
	}
	// case1
	let code = r##"text</a><b></b>"##;
	assert!(parse(code).is_err());
	assert!(parse_param(code).is_ok());
	assert_eq!(
		parse_param(code)?.render(&Default::default()),
		"text<b></b>"
	);
	// case2
	let code = r##"<b>text</a></b>"##;
	assert!(parse(code).is_err());
	assert!(parse_param(code).is_ok());
	assert_eq!(
		parse_param(code)?.render(&Default::default()),
		"<b>text</b>"
	);
	// case3
	let code = r##"<b>text</a</b>"##;
	assert!(parse(code).is_err());
	assert!(parse_param(code).is_ok());
	assert_eq!(
		parse_param(code)?.render(&Default::default()),
		"<b>text</b>"
	);
	Ok(())
}

#[test]
fn test_wrong_doctype() {
	// wrong doctype name
	let code = r##"<!DOCTYP html>"##;
	let result = parse(code);
	assert!(result.is_err());
	// doctype without any attribute
	let code = r##"<!DOCTYPE>"##;
	let result = parse(code);
	assert!(result.is_err());
	// right doctype, has an attribute, but don't validate more
	let code = r##"<!DOCTYPE HTML>"##;
	let result = parse(code);
	assert!(result.is_ok());
}

#[test]
fn test_case_senstive() -> HResult {
	let code = r##"<A></a>"##;
	let result = Doc::parse(
		code,
		ParseOptions {
			case_sensitive_tagname: true,
			..Default::default()
		},
	);
	assert!(result.is_err());
	// self close, allow lowercase or uppercase
	let code = r##"<META>"##;
	let doc = Doc::parse(
		code,
		ParseOptions {
			case_sensitive_tagname: true,
			..Default::default()
		},
	)?;
	assert_eq!(render(&doc), code);
	Ok(())
}

#[test]
fn test_render_decode() -> HResult {
	let code = r##"<div><span>&apos;"</span></div>"##;
	let doc = parse(code)?;
	let render_code = doc.render(&RenderOptions {
		decode_entity: true,
		..Default::default()
	});
	assert_eq!(render_code, "<div><span>'\"</span></div>");
	Ok(())
}

#[test]
fn test_auto_fix_unescaped_lt() -> HResult {
	let code = r##"<div><</div>"##;
	assert!(parse(code).is_err());
	let doc = Doc::parse(
		code,
		ParseOptions {
			auto_fix_unescaped_lt: true,
			..Default::default()
		},
	);
	assert!(doc.is_ok());
	let doc = doc?;
	assert_eq!(render(&doc), "<div>&lt;</div>");
	// child of abstract root
	let code = r##"<<div></div>"##;
	assert!(parse(code).is_err());
	let doc = Doc::parse(
		code,
		ParseOptions {
			auto_fix_unescaped_lt: true,
			..Default::default()
		},
	);
	assert!(doc.is_ok());
	let doc = doc?;
	assert_eq!(render(&doc), "&lt;<div></div>");
	// child of tag node
	let code = r##"<div><<div></div></div>"##;
	assert!(parse(code).is_err());
	let doc = Doc::parse(
		code,
		ParseOptions {
			auto_fix_unescaped_lt: true,
			..Default::default()
		},
	);
	assert!(doc.is_ok());
	let doc = doc?;
	assert_eq!(render(&doc), "<div>&lt;<div></div></div>");
	// prev is text node
	let code = r##"<div>abc<</div>"##;
	assert!(parse(code).is_err());
	let doc = Doc::parse(
		code,
		ParseOptions {
			auto_fix_unescaped_lt: true,
			..Default::default()
		},
	);
	assert!(doc.is_ok());
	let doc = doc?;
	assert_eq!(render(&doc), "<div>abc&lt;</div>");
	// prev is spaces
	let code = r##"<div> <</div>"##;
	assert!(parse(code).is_err());
	let doc = Doc::parse(
		code,
		ParseOptions {
			auto_fix_unescaped_lt: true,
			..Default::default()
		},
	);
	assert!(doc.is_ok());
	let doc = doc?;
	assert_eq!(render(&doc), "<div> &lt;</div>");
	let root = &doc.get_root_node();
	let childs = &root.borrow().childs;
	let childs = childs.as_ref().unwrap();
	let div = &childs[0].borrow();
	let div_childs = div.childs.as_ref().unwrap();
	assert_eq!(div_childs[0].borrow().node_type, NodeType::Text);
	// prev node is self closing tag
	let code = r##"<br><<div></div>"##;
	assert!(parse(code).is_err());
	let doc = Doc::parse(
		code,
		ParseOptions {
			auto_fix_unescaped_lt: true,
			..Default::default()
		},
	);
	assert!(doc.is_ok());
	let doc = doc?;
	assert_eq!(render(&doc), "<br>&lt;<div></div>");
	// prev node is self closing tag
	let code = r##"<p/><<div></div>"##;
	assert!(parse(code).is_err());
	let doc = Doc::parse(
		code,
		ParseOptions {
			allow_self_closing: true,
			auto_fix_unescaped_lt: true,
			..Default::default()
		},
	);
	assert!(doc.is_ok());
	let doc = doc?;
	assert_eq!(render(&doc), "<p />&lt;<div></div>");
	// wrong tag name
	let code = r##"<div><span></span><123</div>"##;
	assert!(parse(code).is_err());
	let doc = Doc::parse(
		code,
		ParseOptions {
			auto_fix_unescaped_lt: true,
			..Default::default()
		},
	);
	assert!(doc.is_ok());
	let doc = doc?;
	assert_eq!(render(&doc), "<div><span></span>&lt;123</div>");
	let root = doc.get_root_node();
	let root = root.borrow();
	let childs = root.childs.as_ref().unwrap();
	let div = childs[0].borrow();
	let div_childs = div.childs.as_ref().unwrap();
	assert_eq!(div_childs.len(), 2);
	assert_eq!(div_childs[1].borrow().node_type, NodeType::Text);
	assert_eq!(div_childs[1].borrow().index, 1);
	Ok(())
}

#[test]
fn test_get_element_by_id() -> HResult {
	let code = r##"<div id="mydiv"></div><p id=haha></p>"##;
	let doc = parse(code)?;
	assert!(doc.get_element_by_id("mydiv").is_some());
	assert!(doc.get_element_by_id("haha").is_some());
	assert!(doc.get_element_by_id("none").is_none());
	Ok(())
}

#[test]
fn test_is_document() -> HResult {
	// normal document
	let code = r##"<!--MYHTML--><!DOCTYPE html> <html></html>   "##;
	let doc = parse(code)?;
	assert_eq!(doc.get_root_node().borrow().is_document(), (true, true));
	// wrong tag or text node
	let code = r##"<!DOCTYPE html><html></html><div></div>"##;
	let doc = parse(code)?;
	assert_eq!(doc.get_root_node().borrow().is_document(), (true, false));
	// wrong html element
	let code = r##"<!DOCTYPE html><body></body>"##;
	let doc = parse(code)?;
	assert_eq!(doc.get_root_node().borrow().is_document(), (false, true));
	Ok(())
}

#[test]
fn test_is_same() -> HResult {
	// normal document
	let code = r##"<div id="same"></div>"##;
	let doc = parse(code)?;
	let root = doc.get_root_node();
	let childs = &root.borrow().childs;
	let childs = childs.as_ref().unwrap();
	let div = &childs[0];
	assert!(Node::is_same(div, &doc.get_element_by_id("same").unwrap()));
	Ok(())
}

#[test]
fn test_eof() -> HResult {
	// not end comment
	let code = r##"<div></div><!--"##;
	assert!(parse(code).is_err());
	// not end text
	let code = r##"<div></div>abc"##;
	assert!(parse(code).is_ok());
	assert_eq!(render(&parse(code)?), "<div></div>abc");
	// not end tag
	let code = r##"<div></div><div>abc"##;
	assert!(parse(code).is_err());
	let doc = Doc::parse(
		code,
		ParseOptions {
			auto_fix_unclosed_tag: true,
			..Default::default()
		},
	);
	assert!(doc.is_ok());
	assert_eq!(render(&doc?), "<div></div><div>abc</div>");
	Ok(())
}

#[test]
fn test_decode() -> HResult {
	let texts = "This  is  a  &gt;";
	let doc = parse(&format!("<div>{}</div>", texts))?;
	let decode_texts = doc.render_text(&RenderOptions {
		decode_entity: true,
		..Default::default()
	});
	assert_eq!(decode_texts, "This  is  a  >");
	// with spaces
	let texts = "This  is a &gt;";
	let doc = parse(&format!("<div>{}</div>", texts))?;
	let decode_texts = doc.render_text(&RenderOptions {
		decode_entity: true,
		minify_spaces: true,
		..Default::default()
	});
	assert_eq!(decode_texts, "This is a >");
	// wrong entity
	let texts = "This  is  a &gt";
	let doc = parse(&format!("<div>{}</div>", texts))?;
	let decode_texts = doc.render_text(&RenderOptions {
		decode_entity: true,
		minify_spaces: true,
		..Default::default()
	});
	assert_eq!(decode_texts, "This is a &gt");
	// wrong entity
	let texts = "This   is   a   &";
	let doc = parse(&format!("<div>{}</div>", texts))?;
	let decode_texts = doc.render_text(&RenderOptions {
		decode_entity: true,
		minify_spaces: true,
		..Default::default()
	});
	assert_eq!(decode_texts, "This is a &");
	Ok(())
}

#[test]
fn test_parse_file() {
	let doc = Doc::parse_file("./cases/full.html", Default::default());
	assert!(doc.is_ok());
}
