const rphtml = require("rphtml");
const htmlCode = `
<!DOCTYPE html PUBLIC a = b "-//W3C//DTD XHTML 1.0 Transitional//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd">
<div class="header">
  <!--header-->
  <h3>this is header.</h3 >
</div>
`;
const nodeList = rphtml.parse(htmlCode, {
  allow_self_closing: true,
  allow_fix_unclose: false,
  case_sensitive_tagname: false,
});
console.log(nodeList);
const doneCode = rphtml.render(nodeList, {
  always_close_void: false,
  lowercase_tagname: true,
  minify_spaces: true,
  remove_attr_quote: false,
  remove_comment: false,
  remove_endtag_space: true,
});
console.log(doneCode);