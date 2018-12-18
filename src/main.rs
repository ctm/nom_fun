#[macro_use] extern crate nom;
#[macro_use] extern crate nom_trace;

//adds a thread local storage object to store the trace
declare_trace!();

pub fn main() {
  named!(parser<Vec<&[u8]>>,
    //wrap a parser with tr!() to add a trace point
    tr!(preceded!(
      tr!(tag!("data: ")),
      tr!(delimited!(
        tag!("("),
        separated_list!(
          tr!(tag!(",")),
          tr!(nom::digit)
        ),
        tr!(tag!(")"))
      ))
    ))
  );

  println!("parsed: {:?}", parser(&b"data: (1,2,3)"[..]));

  // prints the last parser trace
  print_trace!();

  // the list of trace events can be cleared
  reset_trace!();
}
