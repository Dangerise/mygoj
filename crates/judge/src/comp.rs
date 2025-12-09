pub fn comp(a: &str, b: &str) -> bool {
    let mut a_lines = a.lines();
    let mut b_lines = b.lines();

    loop {
        let a = a_lines.next();
        let b = b_lines.next();
        if a.is_none() ^ b.is_none() {
            return false;
        }
        if a.is_none() && b.is_none() {
            return true;
        }
        let mut a = a.unwrap().split_whitespace();
        let mut b = b.unwrap().split_whitespace();
        loop {
            let a = a.next();
            let b = b.next();
            if a.is_none() ^ b.is_none() {
                return false;
            }
            if a.is_none() && b.is_none() {
                break;
            }
            let a = a.unwrap();
            let b = b.unwrap();
            if a != b {
                return false;
            }
        }
    }
}
