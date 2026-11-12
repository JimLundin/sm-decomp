pub fn f(x: Option<Option<u8>>) -> u8 { match x { Some(Some(n)) => 4, Some(None) | None => 0 } }
