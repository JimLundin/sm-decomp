//! Inner doc on line 1.
/// Outer doc.
/** Block doc
spanning two lines */
fn a() {} // trailing
/* outer /* nested */ still a "comment" with 'quotes' */
fn b() {}
/*! inner block doc */
// a line comment holding /* an unclosed block opener
const C: u8 = 1;
